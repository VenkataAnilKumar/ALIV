//! SQLite persistence for the vault: matters, documents, passages (FTS5),
//! extracted fields, file actions and a hash-chained audit log.

use crate::error::{Result, VaultError};
use crate::model::*;
use crate::semantic;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS matters (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    client      TEXT NOT NULL DEFAULT '',
    kind        TEXT NOT NULL DEFAULT 'legal',
    policy      TEXT NOT NULL DEFAULT 'local_only',
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS documents (
    id          TEXT PRIMARY KEY,
    matter_id   TEXT NOT NULL REFERENCES matters(id) ON DELETE CASCADE,
    path        TEXT NOT NULL,
    file_name   TEXT NOT NULL,
    ext         TEXT NOT NULL DEFAULT '',
    size        INTEGER NOT NULL DEFAULT 0,
    sha256      TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL,
    error       TEXT,
    doc_type    TEXT,
    char_count  INTEGER NOT NULL DEFAULT 0,
    ingested_at TEXT NOT NULL,
    ocr_confidence REAL,
    summary     TEXT,
    UNIQUE (matter_id, path)
);

CREATE TABLE IF NOT EXISTS chunks (
    id          INTEGER PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    matter_id   TEXT NOT NULL,
    ordinal     INTEGER NOT NULL,
    text        TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS chunks_doc ON chunks(document_id);

CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
    text, content='chunks', content_rowid='id', tokenize='porter unicode61'
);
CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON chunks BEGIN
    INSERT INTO chunks_fts(rowid, text) VALUES (new.id, new.text);
END;
CREATE TRIGGER IF NOT EXISTS chunks_ad AFTER DELETE ON chunks BEGIN
    INSERT INTO chunks_fts(chunks_fts, rowid, text) VALUES ('delete', old.id, old.text);
END;

-- One embedding per passage, unit length, from the app's local embedder.
CREATE TABLE IF NOT EXISTS chunk_vectors (
    chunk_id    INTEGER PRIMARY KEY REFERENCES chunks(id) ON DELETE CASCADE,
    dim         INTEGER NOT NULL,
    vector      BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS fields (
    document_id   TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    key           TEXT NOT NULL,
    value         TEXT NOT NULL,
    confidence    REAL NOT NULL,
    chunk_ordinal INTEGER,
    snippet       TEXT NOT NULL,
    PRIMARY KEY (document_id, key)
);

CREATE TABLE IF NOT EXISTS actions (
    id          TEXT PRIMARY KEY,
    matter_id   TEXT NOT NULL REFERENCES matters(id) ON DELETE CASCADE,
    document_id TEXT NOT NULL,
    kind        TEXT NOT NULL,
    from_path   TEXT NOT NULL,
    to_path     TEXT NOT NULL,
    reason      TEXT NOT NULL,
    risk        TEXT NOT NULL,
    status      TEXT NOT NULL,
    error       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    matter_id TEXT,
    ts        TEXT NOT NULL,
    kind      TEXT NOT NULL,
    summary   TEXT NOT NULL,
    detail    TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    hash      TEXT NOT NULL
);
"#;

pub struct Store {
    conn: Mutex<Connection>,
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch(SCHEMA)?;
        migrate(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    // ------------------------------------------------------------ matters

    pub fn list_matters(&self) -> Result<Vec<Matter>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.name, m.client, m.kind, m.policy, m.created_at,
                    (SELECT COUNT(*) FROM documents d WHERE d.matter_id = m.id AND d.status = 'indexed'),
                    (SELECT COUNT(*) FROM actions a WHERE a.matter_id = m.id AND a.status IN ('proposed','approved'))
             FROM matters m ORDER BY m.created_at DESC",
        )?;
        let rows = stmt.query_map([], row_to_matter)?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    pub fn get_matter(&self, id: &str) -> Result<Matter> {
        self.list_matters()?
            .into_iter()
            .find(|m| m.id == id)
            .ok_or_else(|| VaultError::NotFound(format!("matter {id}")))
    }

    pub fn create_matter(&self, input: &MatterInput) -> Result<Matter> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(VaultError::Invalid("Matter name is required".into()));
        }
        let kind = validate(input.kind.as_deref().unwrap_or("legal"), &["legal", "finance"], "kind")?;
        let policy = validate(input.policy.as_deref().unwrap_or("local_only"), &["local_only", "hybrid"], "policy")?;
        let id = uuid::Uuid::new_v4().to_string();
        self.conn().execute(
            "INSERT INTO matters (id, name, client, kind, policy, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id, name, input.client.trim(), kind, policy, now()],
        )?;
        self.audit(Some(&id), "matter_created", &format!("Matter \"{name}\" created ({policy})"), serde_json::json!({ "kind": kind, "policy": policy }))?;
        self.get_matter(&id)
    }

    pub fn update_matter(&self, id: &str, input: &MatterInput) -> Result<Matter> {
        let before = self.get_matter(id)?;
        let name = if input.name.trim().is_empty() { before.name.clone() } else { input.name.trim().to_string() };
        let kind = validate(input.kind.as_deref().unwrap_or(&before.kind), &["legal", "finance"], "kind")?;
        let policy = validate(input.policy.as_deref().unwrap_or(&before.policy), &["local_only", "hybrid"], "policy")?;
        self.conn().execute(
            "UPDATE matters SET name=?2, client=?3, kind=?4, policy=?5 WHERE id=?1",
            params![id, name, input.client.trim(), kind, policy],
        )?;
        if policy != before.policy {
            self.audit(Some(id), "policy_changed", &format!("Model policy changed from {} to {policy}", before.policy), serde_json::json!({ "from": before.policy, "to": policy }))?;
        }
        self.get_matter(id)
    }

    pub fn delete_matter(&self, id: &str) -> Result<()> {
        let m = self.get_matter(id)?;
        self.conn().execute("DELETE FROM matters WHERE id=?1", params![id])?;
        // The audit row outlives the matter on purpose.
        self.audit(None, "matter_deleted", &format!("Matter \"{}\" deleted (source files untouched)", m.name), serde_json::json!({ "matterId": id }))?;
        Ok(())
    }

    // ---------------------------------------------------------- documents

    pub fn find_document_by_path(&self, matter_id: &str, path: &str) -> Result<Option<(String, String)>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT id, sha256 FROM documents WHERE matter_id=?1 AND path=?2",
                params![matter_id, path],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
    }

    /// Replaces a document and everything derived from it in one transaction.
    pub fn upsert_document(&self, doc: &Document, sha: &str, chunks: &[String], fields: &[Field]) -> Result<()> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM documents WHERE matter_id=?1 AND path=?2", params![doc.matter_id, doc.path])?;
        tx.execute(
            "INSERT INTO documents (id, matter_id, path, file_name, ext, size, sha256, status, error, doc_type, char_count, ingested_at, ocr_confidence, summary)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![doc.id, doc.matter_id, doc.path, doc.file_name, doc.ext, doc.size, sha, doc.status, doc.error, doc.doc_type, doc.char_count, doc.ingested_at, doc.ocr_confidence, doc.summary],
        )?;
        {
            let mut ins = tx.prepare("INSERT INTO chunks (document_id, matter_id, ordinal, text) VALUES (?1,?2,?3,?4)")?;
            for (i, text) in chunks.iter().enumerate() {
                ins.execute(params![doc.id, doc.matter_id, i as i64, text])?;
            }
            let mut ins = tx.prepare("INSERT INTO fields (document_id, key, value, confidence, chunk_ordinal, snippet) VALUES (?1,?2,?3,?4,?5,?6)")?;
            for f in fields {
                ins.execute(params![doc.id, f.key, f.value, f.confidence, f.chunk_ordinal, f.snippet])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_documents(&self, matter_id: &str) -> Result<Vec<Document>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(&format!("{DOC_SELECT} WHERE d.matter_id=?1 ORDER BY d.file_name COLLATE NOCASE"))?;
        let rows = stmt.query_map(params![matter_id], row_to_document)?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    pub fn get_document(&self, id: &str) -> Result<Document> {
        self.conn()
            .query_row(&format!("{DOC_SELECT} WHERE d.id=?1"), params![id], row_to_document)
            .optional()?
            .ok_or_else(|| VaultError::NotFound(format!("document {id}")))
    }

    pub fn set_document_path(&self, id: &str, path: &str, file_name: &str) -> Result<()> {
        self.conn().execute("UPDATE documents SET path=?2, file_name=?3 WHERE id=?1", params![id, path, file_name])?;
        Ok(())
    }

    pub fn fields_for(&self, document_id: &str) -> Result<Vec<Field>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT key, value, confidence, chunk_ordinal, snippet FROM fields WHERE document_id=?1")?;
        let rows = stmt.query_map(params![document_id], |r| {
            Ok(Field { key: r.get(0)?, value: r.get(1)?, confidence: r.get(2)?, chunk_ordinal: r.get(3)?, snippet: r.get(4)? })
        })?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    pub fn review_grid(&self, matter_id: &str) -> Result<Vec<GridRow>> {
        self.list_documents(matter_id)?
            .into_iter()
            .filter(|d| d.status == "indexed")
            .map(|d| Ok(GridRow { fields: self.fields_for(&d.id)?, document: d }))
            .collect()
    }

    // ------------------------------------------------------------- search

    /// Keyword search, fused with meaning-based search when `query_vector` is
    /// given and the matter has passages embedded at the same dimension.
    pub fn hybrid_search(&self, matter_id: &str, query: &str, query_vector: Option<&[f32]>, limit: usize) -> Result<Vec<SearchHit>> {
        let Some(qv) = query_vector.and_then(semantic::normalize) else {
            return self.search(matter_id, query, limit);
        };
        let pool = limit * semantic::CANDIDATES_PER_RESULT;
        let keyword = self.search(matter_id, query, pool)?;
        let meaning = self.nearest(matter_id, &qv, pool)?;
        if meaning.is_empty() {
            return Ok(keyword.into_iter().take(limit).collect());
        }
        let keyword_ids: Vec<i64> = keyword.iter().map(|h| h.chunk_id).collect();
        let meaning_ids: Vec<i64> = meaning.iter().map(|(id, _)| *id).collect();
        let mut hits = Vec::with_capacity(limit);
        for (id, score) in semantic::fuse(&keyword_ids, &meaning_ids).into_iter().take(limit) {
            let hit = match keyword.iter().find(|h| h.chunk_id == id) {
                Some(h) => SearchHit { score, ..h.clone() },
                // Found by meaning alone: no keyword snippet to show, so the
                // passage's opening stands in for one.
                None => SearchHit { score, ..self.passage_hit(id)? },
            };
            hits.push(hit);
        }
        Ok(hits)
    }

    /// The `limit` passages of a matter closest in meaning to `query`, a unit
    /// vector. Passages embedded at another dimension (a different model) are
    /// not comparable and are ignored.
    fn nearest(&self, matter_id: &str, query: &[f32], limit: usize) -> Result<Vec<(i64, f64)>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT v.chunk_id, v.vector FROM chunk_vectors v JOIN chunks c ON c.id = v.chunk_id
             WHERE c.matter_id = ?1 AND v.dim = ?2",
        )?;
        let mut scored: Vec<(i64, f64)> = stmt
            .query_map(params![matter_id, query.len() as i64], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?)))?
            .filter_map(|r| r.ok())
            .map(|(id, bytes)| (id, semantic::dot(query, &semantic::from_bytes(&bytes))))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        Ok(scored)
    }

    fn passage_hit(&self, chunk_id: i64) -> Result<SearchHit> {
        let conn = self.conn();
        Ok(conn.query_row(
            "SELECT c.id, c.document_id, d.file_name, d.path, c.ordinal, c.text
             FROM chunks c JOIN documents d ON d.id = c.document_id WHERE c.id = ?1",
            params![chunk_id],
            |r| {
                let text: String = r.get(5)?;
                Ok(SearchHit {
                    chunk_id: r.get(0)?,
                    document_id: r.get(1)?,
                    file_name: r.get(2)?,
                    path: r.get(3)?,
                    ordinal: r.get(4)?,
                    snippet: opening(&text, 200),
                    text,
                    score: 0.0,
                })
            },
        )?)
    }

    /// Passages of a matter that have no embedding yet, oldest first, for the
    /// app to embed in batches.
    pub fn pending_embeddings(&self, matter_id: &str, limit: usize) -> Result<Vec<PendingPassage>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT c.id, c.text FROM chunks c LEFT JOIN chunk_vectors v ON v.chunk_id = c.id
             WHERE c.matter_id = ?1 AND v.chunk_id IS NULL ORDER BY c.id LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![matter_id, limit as i64], |r| Ok(PendingPassage { chunk_id: r.get(0)?, text: r.get(1)? }))?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    /// Stores passage embeddings. Vectors are checked and normalized; one that
    /// is empty, non-finite or for an unknown passage is skipped. Returns how
    /// many were stored.
    pub fn store_embeddings(&self, embeddings: &[PassageEmbedding]) -> Result<usize> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        let mut stored = 0;
        {
            let mut ins = tx.prepare(
                "INSERT OR REPLACE INTO chunk_vectors (chunk_id, dim, vector)
                 SELECT ?1, ?2, ?3 WHERE EXISTS (SELECT 1 FROM chunks WHERE id = ?1)",
            )?;
            for e in embeddings {
                if let Some(v) = semantic::normalize(&e.vector) {
                    stored += ins.execute(params![e.chunk_id, v.len() as i64, semantic::to_bytes(&v)])?;
                }
            }
        }
        tx.commit()?;
        Ok(stored)
    }

    /// How much of a matter is embedded, so the app can say whether search
    /// covers meaning yet.
    pub fn embedding_status(&self, matter_id: &str) -> Result<EmbeddingStatus> {
        Ok(self.conn().query_row(
            "SELECT COUNT(*), COUNT(v.chunk_id) FROM chunks c LEFT JOIN chunk_vectors v ON v.chunk_id = c.id WHERE c.matter_id = ?1",
            params![matter_id],
            |r| Ok(EmbeddingStatus { passages: r.get(0)?, embedded: r.get(1)? }),
        )?)
    }

    pub fn search(&self, matter_id: &str, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let Some(fts) = fts_query(query) else { return Ok(vec![]) };
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT c.id, c.document_id, d.file_name, d.path, c.ordinal, c.text,
                    snippet(chunks_fts, 0, '«', '»', '…', 28), bm25(chunks_fts) AS rank
             FROM chunks_fts
             JOIN chunks c ON c.id = chunks_fts.rowid
             JOIN documents d ON d.id = c.document_id
             WHERE chunks_fts MATCH ?1 AND c.matter_id = ?2
             ORDER BY rank LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![fts, matter_id, limit as i64], |r| {
            Ok(SearchHit {
                chunk_id: r.get(0)?,
                document_id: r.get(1)?,
                file_name: r.get(2)?,
                path: r.get(3)?,
                ordinal: r.get(4)?,
                text: r.get(5)?,
                snippet: r.get(6)?,
                score: -r.get::<_, f64>(7)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    // ------------------------------------------------------------ actions

    pub fn insert_action(&self, a: &Action) -> Result<()> {
        self.conn().execute(
            "INSERT INTO actions (id, matter_id, document_id, kind, from_path, to_path, reason, risk, status, error, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![a.id, a.matter_id, a.document_id, a.kind, a.from_path, a.to_path, a.reason, a.risk, a.status, a.error, a.created_at, a.updated_at],
        )?;
        Ok(())
    }

    pub fn list_actions(&self, matter_id: &str) -> Result<Vec<Action>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(&format!("{ACTION_SELECT} WHERE matter_id=?1 ORDER BY created_at DESC, to_path"))?;
        let rows = stmt.query_map(params![matter_id], row_to_action)?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    pub fn get_action(&self, id: &str) -> Result<Action> {
        self.conn()
            .query_row(&format!("{ACTION_SELECT} WHERE id=?1"), params![id], row_to_action)
            .optional()?
            .ok_or_else(|| VaultError::NotFound(format!("action {id}")))
    }

    pub fn update_action_status(&self, id: &str, status: &str, error: Option<&str>) -> Result<Action> {
        self.conn().execute(
            "UPDATE actions SET status=?2, error=?3, updated_at=?4 WHERE id=?1",
            params![id, status, error, now()],
        )?;
        self.get_action(id)
    }

    /// Drops pending proposals so a fresh proposal run does not duplicate them.
    pub fn clear_open_proposals(&self, matter_id: &str) -> Result<()> {
        self.conn().execute("DELETE FROM actions WHERE matter_id=?1 AND status IN ('proposed','approved')", params![matter_id])?;
        Ok(())
    }

    // -------------------------------------------------------------- audit

    /// Appends an entry whose hash covers the previous entry's hash, so any
    /// edit or deletion in the middle of the log is detectable.
    pub fn audit(&self, matter_id: Option<&str>, kind: &str, summary: &str, detail: serde_json::Value) -> Result<i64> {
        let conn = self.conn();
        let prev: String = conn
            .query_row("SELECT hash FROM audit ORDER BY id DESC LIMIT 1", [], |r| r.get(0))
            .optional()?
            .unwrap_or_default();
        let ts = now();
        let detail = detail.to_string();
        let hash = chain_hash(&prev, matter_id.unwrap_or(""), &ts, kind, summary, &detail);
        conn.execute(
            "INSERT INTO audit (matter_id, ts, kind, summary, detail, prev_hash, hash) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![matter_id, ts, kind, summary, detail, prev, hash],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_audit(&self, matter_id: Option<&str>, limit: usize) -> Result<Vec<AuditEntry>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, matter_id, ts, kind, summary, detail, hash FROM audit
             WHERE ?1 IS NULL OR matter_id = ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![matter_id, limit as i64], |r| {
            let detail: String = r.get(5)?;
            Ok(AuditEntry {
                id: r.get(0)?,
                matter_id: r.get(1)?,
                ts: r.get(2)?,
                kind: r.get(3)?,
                summary: r.get(4)?,
                detail: serde_json::from_str(&detail).unwrap_or(serde_json::Value::Null),
                hash: r.get(6)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    /// Counts for [`Vault::diagnostics`](crate::Vault::diagnostics), filled
    /// in apart from the OCR tools and the audit check.
    pub fn diagnostics_counts(&self, audit: AuditVerification) -> Result<VaultDiagnostics> {
        let conn = self.conn();
        let count = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
        let database_path: Option<String> = conn
            .query_row("SELECT file FROM pragma_database_list WHERE name = 'main'", [], |r| r.get(0))
            .optional()?
            .filter(|p: &String| !p.is_empty());
        let mut unreadable = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT COALESCE(error, status), COUNT(*) FROM documents WHERE status != 'indexed'
                 GROUP BY 1 ORDER BY 2 DESC, 1 LIMIT 20",
            )?;
            for row in stmt.query_map([], |r| Ok(ReasonCount { reason: r.get(0)?, count: r.get(1)? }))? {
                unreadable.push(row?);
            }
        }
        Ok(VaultDiagnostics {
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: now(),
            database_path,
            database_bytes: count("SELECT page_count * page_size FROM pragma_page_count, pragma_page_size")?,
            matters: count("SELECT COUNT(*) FROM matters")?,
            documents: count("SELECT COUNT(*) FROM documents")?,
            indexed: count("SELECT COUNT(*) FROM documents WHERE status = 'indexed'")?,
            skipped: count("SELECT COUNT(*) FROM documents WHERE status = 'skipped'")?,
            failed: count("SELECT COUNT(*) FROM documents WHERE status = 'failed'")?,
            ocr_documents: count("SELECT COUNT(*) FROM documents WHERE ocr_confidence IS NOT NULL")?,
            passages: count("SELECT COUNT(*) FROM chunks")?,
            embedded: count("SELECT COUNT(*) FROM chunk_vectors")?,
            unreadable,
            tesseract: None,
            pdftoppm: None,
            audit,
        })
    }

    pub fn verify_audit(&self) -> Result<AuditVerification> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, matter_id, ts, kind, summary, detail, prev_hash, hash FROM audit ORDER BY id")?;
        let mut rows = stmt.query([])?;
        let mut prev = String::new();
        let mut entries = 0;
        while let Some(r) = rows.next()? {
            entries += 1;
            let id: i64 = r.get(0)?;
            let matter: Option<String> = r.get(1)?;
            let (ts, kind, summary, detail, prev_hash, hash): (String, String, String, String, String, String) =
                (r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?);
            let expected = chain_hash(&prev, matter.as_deref().unwrap_or(""), &ts, &kind, &summary, &detail);
            if prev_hash != prev || hash != expected {
                return Ok(AuditVerification { ok: false, entries, first_broken_id: Some(id) });
            }
            prev = hash;
        }
        Ok(AuditVerification { ok: true, entries, first_broken_id: None })
    }
}

fn chain_hash(prev: &str, matter: &str, ts: &str, kind: &str, summary: &str, detail: &str) -> String {
    let mut h = Sha256::new();
    for part in [prev, matter, ts, kind, summary, detail] {
        h.update(part.as_bytes());
        h.update([0u8]);
    }
    format!("{:x}", h.finalize())
}

fn validate<'a>(value: &'a str, allowed: &[&str], what: &str) -> Result<&'a str> {
    if allowed.contains(&value) {
        Ok(value)
    } else {
        Err(VaultError::Invalid(format!("unknown {what} \"{value}\"")))
    }
}

const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "do", "does", "for", "from", "has", "have", "how",
    "in", "is", "it", "its", "me", "of", "on", "or", "show", "that", "the", "their", "there", "this",
    "to", "was", "what", "when", "where", "which", "who", "will", "with", "all", "any", "find", "list",
    "tell", "give", "can", "you", "our", "we", "i", "my", "please", "about",
];

/// Turns a natural-language question into an FTS5 OR-query of quoted terms,
/// so user punctuation can never be read as FTS syntax.
pub fn fts_query(question: &str) -> Option<String> {
    let mut terms: Vec<String> = Vec::new();
    for raw in question.split(|c: char| !c.is_alphanumeric() && c != '\'') {
        let t = raw.trim_matches('\'').to_lowercase();
        if t.chars().count() < 2 || STOPWORDS.contains(&t.as_str()) || terms.contains(&t) {
            continue;
        }
        terms.push(t);
    }
    if terms.is_empty() {
        return None;
    }
    Some(terms.iter().map(|t| format!("\"{}\"", t.replace('"', ""))).collect::<Vec<_>>().join(" OR "))
}

/// Brings a vault created by an earlier version up to [`SCHEMA`]: `CREATE
/// TABLE IF NOT EXISTS` leaves an existing table's columns as they were.
fn migrate(conn: &Connection) -> Result<()> {
    let has_column = |table: &str, column: &str| -> Result<bool> {
        let mut stmt = conn.prepare(&format!("SELECT 1 FROM pragma_table_info('{table}') WHERE name=?1"))?;
        Ok(stmt.exists(params![column])?)
    };
    if !has_column("documents", "ocr_confidence")? {
        conn.execute("ALTER TABLE documents ADD COLUMN ocr_confidence REAL", [])?;
    }
    if !has_column("documents", "summary")? {
        conn.execute("ALTER TABLE documents ADD COLUMN summary TEXT", [])?;
    }
    Ok(())
}

/// The first `max` characters of `text`, cut at a word boundary.
fn opening(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        return flat;
    }
    let cut: String = flat.chars().take(max).collect();
    match cut.rfind(' ') {
        Some(i) => format!("{}…", &cut[..i]),
        None => format!("{cut}…"),
    }
}

const DOC_SELECT: &str = "SELECT d.id, d.matter_id, d.path, d.file_name, d.ext, d.size, d.status, d.error, d.doc_type, d.char_count,
    (SELECT COUNT(*) FROM chunks c WHERE c.document_id = d.id), d.ingested_at, d.ocr_confidence, d.summary FROM documents d";

const ACTION_SELECT: &str = "SELECT id, matter_id, document_id, kind, from_path, to_path, reason, risk, status, error, created_at, updated_at FROM actions";

fn row_to_matter(r: &rusqlite::Row) -> rusqlite::Result<Matter> {
    Ok(Matter {
        id: r.get(0)?,
        name: r.get(1)?,
        client: r.get(2)?,
        kind: r.get(3)?,
        policy: r.get(4)?,
        created_at: r.get(5)?,
        document_count: r.get(6)?,
        pending_actions: r.get(7)?,
    })
}

fn row_to_document(r: &rusqlite::Row) -> rusqlite::Result<Document> {
    Ok(Document {
        id: r.get(0)?,
        matter_id: r.get(1)?,
        path: r.get(2)?,
        file_name: r.get(3)?,
        ext: r.get(4)?,
        size: r.get(5)?,
        status: r.get(6)?,
        error: r.get(7)?,
        doc_type: r.get(8)?,
        char_count: r.get(9)?,
        chunk_count: r.get(10)?,
        ingested_at: r.get(11)?,
        ocr_confidence: r.get(12)?,
        summary: r.get(13)?,
    })
}

fn row_to_action(r: &rusqlite::Row) -> rusqlite::Result<Action> {
    Ok(Action {
        id: r.get(0)?,
        matter_id: r.get(1)?,
        document_id: r.get(2)?,
        kind: r.get(3)?,
        from_path: r.get(4)?,
        to_path: r.get(5)?,
        reason: r.get(6)?,
        risk: r.get(7)?,
        status: r.get(8)?,
        error: r.get(9)?,
        created_at: r.get(10)?,
        updated_at: r.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matter(store: &Store) -> Matter {
        store
            .create_matter(&MatterInput { name: "Acme lease review".into(), client: "Redstone".into(), ..Default::default() })
            .unwrap()
    }

    #[test]
    fn matter_crud_and_validation() {
        let s = Store::open_in_memory().unwrap();
        let m = matter(&s);
        assert_eq!(m.policy, "local_only");
        assert_eq!(m.kind, "legal");
        assert!(s.create_matter(&MatterInput { name: "  ".into(), ..Default::default() }).is_err());
        assert!(s
            .create_matter(&MatterInput { name: "x".into(), policy: Some("anything".into()), ..Default::default() })
            .is_err());
        let m2 = s
            .update_matter(&m.id, &MatterInput { name: String::new(), client: "Redstone".into(), kind: None, policy: Some("hybrid".into()) })
            .unwrap();
        assert_eq!(m2.name, "Acme lease review");
        assert_eq!(m2.policy, "hybrid");
        s.delete_matter(&m.id).unwrap();
        assert!(s.list_matters().unwrap().is_empty());
    }

    #[test]
    fn fts_query_is_safe() {
        assert_eq!(fts_query("What is the penalty?").unwrap(), "\"penalty\"");
        assert_eq!(fts_query("\"); DROP TABLE x; --").unwrap(), "\"drop\" OR \"table\"");
        assert!(fts_query("?? the of").is_none());
    }

    /// A vault written before OCR existed opens, gains the column, and its
    /// documents read back as not OCR'd.
    #[test]
    fn a_vault_from_before_ocr_is_migrated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(&SCHEMA.replace("    ocr_confidence REAL,\n", "").replace("    summary     TEXT,\n", "")).unwrap();
            conn.execute("INSERT INTO matters (id, name, created_at) VALUES ('m', 'Old', '2025-01-01')", []).unwrap();
            conn.execute(
                "INSERT INTO documents (id, matter_id, path, file_name, status, ingested_at) VALUES ('d', 'm', '/x/a.txt', 'a.txt', 'indexed', '2025-01-01')",
                [],
            )
            .unwrap();
        }
        let s = Store::open(&path).unwrap();
        let d = s.get_document("d").unwrap();
        assert_eq!((d.ocr_confidence, d.summary), (None, None));
        drop(s);
        Store::open(&path).expect("migrating twice is a no-op");
    }

    /// A passage with no shared words is still found by meaning, a passage in
    /// both rankings comes first, and another matter's vectors never leak in.
    #[test]
    fn hybrid_search_fuses_keyword_and_meaning() {
        let s = Store::open_in_memory().unwrap();
        let (a, b) = (matter(&s), matter(&s));
        let doc = |m: &str, name: &str| Document {
            id: uuid::Uuid::new_v4().to_string(),
            matter_id: m.into(),
            path: format!("/x/{name}"),
            file_name: name.into(),
            ext: "txt".into(),
            size: 1,
            status: "indexed".into(),
            error: None,
            doc_type: None,
            char_count: 10,
            chunk_count: 0,
            ingested_at: now(),
            ocr_confidence: None,
            summary: None,
        };
        s.upsert_document(
            &doc(&a.id, "msa.txt"),
            "h",
            &[
                "Termination for convenience: either party may end this agreement on 30 days notice.".into(),
                "Fees are payable monthly in arrears.".into(),
                "The agreement may be terminated early for material breach.".into(),
            ],
            &[],
        )
        .unwrap();
        s.upsert_document(&doc(&b.id, "other.txt"), "h", &["Unrelated matter text.".into()], &[]).unwrap();

        // Toy 3-d "embeddings": axis 0 = ending the contract, 1 = money.
        let vectors = [[0.9f32, 0.1, 0.0], [0.0, 1.0, 0.0], [0.8, 0.0, 0.2]];
        let pending = s.pending_embeddings(&a.id, 10).unwrap();
        assert_eq!(pending.len(), 3);
        let embeddings: Vec<PassageEmbedding> =
            pending.iter().zip(vectors).map(|(p, v)| PassageEmbedding { chunk_id: p.chunk_id, vector: v.to_vec() }).collect();
        assert_eq!(s.store_embeddings(&embeddings).unwrap(), 3);
        let other = s.pending_embeddings(&b.id, 10).unwrap();
        s.store_embeddings(&[PassageEmbedding { chunk_id: other[0].chunk_id, vector: vec![1.0, 0.0, 0.0] }]).unwrap();
        assert_eq!(s.embedding_status(&a.id).unwrap(), EmbeddingStatus { passages: 3, embedded: 3 });
        assert!(s.pending_embeddings(&a.id, 10).unwrap().is_empty());

        // "How do we exit?" shares no keyword with the termination clauses;
        // only meaning finds them.
        let question = [1.0f32, 0.0, 0.0];
        assert!(s.search(&a.id, "how do we exit", 5).unwrap().is_empty());
        let hits = s.hybrid_search(&a.id, "how do we exit", Some(&question), 2).unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits[0].text.starts_with("Termination for convenience"), "{hits:?}");
        assert!(!hits[0].snippet.is_empty());
        assert!(hits.iter().all(|h| h.file_name == "msa.txt"), "other matters stay out");

        // Matching both ways ranks first: "terminated" by keyword and meaning.
        let hits = s.hybrid_search(&a.id, "terminated", Some(&question), 3).unwrap();
        assert!(hits[0].text.contains("terminated early"), "{hits:?}");

        // A vector from a model of another size is ignored, not compared.
        let hits = s.hybrid_search(&a.id, "fees", Some(&[1.0, 0.0]), 3).unwrap();
        assert_eq!(hits.len(), 1, "keyword-only: {hits:?}");

        // Re-ingesting replaces the passages, so their vectors go with them.
        s.upsert_document(&doc(&a.id, "msa.txt"), "h2", &["New text.".into()], &[]).unwrap();
        assert_eq!(s.embedding_status(&a.id).unwrap(), EmbeddingStatus { passages: 1, embedded: 0 });
    }

    #[test]
    fn bad_embeddings_are_skipped() {
        let s = Store::open_in_memory().unwrap();
        assert_eq!(
            s.store_embeddings(&[
                PassageEmbedding { chunk_id: 999, vector: vec![1.0] },
                PassageEmbedding { chunk_id: 1, vector: vec![] },
            ])
            .unwrap(),
            0
        );
    }

    #[test]
    fn search_ranks_and_scopes_by_matter() {
        let s = Store::open_in_memory().unwrap();
        let a = matter(&s);
        let b = matter(&s);
        let doc = |m: &str, name: &str| Document {
            id: uuid::Uuid::new_v4().to_string(),
            matter_id: m.into(),
            path: format!("/x/{name}"),
            file_name: name.into(),
            ext: "txt".into(),
            size: 1,
            status: "indexed".into(),
            error: None,
            doc_type: None,
            char_count: 10,
            chunk_count: 0,
            ingested_at: now(),
            ocr_confidence: None,
            summary: None,
        };
        s.upsert_document(&doc(&a.id, "lease.txt"), "h", &["Holdover penalty of $2,800 per day.".into(), "Rent is due monthly.".into()], &[]).unwrap();
        s.upsert_document(&doc(&b.id, "other.txt"), "h", &["Penalty clause in another matter.".into()], &[]).unwrap();
        let hits = s.search(&a.id, "holdover penalty?", 5).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].file_name, "lease.txt");
        assert!(hits[0].snippet.contains("«"));
        // Re-ingesting the same path replaces passages instead of duplicating them.
        s.upsert_document(&doc(&a.id, "lease.txt"), "h2", &["Holdover penalty updated.".into()], &[]).unwrap();
        assert_eq!(s.search(&a.id, "holdover", 5).unwrap().len(), 1);
        assert_eq!(s.list_documents(&a.id).unwrap()[0].chunk_count, 1);
    }

    #[test]
    fn audit_chain_detects_tampering() {
        let s = Store::open_in_memory().unwrap();
        let m = matter(&s);
        s.audit(Some(&m.id), "test", "one", serde_json::json!({})).unwrap();
        s.audit(Some(&m.id), "test", "two", serde_json::json!({"n": 2})).unwrap();
        let v = s.verify_audit().unwrap();
        assert!(v.ok);
        assert_eq!(v.entries, 3);
        s.conn().execute("UPDATE audit SET summary='edited' WHERE summary='one'", []).unwrap();
        let v = s.verify_audit().unwrap();
        assert!(!v.ok);
        assert_eq!(v.first_broken_id, Some(2));
    }
}
