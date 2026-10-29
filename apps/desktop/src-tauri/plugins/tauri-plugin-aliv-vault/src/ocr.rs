//! OCR for scanned documents: images, and PDFs whose pages have no text layer.
//!
//! Runs the Tesseract command-line tool, and `pdftoppm` (Poppler) to render
//! PDF pages to images first. Both are found on PATH or through
//! `ALIV_TESSERACT` / `ALIV_PDFTOPPM`; without Tesseract, scans stay listed as
//! skipped with a reason that says what to install.
//!
//! Tesseract's TSV output is used rather than plain text so the result keeps
//! each line together (a table row "Vendor  Acme LLC" stays one line) and
//! carries a per-word confidence, which is averaged into one score for the
//! document.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Pages rendered and read per PDF. Bounds the time one scanned file can take.
pub const MAX_PDF_PAGES: usize = 50;
/// Render resolution for PDF pages; Tesseract is tuned for about 300 dpi.
const PDF_DPI: &str = "300";
/// Images narrower than this (an A4 page scanned below about 200 dpi) are
/// enlarged before OCR. At screen resolution Tesseract misreads digits
/// ("2025-1223", "2027-03-48"); the same page enlarged twice reads cleanly.
const MIN_OCR_WIDTH: u32 = 1600;
const MAX_UPSCALE: u32 = 4;

/// Text read from a scanned document.
#[derive(Debug, Clone, PartialEq)]
pub struct OcrText {
    pub text: String,
    /// Mean word confidence, 0.0 to 1.0.
    pub confidence: f64,
    pub pages: usize,
}

/// Reads text from a scanned file. A trait so tests can run the ingestion
/// pipeline without Tesseract installed.
pub trait OcrEngine: Send + Sync {
    fn recognize(&self, file: &Path, ext: &str) -> Result<OcrText, String>;
}

/// [`OcrEngine`] backed by the `tesseract` and `pdftoppm` command-line tools.
#[derive(Debug, Clone)]
pub struct Tesseract {
    tesseract: PathBuf,
    pdftoppm: Option<PathBuf>,
}

impl Tesseract {
    /// Finds Tesseract (and, for PDFs, pdftoppm). `None` when Tesseract is
    /// not installed.
    pub fn detect() -> Option<Self> {
        let tesseract = locate("ALIV_TESSERACT", "tesseract")?;
        Some(Self { tesseract, pdftoppm: locate("ALIV_PDFTOPPM", "pdftoppm") })
    }

    fn read_image(&self, image: &Path) -> Result<(String, Vec<f64>), String> {
        // `--psm 6` reads the page as one block of text, which keeps a
        // label and its value on the same line; the default page
        // segmentation splits two-column layouts into separate columns.
        let out = Command::new(&self.tesseract)
            .arg(image)
            .args(["stdout", "--psm", "6", "tsv"])
            .output()
            .map_err(|e| format!("could not run tesseract: {e}"))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!("tesseract failed: {}", err.lines().last().unwrap_or("").trim()));
        }
        Ok(parse_tsv(&String::from_utf8_lossy(&out.stdout)))
    }

    /// Reads a scanned image, enlarging it first when its resolution is low.
    /// An image the decoder cannot open (HEIC, say) goes to Tesseract as is.
    fn read_scan_image(&self, image: &Path) -> Result<(String, Vec<f64>), String> {
        let scratch = Scratch::new()?;
        let source = match upscaled(image, &scratch.0) {
            Ok(Some(enlarged)) => enlarged,
            Ok(None) => image.to_path_buf(),
            Err(e) => {
                log::debug!("{}: not enlarged before OCR: {e}", image.display());
                image.to_path_buf()
            }
        };
        self.read_image(&source)
    }

    fn read_pdf(&self, pdf: &Path) -> Result<(String, Vec<f64>, usize), String> {
        let pdftoppm = self
            .pdftoppm
            .as_ref()
            .ok_or("scanned PDF: reading it needs pdftoppm (Poppler), which was not found")?;
        let scratch = Scratch::new()?;
        let dir = &scratch.0;
        let status = Command::new(pdftoppm)
            .args(["-r", PDF_DPI, "-gray", "-png", "-l", &MAX_PDF_PAGES.to_string()])
            .arg(pdf)
            .arg(dir.join("page"))
            .status()
            .map_err(|e| format!("could not run pdftoppm: {e}"))?;
        if !status.success() {
            return Err("pdftoppm could not render the PDF".to_string());
        }
        let mut pages: Vec<PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|e| e == "png"))
            .collect();
        // pdftoppm zero-pads page numbers to a common width, so a plain
        // sort is page order.
        pages.sort();
        let mut text = String::new();
        let mut confidences = Vec::new();
        for page in &pages {
            let (page_text, page_conf) = self.read_image(page)?;
            if !text.is_empty() {
                text.push_str("\n\n");
            }
            text.push_str(&page_text);
            confidences.extend(page_conf);
        }
        Ok((text, confidences, pages.len()))
    }
}

/// A temporary directory removed when dropped, so a failed OCR run leaves
/// no page images behind.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Result<Self, String> {
        let dir = std::env::temp_dir().join(format!("aliv-ocr-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(Self(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A grayscale copy of `image` enlarged to at least [`MIN_OCR_WIDTH`], written
/// into `dir`; `None` when the image is already wide enough.
fn upscaled(image: &Path, dir: &Path) -> Result<Option<PathBuf>, String> {
    let img = image::ImageReader::open(image)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    let factor = MIN_OCR_WIDTH.div_ceil(img.width().max(1)).min(MAX_UPSCALE);
    if factor <= 1 {
        return Ok(None);
    }
    let out = dir.join("enlarged.png");
    img.grayscale()
        .resize(img.width() * factor, img.height() * factor, image::imageops::FilterType::Lanczos3)
        .save(&out)
        .map_err(|e| e.to_string())?;
    Ok(Some(out))
}

impl OcrEngine for Tesseract {
    fn recognize(&self, file: &Path, ext: &str) -> Result<OcrText, String> {
        let (raw, confidences, pages) = if ext == "pdf" {
            self.read_pdf(file)?
        } else {
            let (text, conf) = self.read_scan_image(file)?;
            (text, conf, 1)
        };
        let confidence = if confidences.is_empty() {
            0.0
        } else {
            confidences.iter().sum::<f64>() / confidences.len() as f64 / 100.0
        };
        Ok(OcrText { text: repair_labels(&raw), confidence, pages })
    }
}

/// The reason a scan is skipped when no OCR engine is available.
pub const NOT_INSTALLED: &str = "scanned document: install Tesseract OCR to read it";

/// `env` names an explicit path; otherwise `name` is looked up on PATH and in
/// the usual install locations, which a GUI app launched from the macOS Dock
/// or Windows Start menu does not have on its PATH.
fn locate(env: &str, name: &str) -> Option<PathBuf> {
    if let Some(p) = std::env::var_os(env).map(PathBuf::from) {
        return p.is_file().then_some(p);
    }
    let exe = if cfg!(windows) { format!("{name}.exe") } else { name.to_string() };
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    dirs.extend(
        ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", r"C:\Program Files\Tesseract-OCR"]
            .iter()
            .map(PathBuf::from),
    );
    dirs.into_iter().map(|d| d.join(&exe)).find(|p| p.is_file())
}

/// Rebuilds text from Tesseract TSV: words joined into their lines, a blank
/// line between blocks. Returns the text and each word's confidence (0-100).
pub fn parse_tsv(tsv: &str) -> (String, Vec<f64>) {
    let mut text = String::new();
    let mut confidences = Vec::new();
    let mut current: Option<(&str, &str, &str, &str)> = None;
    for row in tsv.lines().skip(1) {
        let cols: Vec<&str> = row.split('\t').collect();
        // level, page, block, par, line, word, left, top, width, height, conf, text
        if cols.len() < 12 || cols[0] != "5" {
            continue;
        }
        let word = cols[11].trim();
        if word.is_empty() {
            continue;
        }
        let key = (cols[1], cols[2], cols[3], cols[4]);
        match current {
            Some(prev) if prev == key => text.push(' '),
            Some(prev) if (prev.0, prev.1) == (key.0, key.1) => text.push('\n'),
            Some(_) => text.push_str("\n\n"),
            None => {}
        }
        current = Some(key);
        text.push_str(word);
        if let Ok(conf) = cols[10].parse::<f64>() {
            if conf >= 0.0 {
                confidences.push(conf);
            }
        }
    }
    (text, confidences)
}

/// Undoes the two misreads OCR makes most on a document's header fields: a
/// stray quote mark in front of a value or label, and a dropped or misread
/// colon after a field label ("Vendor ‘Acme LLC", "Date’ 2025-03-14"). The
/// extractor reads "Label: value" lines, so these become "Vendor: Acme LLC".
///
/// Only a line that starts with a known field label and continues with
/// something value-like (a capital, a digit or a currency sign) is changed,
/// so prose such as "Date of signature ..." is left alone.
pub fn repair_labels(text: &str) -> String {
    const LABELS: &str = "invoice date|issue date|effective date|start date|commencement date|expiry date|expiration date|end date|due date|termination date|date|expiry|expires|amount due|total due|amount|total|vendor|client|landlord|tenant|lessor|lessee|supplier|customer|buyer|seller|bill to|billed to";
    static LABEL: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let label = LABEL.get_or_init(|| {
        regex::Regex::new(&format!(r#"^(?i:({LABELS}))\s*[:;,.'’‘`"]*\s+[‘’'`"|]*([A-Z0-9$£€].*)$"#))
            .expect("valid regex")
    });
    let mut out = String::with_capacity(text.len());
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let trimmed = line.trim_matches(|c: char| "‘’'`\"|".contains(c) || c.is_whitespace());
        match label.captures(trimmed) {
            Some(c) => {
                out.push_str(&c[1]);
                out.push_str(": ");
                out.push_str(c[2].trim_end());
            }
            None => out.push_str(trimmed),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const TSV: &str = "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
1\t1\t0\t0\t0\t0\t0\t0\t794\t1123\t-1\t
2\t1\t1\t0\t0\t0\t60\t102\t41\t97\t-1\t
5\t1\t1\t1\t1\t1\t60\t102\t80\t20\t96.5\tScanned
5\t1\t1\t1\t1\t2\t150\t102\t80\t20\t95.5\tContract
5\t1\t1\t1\t2\t1\t60\t130\t80\t20\t91\tVendor
5\t1\t1\t1\t2\t2\t150\t130\t80\t20\t62\t‘Summit
5\t1\t1\t1\t2\t3\t240\t130\t80\t20\t90\tAudit
5\t1\t2\t1\t1\t1\t60\t300\t80\t20\t85\tAmount
5\t1\t2\t1\t1\t2\t150\t300\t80\t20\t-1\t
";

    #[test]
    fn tsv_keeps_lines_and_separates_blocks() {
        let (text, conf) = parse_tsv(TSV);
        assert_eq!(text, "Scanned Contract\nVendor ‘Summit Audit\n\nAmount");
        assert_eq!(conf, vec![96.5, 95.5, 91.0, 62.0, 90.0, 85.0], "blank words and -1 are not scored");
    }

    #[test]
    fn misread_colons_and_stray_quotes_are_repaired() {
        let raw = "Scanned Contract #161\nVendor ‘Summit Audit Partners\nClient GlobalTech Inc.\nDate’ 2025-03-14\nExpiry 2026-11-27\n‘Amount $94,932.00";
        assert_eq!(
            repair_labels(raw),
            "Scanned Contract #161\nVendor: Summit Audit Partners\nClient: GlobalTech Inc.\nDate: 2025-03-14\nExpiry: 2026-11-27\nAmount: $94,932.00"
        );
    }

    #[test]
    fn prose_and_labelled_lines_are_left_alone() {
        for line in [
            "Date of signature to be agreed by the parties.",
            "Client shall pay within thirty days.",
            "Vendor: Acme LLC",
            "Total due is payable on receipt.",
        ] {
            assert_eq!(repair_labels(line), line);
        }
    }

    #[test]
    fn low_resolution_scans_are_enlarged_and_large_ones_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let small = dir.path().join("small.png");
        image::RgbImage::new(400, 560).save(&small).unwrap();
        let out = upscaled(&small, dir.path()).unwrap().expect("enlarged");
        let (w, h) = image::image_dimensions(&out).unwrap();
        assert_eq!((w, h), (1600, 2240), "4x, aspect kept");

        let large = dir.path().join("large.png");
        image::RgbImage::new(2480, 3508).save(&large).unwrap();
        assert_eq!(upscaled(&large, dir.path()).unwrap(), None);

        let tiny = dir.path().join("tiny.png");
        image::RgbImage::new(100, 100).save(&tiny).unwrap();
        let out = upscaled(&tiny, dir.path()).unwrap().unwrap();
        assert_eq!(image::image_dimensions(&out).unwrap().0, 400, "capped at {MAX_UPSCALE}x");
    }

    #[test]
    fn a_missing_tool_is_none_not_a_panic() {
        assert!(locate("ALIV_TEST_NO_SUCH_TOOL", "aliv-no-such-tool-xyz").is_none());
    }
}
