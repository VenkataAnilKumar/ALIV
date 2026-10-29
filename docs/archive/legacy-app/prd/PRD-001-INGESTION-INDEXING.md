# PRD-001 Ingestion and Indexing

## 1. Goal
Create a reliable local ingestion pipeline that parses, chunks, and indexes mixed document types into SQLite and sqlite-vec.

## 2. User Story
As a user, I can point ALIV to local folders and trust that documents are indexed with searchable metadata and embeddings.

## 3. Functional Requirements
1. Folder registration for one or more local paths.
2. File discovery and change detection.
3. Supported file parsing:
- PDF (native and scanned via OCR pipeline)
- DOCX
- Email exports (eml/msg where possible)
- Image files used for scanned docs
4. Text extraction and chunking with deterministic chunk IDs.
5. Metadata extraction (path, type, modified time, hash, inferred doc type).
6. Embedding generation and persistence in sqlite-vec.
7. Re-index behavior for modified files.
8. Skip unchanged files using hash/version checks.

## 4. Non-Functional Requirements
1. WAL mode enabled for SQLite.
2. Ingestion throughput and failure counts logged.
3. Resume from partial failures without full restart.
4. Corrupt file handling without pipeline crash.

## 5. Acceptance Criteria
1. 200-file corpus indexing completes without fatal crash.
2. Ingestion success rate >= 98%.
3. Re-run on unchanged corpus performs incremental no-op for at least 95% of files.
4. Error report generated with file path and failure reason.
5. Every run records parser, OCR, and embedding model versions for reproducibility.

## 6. Telemetry
1. Files discovered, indexed, skipped, failed.
2. Parse time per document and chunk count per file.
3. Embedding generation time and DB write latency.

## 7. Test Cases
1. Corrupt PDF.
2. Password-protected PDF.
3. Duplicate filenames in different folders.
4. Large document (>200 pages).
5. Scanned image quality extremes.
6. Zero-byte files and unsupported extensions.

## 8. Dependencies
1. Python parsing scripts and OCR stack.
2. Embedding model availability.
3. SQLite schema and migration scripts.

## 9. Done Definition
PRD-001 is done when ingestion thresholds pass, failure taxonomy is documented, and indexing can be re-run deterministically.
