# Autonomous Local Intelligence Vault (ALIV)
**Product Requirements & Architecture Document**

## 1. Product Vision & Idea
**Concept:** A hyper-secure, offline-capable application that transforms a static, chaotic folder of local PDFs, Word documents, and emails into an autonomous knowledge worker—without ever uploading proprietary data to cloud vector databases.

### The Core Problem
Legal, finance, and accounting professionals handle thousands of highly sensitive documents (NDAs, invoices, tax records) locally. Existing AI solutions require uploading these documents to the cloud (OpenAI, Claude, etc.), which violates NDAs, compliance policies (SOC2/HIPAA), and general data privacy standards. Additionally, local file systems become unmanageable quickly ("the messy inbox problem").

### The Solution
ALIV is a completely local, headless execution agent bundled with a sleek control-center UI. It continuously monitors designated local folders, semantically maps all documents using local AI models, and provides the user with an intelligent agent capable of querying the data, synthesising reports to Excel, and autonomously managing the file system natively.

---

## 2. Core Workflows

### Flow A: The Legal/Finance Agent (Intelligent Retrieval & Synthesis)
**Trigger:** The user opens the Control Center and inputs a prompt: *"Find all agreements expiring in 90 days and extract the penalty clauses."*
1. **Understanding:** The agent parses the request and translates it into an SQL query combined with a semantic vector search.
2. **Retrieval:** It queries the local embedded SQLite database for contracts with an upcoming expiration date metadata tag, and cross-references those documents against chunks matching "penalty clause".
3. **Execution:** The embedded local LLM synthesizes the extracted clauses from the documents.
4. **Action:** The system spawns a background process to format the structured data and writes an Excel summary file directly to the user's Desktop.

### Flow B: The Messy Inbox Organizer (Autonomous File System Actions)
**Trigger:** The system detects 500 new chaotic files (e.g., `scan_123.pdf`, `img_099.jpg`) dropped into the `Downloads` folder.
1. **Cartography (Scanning):** The background Rust agent detects the new files instantly.
2. **Attunement:** It streams the files through local OCR/Text extraction and sends the text to the local LLM.
3. **Reasoning:** The LLM is strictly prompted to return a standardized naming convention: `{DocumentType}_{VendorName}_{Date}` (e.g., `Invoice_Apple_Jan2025.pdf`).
4. **Action execution:** The agent natively renames the files in the OS and moves them into structurally organized sub-folders, handling collision logic gracefully.

---

## 2.1 Local AI Model Specifications

### Synthesis / Reasoning LLM
| Property | Specification |
|---|---|
| Provider | Ollama (local inference server) |
| Default model | `llama3.2:3b` (3B params, quantized Q4_K_M) |
| Fallback model | `mistral:7b-instruct-q4_K_M` |
| Context window | 8,192 tokens minimum |
| Inference mode | HTTP REST via `http://localhost:11434` |
| Startup check | ALIV verifies Ollama is running before any LLM call |
| Timeout | 60 seconds per inference request |

### Embedding Model
| Property | Specification |
|---|---|
| Provider | sentence-transformers (Python subprocess) |
| Default model | `all-MiniLM-L6-v2` |
| Vector dimensions | 384 |
| Max input tokens | 512 tokens per chunk |
| Batch size | 32 chunks per embedding call |
| Model cache path | `~/.aliv/models/embeddings/` |

### OCR Engine
| Property | Specification |
|---|---|
| Engine | Tesseract OCR 5.x |
| Triggered when | PDF has no extractable text layer |
| Languages | `eng` (default), user-configurable |
| Minimum confidence | 60 (below this, chunk is flagged as low-confidence) |

### Minimum Hardware Requirements
| Component | Minimum | Recommended |
|---|---|---|
| CPU | 4-core x86_64 / ARM64 | 8-core |
| RAM | 8 GB | 16 GB |
| Disk (corpus + DB) | 10 GB free | 50 GB free |
| GPU | None required | NVIDIA CUDA 12+ for faster embeddings |
| OS | Windows 10+, macOS 12+, Ubuntu 20.04+ | — |

### Model Version Pinning
- Model name and version are stored in the `benchmark_runs` table alongside every run.
- Changing the embedding model requires re-indexing the full corpus.
- LLM version changes require re-running the full benchmark suite before phase gate.

---

## 3. Technology Stack & Architecture

We are utilizing a **Hybrid Rust/Tauri 2 + Python Orchestrator** pattern. 
* **Why?** It blends the absolute performance, low-memory footprint, and native OS capabilities of Rust with the unmatched Machine Learning document-parsing ecosystem of Python.

### Stack Breakdown
* **Frontend:** React, TypeScript, Vite
  * *Purpose:* Provides a Sleek Desktop UI (glassmorphism dashboards, node/graph relationships, real-time command line `stdout` streaming).
* **Backend Shell:** Rust & Tauri 2
  * *Purpose:* The brains of the operation. Extremely low-memory, fearless concurrency for background file watching, deep native OS integration.
* **Database & State:** SQLite in WAL (Write-Ahead Logging) mode + `sqlite-vec`
  * *Purpose:* Stores history, extracted text, metadata, and relationship embeddings securely on a single disk file without lock contention between background threads and the UI.
* **Execution & AI Subsystems:** Embedded Python / PTY Subprocesses
  * *Purpose:* Rust commands headless Python scripts or ONNX runtimes via standard input/output when heavy parsing (PDF scraping using `PyMuPDF`, Pandas Excel manipulation) or embedding generation is required.

---

## 4. The Core Paradigms

1. **Cartography (Scanning):** Rust's blazing-fast background threads (via libraries like `walkdir`) continuously index designated folders, scraping new files without disrupting the user's primary CPU thread.
2. **Attunement (Context Assembly):** Rust intelligently pulls the necessary constraints, historical data, and relevant document vectors from SQLite to build perfect, token-optimized prompts before contacting the local LLM reasoning engine.
3. **Process Execution:** True autonomous capability. Rust utilizes `std::process::Command` to dynamically spawn tests, invoke local Python parser scripts, trigger `ffmpeg`, or execute `kubectl` commands based on the agent's decision logic.
4. **Output Parsing Control Loop:** The system captures the `stdout`/`stderr` of all spawned background processes in real-time. If a Python extraction process fails, Rust captures the error, pipes it back to the agent for "self-correction", and dynamically streams the logs to the React UI for the user to monitor.

---

## 5. Product-Only Validation Track (No Buyer Dependency)

This track validates ALIV as a working product before customer development.

Authoritative thresholds and validation formulas are defined in `docs/prd/PRD-000-MASTER.md`.

### Validation Objective
Prove the core loop is reliable and useful on local data:
1. Ingest and index documents.
2. Retrieve relevant evidence.
3. Extract structured facts.
4. Execute safe file actions with rollback.

### Week 1 Validation Dataset
1. Build a 200-file local corpus with mixed document types.
2. Create 50 benchmark prompts and expected outputs.
3. Label critical fields for extraction scoring (expiry date, vendor, penalty clause).

### Product Pass Thresholds
1. Ingestion success rate: 98 percent or higher.
2. Retrieval relevance at top-5: 85 percent or higher.
3. Extraction precision on critical fields: 95 percent or higher.
4. Harmful file-action rate: less than 1 percent.
5. Query latency p95 on reference machine: under 8 seconds.

### Safety-First Validation Rules
1. Keep autonomous file actions in preview mode until thresholds are met.
2. Require citation-backed outputs for every extracted answer.
3. Log all failures with root-cause tags.
4. Re-run benchmark suite after each major fix.

### Exit Decision
1. **Pass:** Move to broader corpus and longer soak tests.
2. **Partial pass:** Scope down features and fix top failure classes.
3. **Fail:** Re-architect weak subsystem before expanding product surface.
