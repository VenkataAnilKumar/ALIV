//! Vault operations that combine storage, parsing and extraction: ingestion,
//! file-rename proposals with an approval gate and undo, and the model-call
//! policy check.

use crate::chunk::{chunk_for_offset, chunk_text};
use crate::error::{Result, VaultError};
use crate::extract;
use crate::model::*;
use crate::ocr::{self, OcrEngine};
use crate::store::{now, Store};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Extensions ingested as documents. Images are read by OCR, as are PDFs
/// with no text layer.
const DOC_EXTS: &[&str] = &["pdf", "docx", "txt", "md", "eml", "html", "htm", "csv", "xlsx", "xls", "pptx", "rtf"];
const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "tif", "tiff", "heic"];
/// Below this many characters a parse is treated as "no text": a scanned PDF
/// often carries a stray page number or producer string.
const MIN_TEXT_CHARS: usize = 20;
const MAX_FILES_PER_RUN: usize = 5000;

pub struct Vault {
    pub store: Store,
    ocr: Option<Box<dyn OcrEngine>>,
}

impl Vault {
    /// A vault that reads scans with Tesseract when it is installed.
    pub fn new(store: Store) -> Self {
        Self::with_ocr(store, ocr::Tesseract::detect().map(|t| Box::new(t) as Box<dyn OcrEngine>))
    }

    pub fn with_ocr(store: Store, ocr: Option<Box<dyn OcrEngine>>) -> Self {
        Self { store, ocr }
    }

    // ---------------------------------------------------------- ingestion

    /// Indexes files and folders (recursively) into a matter. Unchanged files
    /// (same content hash) are left as they are.
    pub fn ingest_paths(&self, matter_id: &str, paths: &[String]) -> Result<IngestReport> {
        self.store.get_matter(matter_id)?;
        let mut report = IngestReport::default();
        let mut files: Vec<PathBuf> = Vec::new();
        for p in paths {
            let path = PathBuf::from(p);
            if path.is_dir() {
                for entry in walkdir::WalkDir::new(&path).follow_links(false).into_iter().filter_map(|e| e.ok()) {
                    let name = entry.file_name().to_string_lossy();
                    if entry.file_type().is_file() && !name.starts_with('.') && !name.starts_with("~$") {
                        files.push(entry.into_path());
                    }
                }
            } else if path.is_file() {
                files.push(path);
            } else {
                report.failed += 1;
                report.errors.push(format!("{p}: not found"));
            }
        }
        files.sort();
        files.dedup();
        files.truncate(MAX_FILES_PER_RUN);

        for file in files {
            let ext = file.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            if !DOC_EXTS.contains(&ext.as_str()) && !IMAGE_EXTS.contains(&ext.as_str()) {
                continue;
            }
            report.scanned += 1;
            match self.ingest_file(matter_id, &file, &ext) {
                Ok(Outcome::Indexed { ocr }) => {
                    report.indexed += 1;
                    report.ocr += usize::from(ocr);
                }
                Ok(Outcome::Unchanged) => report.unchanged += 1,
                Ok(Outcome::Skipped(reason)) => {
                    report.skipped += 1;
                    report.errors.push(format!("{}: {reason}", file.display()));
                }
                Err(e) => {
                    report.failed += 1;
                    report.errors.push(format!("{}: {e}", file.display()));
                }
            }
        }
        self.store.audit(
            Some(matter_id),
            "ingest",
            &format!(
                "Indexed {} file(s); {} unchanged, {} skipped, {} failed",
                report.indexed, report.unchanged, report.skipped, report.failed
            ),
            serde_json::json!({ "paths": paths, "report": &report }),
        )?;
        Ok(report)
    }

    fn ingest_file(&self, matter_id: &str, file: &Path, ext: &str) -> Result<Outcome> {
        let path = file.to_string_lossy().to_string();
        let bytes = std::fs::read(file)?;
        let sha = format!("{:x}", Sha256::digest(&bytes));
        let existing = self.store.find_document_by_path(matter_id, &path)?;
        if let Some((_, old_sha)) = &existing {
            if *old_sha == sha {
                return Ok(Outcome::Unchanged);
            }
        }
        let mut doc = Document {
            id: existing.map(|(id, _)| id).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            matter_id: matter_id.to_string(),
            file_name: file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
            path,
            ext: ext.to_string(),
            size: bytes.len() as i64,
            status: "indexed".into(),
            error: None,
            doc_type: None,
            char_count: 0,
            chunk_count: 0,
            ingested_at: now(),
            ocr_confidence: None,
            summary: None,
        };

        let has_text = |t: &str| t.trim().chars().count() >= MIN_TEXT_CHARS;
        let text = if IMAGE_EXTS.contains(&ext) {
            None
        } else {
            match parse(file, ext) {
                Ok(t) if has_text(&t) => Some(t),
                Ok(_) if ext != "pdf" => return self.record_skipped(doc, &sha, "no extractable text".into()),
                Err(reason) if ext != "pdf" => return self.record_skipped(doc, &sha, reason),
                // A PDF with no text layer is a scan. The parser reports some
                // as empty and refuses others, so both go to OCR.
                Ok(_) => None,
                Err(reason) => {
                    log::debug!("{}: no text layer ({reason}); trying OCR", file.display());
                    None
                }
            }
        };
        let text = match text {
            Some(t) => t,
            None => match self.read_scan(file, ext) {
                Ok(scan) if has_text(&scan.text) => {
                    doc.ocr_confidence = Some(scan.confidence);
                    scan.text
                }
                Ok(_) => return self.record_skipped(doc, &sha, "scanned document: OCR found no readable text".into()),
                Err(reason) => return self.record_skipped(doc, &sha, reason),
            },
        };

        let chunks = chunk_text(&text);
        // A value read from a scan is only as sure as the OCR that read it, so
        // a poor scan lands its fields in "needs review".
        let ocr_factor = doc.ocr_confidence.unwrap_or(1.0);
        let fields: Vec<Field> = extract::extract_all(&text)
            .into_iter()
            .map(|e| Field {
                key: e.key.to_string(),
                value: e.value,
                confidence: e.confidence * ocr_factor,
                chunk_ordinal: chunk_for_offset(&chunks, e.offset).map(|o| o as i64),
                snippet: e.snippet,
            })
            .collect();
        doc.doc_type = fields.iter().find(|f| f.key == extract::DOC_TYPE).map(|f| f.value.clone());
        doc.char_count = text.chars().count() as i64;
        doc.summary = Some(extract::summarize(&fields.iter().map(|f| (f.key.as_str(), f.value.as_str())).collect::<Vec<_>>(), &text));
        let texts: Vec<String> = chunks.into_iter().map(|c| c.text).collect();
        self.store.upsert_document(&doc, &sha, &texts, &fields)?;
        Ok(Outcome::Indexed { ocr: doc.ocr_confidence.is_some() })
    }

    /// A health report for support. Holds counts and reasons, never document
    /// text, so it can be shared.
    pub fn diagnostics(&self) -> Result<VaultDiagnostics> {
        let audit = self.store.verify_audit()?;
        let mut report = self.store.diagnostics_counts(audit)?;
        if let Some(t) = ocr::Tesseract::detect() {
            report.tesseract = Some(t.tesseract_path().display().to_string());
            report.pdftoppm = t.pdftoppm_path().map(|p| p.display().to_string());
        }
        Ok(report)
    }

    fn read_scan(&self, file: &Path, ext: &str) -> std::result::Result<ocr::OcrText, String> {
        match &self.ocr {
            Some(engine) => engine.recognize(file, ext),
            None => Err(ocr::NOT_INSTALLED.to_string()),
        }
    }

    /// Keeps an unreadable file visible in the matter, with its reason.
    fn record_skipped(&self, mut doc: Document, sha: &str, reason: String) -> Result<Outcome> {
        doc.status = "skipped".into();
        doc.error = Some(reason.clone());
        self.store.upsert_document(&doc, sha, &[], &[])?;
        Ok(Outcome::Skipped(reason))
    }

    // ------------------------------------------------------- file actions

    /// Proposes `{DocType}_{Counterparty}_{YYYY-MM-DD}.{ext}` renames for the
    /// matter's documents. Nothing on disk changes until a proposal is
    /// approved and executed.
    pub fn propose_renames(&self, matter_id: &str) -> Result<Vec<Action>> {
        self.store.clear_open_proposals(matter_id)?;
        let mut taken: HashSet<String> = HashSet::new();
        let mut out = Vec::new();
        for row in self.store.review_grid(matter_id)? {
            let doc = &row.document;
            let field = |k: &str| row.fields.iter().find(|f| f.key == k);
            let doc_type = field(extract::DOC_TYPE).map(|f| f.value.clone()).unwrap_or_else(|| "Document".into());
            let party = field(extract::PARTIES).map(|f| f.value.split(" · ").next().unwrap_or("").to_string());
            let date = field(extract::EFFECTIVE_DATE).or_else(|| field(extract::EXPIRY_DATE)).map(|f| f.value.clone());

            let mut parts = vec![slug(&doc_type)];
            if let Some(p) = party.as_deref().filter(|p| !p.is_empty()) {
                parts.push(slug(p));
            }
            if let Some(d) = &date {
                parts.push(d.clone());
            }
            let stem = parts.into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join("_");
            let from = PathBuf::from(&doc.path);
            let dir = from.parent().map(Path::to_path_buf).unwrap_or_default();
            let ext = if doc.ext.is_empty() { String::new() } else { format!(".{}", doc.ext) };
            let mut target = dir.join(format!("{stem}{ext}"));
            if target == from {
                continue;
            }
            let mut n = 2;
            while taken.contains(&target.to_string_lossy().to_string()) || (target.exists() && target != from) {
                target = dir.join(format!("{stem}_{n}{ext}"));
                n += 1;
            }
            taken.insert(target.to_string_lossy().to_string());

            let weak = |k: &str| field(k).map(|f| f.confidence < 0.75).unwrap_or(true);
            let risk = if doc_type == "Document" || party.is_none() {
                "high"
            } else if date.is_none() || weak(extract::DOC_TYPE) || weak(extract::PARTIES) {
                "medium"
            } else {
                "low"
            };
            let mut reason = format!("Type {doc_type}");
            if let Some(p) = &party {
                reason.push_str(&format!(", party {p}"));
            }
            if let Some(d) = &date {
                reason.push_str(&format!(", dated {d}"));
            }
            let ts = now();
            let action = Action {
                id: uuid::Uuid::new_v4().to_string(),
                matter_id: matter_id.to_string(),
                document_id: doc.id.clone(),
                kind: "rename".into(),
                from_path: doc.path.clone(),
                to_path: target.to_string_lossy().to_string(),
                reason,
                risk: risk.into(),
                status: "proposed".into(),
                error: None,
                created_at: ts.clone(),
                updated_at: ts,
            };
            self.store.insert_action(&action)?;
            out.push(action);
        }
        self.store.audit(
            Some(matter_id),
            "actions_proposed",
            &format!("Proposed {} rename(s) for review", out.len()),
            serde_json::json!({ "count": out.len() }),
        )?;
        Ok(out)
    }

    pub fn set_action_status(&self, id: &str, status: &str) -> Result<Action> {
        let a = self.store.get_action(id)?;
        let allowed = matches!((a.status.as_str(), status), ("proposed", "approved") | ("proposed", "rejected") | ("approved", "proposed") | ("approved", "rejected") | ("rejected", "proposed"));
        if !allowed {
            return Err(VaultError::Invalid(format!("cannot change a {} action to {status}", a.status)));
        }
        let updated = self.store.update_action_status(id, status, None)?;
        self.store.audit(
            Some(&a.matter_id),
            &format!("action_{status}"),
            &format!("Rename {} → {} marked {status}", file_name(&a.from_path), file_name(&a.to_path)),
            serde_json::json!({ "actionId": id }),
        )?;
        Ok(updated)
    }

    /// Executes approved actions only. Proposed actions are never touched:
    /// the approval step cannot be skipped.
    pub fn execute_approved(&self, matter_id: &str) -> Result<Vec<Action>> {
        let mut done = Vec::new();
        for a in self.store.list_actions(matter_id)?.into_iter().filter(|a| a.status == "approved") {
            let result = (|| -> Result<()> {
                let from = Path::new(&a.from_path);
                let to = Path::new(&a.to_path);
                if !from.exists() {
                    return Err(VaultError::Invalid("source file no longer exists".into()));
                }
                if to.exists() {
                    return Err(VaultError::Invalid("a file already exists at the target name".into()));
                }
                std::fs::rename(from, to)?;
                self.store.set_document_path(&a.document_id, &a.to_path, &file_name(&a.to_path))?;
                Ok(())
            })();
            let (status, err) = match &result {
                Ok(()) => ("executed", None),
                Err(e) => ("failed", Some(e.to_string())),
            };
            let updated = self.store.update_action_status(&a.id, status, err.as_deref())?;
            self.store.audit(
                Some(matter_id),
                if result.is_ok() { "action_executed" } else { "action_failed" },
                &format!("Rename {} → {}{}", file_name(&a.from_path), file_name(&a.to_path), err.map(|e| format!(" failed: {e}")).unwrap_or_default()),
                serde_json::json!({ "actionId": a.id, "from": a.from_path, "to": a.to_path }),
            )?;
            done.push(updated);
        }
        Ok(done)
    }

    pub fn undo_action(&self, id: &str) -> Result<Action> {
        let a = self.store.get_action(id)?;
        if a.status != "executed" {
            return Err(VaultError::Invalid(format!("only executed actions can be undone (this one is {})", a.status)));
        }
        let (from, to) = (Path::new(&a.from_path), Path::new(&a.to_path));
        if !to.exists() {
            return Err(VaultError::Invalid("renamed file is no longer at its new location".into()));
        }
        if from.exists() {
            return Err(VaultError::Invalid("a file already exists at the original name".into()));
        }
        std::fs::rename(to, from)?;
        self.store.set_document_path(&a.document_id, &a.from_path, &file_name(&a.from_path))?;
        let updated = self.store.update_action_status(id, "undone", None)?;
        self.store.audit(
            Some(&a.matter_id),
            "action_undone",
            &format!("Undid rename; restored {}", file_name(&a.from_path)),
            serde_json::json!({ "actionId": id }),
        )?;
        Ok(updated)
    }

    // ------------------------------------------------------ model egress

    /// The single place a model call is approved. Local-only matters can
    /// never send passages to a cloud provider. Every decision is logged.
    pub fn authorize_model_call(&self, req: &ModelCallRequest) -> Result<ModelCallDecision> {
        let matter = self.store.get_matter(&req.matter_id)?;
        let location = match req.location.as_str() {
            "local" | "cloud" => req.location.as_str(),
            other => return Err(VaultError::Invalid(format!("unknown model location \"{other}\""))),
        };
        let allowed = !(location == "cloud" && matter.policy == "local_only");
        let reason = if allowed {
            format!("{} model allowed by {} policy", location, matter.policy)
        } else {
            "This matter is Local only. Choose a local model, or change the matter policy to Hybrid.".to_string()
        };
        let summary = if allowed {
            format!("Sent {} chars from {} passage(s) to {} ({}, {})", req.chars_sent, req.source_count, req.model, req.provider, location)
        } else {
            format!("Blocked call to {} ({}): matter is local only", req.model, req.provider)
        };
        let audit_id = self.store.audit(
            Some(&req.matter_id),
            if allowed { "model_call" } else { "model_call_blocked" },
            &summary,
            serde_json::to_value(req).unwrap_or_default(),
        )?;
        Ok(ModelCallDecision { allowed, reason, audit_id })
    }

    // ------------------------------------------------------------- export

    pub fn export_register_csv(&self, matter_id: &str, dest: &Path) -> Result<usize> {
        let rows = self.store.review_grid(matter_id)?;
        let keys = [
            extract::DOC_TYPE,
            extract::PARTIES,
            extract::EFFECTIVE_DATE,
            extract::EXPIRY_DATE,
            extract::RENEWAL_DEADLINE,
            extract::AMOUNT,
            extract::NOTICE_PERIOD,
            extract::GOVERNING_LAW,
            extract::PENALTY,
        ];
        let mut out = String::from("File,Path,Type,Parties,Effective date,Expiry / due date,Renewal deadline,Amount,Notice period,Governing law,Penalty\n");
        for r in &rows {
            let mut cells = vec![csv_cell(&r.document.file_name), csv_cell(&r.document.path)];
            for k in keys {
                cells.push(csv_cell(r.fields.iter().find(|f| f.key == k).map(|f| f.value.as_str()).unwrap_or("")));
            }
            out.push_str(&cells.join(","));
            out.push('\n');
        }
        std::fs::write(dest, out)?;
        self.store.audit(
            Some(matter_id),
            "export",
            &format!("Exported register ({} rows) to {}", rows.len(), dest.display()),
            serde_json::json!({ "dest": dest }),
        )?;
        Ok(rows.len())
    }
}

enum Outcome {
    Indexed { ocr: bool },
    Unchanged,
    Skipped(String),
}

fn parse(file: &Path, ext: &str) -> std::result::Result<String, String> {
    let path = file.to_string_lossy();
    let text = match ext {
        "eml" => std::fs::read(file).map(|b| clean_email(&String::from_utf8_lossy(&b))).map_err(|e| e.to_string())?,
        "rtf" => std::fs::read(file).map(|b| strip_rtf(&String::from_utf8_lossy(&b))).map_err(|e| e.to_string())?,
        _ => tauri_plugin_rag::parser::parse_document(&path, ext).map_err(|e| e.to_string())?,
    };
    Ok(text)
}

/// Keeps the headers a reader cares about and the text/plain body parts.
fn clean_email(raw: &str) -> String {
    let mut out = String::new();
    let (headers, body) = raw.split_once("\n\n").or_else(|| raw.split_once("\r\n\r\n")).unwrap_or((raw, ""));
    for line in headers.lines() {
        let l = line.to_lowercase();
        if ["from:", "to:", "cc:", "date:", "subject:"].iter().any(|h| l.starts_with(h)) {
            out.push_str(line.trim());
            out.push('\n');
        }
    }
    out.push('\n');
    for line in body.lines() {
        let t = line.trim_end();
        let lower = t.to_lowercase();
        if t.starts_with("--") || lower.starts_with("content-") || lower.starts_with("mime-version") {
            continue;
        }
        out.push_str(t);
        out.push('\n');
    }
    out
}

fn strip_rtf(raw: &str) -> String {
    let no_ctrl = regex::Regex::new(r"\\[a-zA-Z]+-?\d* ?|[{}]").unwrap().replace_all(raw, "");
    no_ctrl.to_string()
}

fn slug(s: &str) -> String {
    let words: Vec<String> = s
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .filter(|w| !matches!(w.to_lowercase().as_str(), "llc" | "inc" | "ltd" | "llp" | "corp" | "co" | "the" | "plc" | "gmbh"))
        .take(4)
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect();
    let joined = words.join("");
    joined.chars().take(40).collect()
}

fn file_name(path: &str) -> String {
    Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string())
}

fn csv_cell(s: &str) -> String {
    // Leading =,+,-,@ would be evaluated as a formula by spreadsheet apps.
    let s = if s.starts_with(['=', '+', '-', '@']) { format!("'{s}") } else { s.to_string() };
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const LEASE: &str = "COMMERCIAL LEASE AGREEMENT\nDocument Type: Lease Agreement\nVendor / Landlord: Acme Commercial Properties LLC\nClient / Tenant: Redstone Enterprises Inc.\nEffective Date: 2025-03-01\nExpiry Date: 2026-03-01\nHoldover penalty of $2,800 per day applies after expiry.\n";
    const NDA: &str = "MUTUAL NON-DISCLOSURE AGREEMENT\nThis Agreement is entered into between Axiom Legal LLP (\"Discloser\") and Meridian Capital Partners (\"Recipient\").\nThis Agreement is effective from July 15, 2025 and shall remain in effect until July 15, 2026. Liquidated damages of $50,000 per breach.\n";

    fn setup() -> (Vault, String, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("scan_001.txt"), LEASE).unwrap();
        fs::write(dir.path().join("doc 2.txt"), NDA).unwrap();
        fs::write(dir.path().join("photo.png"), [0u8; 10]).unwrap();
        fs::write(dir.path().join("notes.bin"), [1u8; 10]).unwrap();
        // No OCR engine, so the run does not depend on what is installed.
        let v = Vault::with_ocr(Store::open_in_memory().unwrap(), None);
        let m = v.store.create_matter(&MatterInput { name: "Redstone".into(), ..Default::default() }).unwrap();
        (v, m.id, dir)
    }

    /// Stands in for Tesseract: "reads" every scan as `text`.
    struct FakeOcr {
        text: &'static str,
        confidence: f64,
    }

    impl OcrEngine for FakeOcr {
        fn recognize(&self, _file: &Path, _ext: &str) -> std::result::Result<ocr::OcrText, String> {
            Ok(ocr::OcrText { text: self.text.to_string(), confidence: self.confidence, pages: 1 })
        }
    }

    fn ocr_vault(text: &'static str, confidence: f64) -> (Vault, String) {
        let v = Vault::with_ocr(Store::open_in_memory().unwrap(), Some(Box::new(FakeOcr { text, confidence })));
        let m = v.store.create_matter(&MatterInput { name: "Scans".into(), ..Default::default() }).unwrap();
        (v, m.id)
    }

    /// A scanned image is read, indexed and extracted like any document, and
    /// its fields carry the OCR confidence so a poor scan needs review.
    #[test]
    fn a_scanned_image_is_read_by_ocr() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("signed lease.png"), [0u8; 10]).unwrap();
        let (v, m) = ocr_vault(LEASE, 0.8);
        let r = v.ingest_paths(&m, &[dir.path().to_string_lossy().to_string()]).unwrap();
        assert_eq!((r.indexed, r.ocr, r.skipped), (1, 1, 0), "{r:?}");

        let row = v.store.review_grid(&m).unwrap().remove(0);
        assert_eq!(row.document.status, "indexed");
        assert_eq!(row.document.ocr_confidence, Some(0.8));
        let expiry = row.fields.iter().find(|f| f.key == extract::EXPIRY_DATE).unwrap();
        assert_eq!(expiry.value, "2026-03-01");
        assert!(expiry.confidence <= 0.8, "scaled by the OCR confidence: {}", expiry.confidence);
        assert_eq!(v.store.search(&m, "holdover penalty", 5).unwrap().len(), 1, "OCR text is searchable");
    }

    /// A PDF with no text layer is a scan, so it goes to OCR; a PDF with text
    /// never does.
    #[test]
    fn a_pdf_without_a_text_layer_falls_back_to_ocr() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("scanned.pdf"), blank_pdf()).unwrap();
        let (v, m) = ocr_vault(NDA, 0.9);
        let r = v.ingest_paths(&m, &[dir.path().to_string_lossy().to_string()]).unwrap();
        assert_eq!((r.indexed, r.ocr), (1, 1), "{r:?}");
        assert_eq!(v.store.list_documents(&m).unwrap()[0].ocr_confidence, Some(0.9));
    }

    #[test]
    fn a_scan_ocr_cannot_read_stays_listed_with_the_reason() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("blank.jpg"), [0u8; 10]).unwrap();
        let (v, m) = ocr_vault("  \n ", 0.1);
        let r = v.ingest_paths(&m, &[dir.path().to_string_lossy().to_string()]).unwrap();
        assert_eq!((r.indexed, r.skipped), (0, 1), "{r:?}");
        let doc = &v.store.list_documents(&m).unwrap()[0];
        assert_eq!(doc.status, "skipped");
        assert!(doc.error.as_deref().unwrap().contains("no readable text"));
    }

    /// The real engine on a real scan from the benchmark corpus. Runs where
    /// Tesseract is installed (CI installs it).
    #[test]
    fn tesseract_reads_a_benchmark_scan() {
        let Some(engine) = ocr::Tesseract::detect() else {
            eprintln!("tesseract not installed; skipping");
            return;
        };
        let scan = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../../benchmarks/corpus/scan_161.png");
        let out = engine.recognize(&scan, "png").unwrap();
        assert!(out.confidence > 0.5, "{}", out.confidence);
        let fields = extract::extract_all(&out.text);
        let get = |k: &str| fields.iter().find(|f| f.key == k).map(|f| f.value.clone());
        assert_eq!(get(extract::EFFECTIVE_DATE).as_deref(), Some("2025-03-14"), "{}", out.text);
        assert_eq!(get(extract::EXPIRY_DATE).as_deref(), Some("2026-11-27"), "{}", out.text);
        assert!(get(extract::PARTIES).unwrap_or_default().contains("Summit Audit Partners"), "{}", out.text);
    }

    /// A one-page PDF with an empty page: valid, but no text layer.
    fn blank_pdf() -> Vec<u8> {
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << >> >>",
        ];
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (i, body) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend(format!("{} 0 obj\n{body}\nendobj\n", i + 1).bytes());
        }
        let xref = pdf.len();
        pdf.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).bytes());
        for off in offsets {
            pdf.extend(format!("{off:010} 00000 n \n").bytes());
        }
        pdf.extend(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).bytes());
        pdf
    }

    /// The health report counts what was read and why the rest was not, and
    /// carries no document text.
    #[test]
    fn diagnostics_report_counts_and_reasons() {
        let (v, m, dir) = setup();
        v.ingest_paths(&m, &[dir.path().to_string_lossy().to_string()]).unwrap();
        let d = v.diagnostics().unwrap();
        assert_eq!((d.matters, d.documents, d.indexed, d.skipped), (1, 3, 2, 1));
        assert_eq!(d.unreadable, vec![ReasonCount { reason: ocr::NOT_INSTALLED.into(), count: 1 }]);
        assert!(d.passages >= 2 && d.embedded == 0);
        assert!(d.audit.ok && d.audit.entries > 0);
        assert!(d.database_bytes > 0);
        let json = serde_json::to_string(&d).unwrap();
        assert!(!json.contains("Holdover"), "no document text in the report");
    }

    #[test]
    fn ingest_index_and_reingest() {
        let (v, m, dir) = setup();
        let r = v.ingest_paths(&m, &[dir.path().to_string_lossy().to_string()]).unwrap();
        assert_eq!((r.scanned, r.indexed, r.skipped, r.failed), (3, 2, 1, 0), "{r:?}");
        let docs = v.store.list_documents(&m).unwrap();
        assert_eq!(docs.len(), 3);
        let png = docs.iter().find(|d| d.ext == "png").unwrap();
        assert_eq!(png.status, "skipped");
        assert!(png.error.as_deref().unwrap().contains("OCR"));

        let grid = v.store.review_grid(&m).unwrap();
        let lease = grid.iter().find(|r| r.document.file_name == "scan_001.txt").unwrap();
        let exp = lease.fields.iter().find(|f| f.key == "expiry_date").unwrap();
        assert_eq!(exp.value, "2026-03-01");
        assert_eq!(exp.chunk_ordinal, Some(0));

        let hits = v.store.search(&m, "liquidated damages", 5).unwrap();
        assert_eq!(hits[0].file_name, "doc 2.txt");

        let again = v.ingest_paths(&m, &[dir.path().to_string_lossy().to_string()]).unwrap();
        assert_eq!(again.unchanged, 3, "skipped files are not re-read either");
        assert_eq!(again.indexed, 0);
    }

    #[test]
    fn renames_require_approval_and_can_be_undone() {
        let (v, m, dir) = setup();
        v.ingest_paths(&m, &[dir.path().to_string_lossy().to_string()]).unwrap();
        let proposals = v.propose_renames(&m).unwrap();
        let lease = proposals.iter().find(|a| a.from_path.ends_with("scan_001.txt")).unwrap();
        assert!(lease.to_path.ends_with("Lease_AcmeCommercialProperties_2025-03-01.txt"), "{}", lease.to_path);
        assert_eq!(lease.risk, "low");
        let nda = proposals.iter().find(|a| a.from_path.ends_with("doc 2.txt")).unwrap();
        assert!(nda.to_path.ends_with("NDA_AxiomLegal_2025-07-15.txt"), "{}", nda.to_path);

        // Nothing runs without approval.
        assert!(v.execute_approved(&m).unwrap().is_empty());
        assert!(dir.path().join("scan_001.txt").exists());

        v.set_action_status(&lease.id, "approved").unwrap();
        let done = v.execute_approved(&m).unwrap();
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].status, "executed");
        assert!(!dir.path().join("scan_001.txt").exists());
        assert!(Path::new(&lease.to_path).exists());
        assert!(dir.path().join("doc 2.txt").exists(), "unapproved file untouched");
        let doc = v.store.get_document(&lease.document_id).unwrap();
        assert_eq!(doc.path, lease.to_path);

        assert!(v.set_action_status(&lease.id, "approved").is_err(), "executed actions are final");
        let undone = v.undo_action(&lease.id).unwrap();
        assert_eq!(undone.status, "undone");
        assert!(dir.path().join("scan_001.txt").exists());
        assert!(v.store.verify_audit().unwrap().ok);
    }

    #[test]
    fn target_collisions_get_suffixes() {
        let (v, m, dir) = setup();
        fs::write(dir.path().join("copy.txt"), LEASE.replace("Holdover", "Copy holdover")).unwrap();
        v.ingest_paths(&m, &[dir.path().to_string_lossy().to_string()]).unwrap();
        let targets: Vec<String> = v.propose_renames(&m).unwrap().into_iter().map(|a| file_name(&a.to_path)).collect();
        assert!(targets.contains(&"Lease_AcmeCommercialProperties_2025-03-01.txt".to_string()));
        assert!(targets.contains(&"Lease_AcmeCommercialProperties_2025-03-01_2.txt".to_string()));
    }

    #[test]
    fn local_only_matter_blocks_cloud_models() {
        let (v, m, _dir) = setup();
        let req = |loc: &str| ModelCallRequest {
            matter_id: m.clone(),
            provider: "anthropic".into(),
            model: "claude".into(),
            location: loc.into(),
            chars_sent: 1200,
            source_count: 3,
            purpose: "answer".into(),
        };
        assert!(!v.authorize_model_call(&req("cloud")).unwrap().allowed);
        assert!(v.authorize_model_call(&req("local")).unwrap().allowed);
        v.store
            .update_matter(&m, &MatterInput { name: String::new(), client: String::new(), kind: None, policy: Some("hybrid".into()) })
            .unwrap();
        assert!(v.authorize_model_call(&req("cloud")).unwrap().allowed);
        let kinds: Vec<String> = v.store.list_audit(Some(&m), 50).unwrap().into_iter().map(|a| a.kind).collect();
        assert!(kinds.contains(&"model_call_blocked".to_string()));
        assert!(v.authorize_model_call(&req("moon")).is_err());
    }

    #[test]
    fn csv_export_escapes_and_neutralizes_formulas() {
        assert_eq!(csv_cell("a,b"), "\"a,b\"");
        assert_eq!(csv_cell("=SUM(A1)"), "'=SUM(A1)");
        let (v, m, dir) = setup();
        v.ingest_paths(&m, &[dir.path().to_string_lossy().to_string()]).unwrap();
        let dest = dir.path().join("register.csv");
        assert_eq!(v.export_register_csv(&m, &dest).unwrap(), 2);
        let csv = fs::read_to_string(dest).unwrap();
        assert!(csv.contains("Lease,Acme Commercial Properties LLC · Redstone Enterprises Inc.,2025-03-01,2026-03-01"), "{csv}");
    }

    #[test]
    fn email_cleanup_keeps_headers_and_body() {
        let raw = "Content-Type: multipart/mixed; boundary=\"x\"\nFrom: Vexlar <a@b.com>\nSubject: Invoice\n\n--x\nContent-Type: text/plain\n\nAmount: $5.00\n--x--\n";
        let t = clean_email(raw);
        assert!(t.contains("From: Vexlar <a@b.com>"));
        assert!(t.contains("Amount: $5.00"));
        assert!(!t.contains("Content-Type"));
    }
}
