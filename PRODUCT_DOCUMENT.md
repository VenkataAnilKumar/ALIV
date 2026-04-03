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
1. **Cartography (Scanning):** The background rust agent detects the new files instantly.
2. **Attunement:** It streams the files through local OCR/Text extraction and sends the text to the local LLM.
3. **Reasoning:** The LLM is strictly prompted to return a standardized naming convention: `{DocumentType}_{VendorName}_{Date}` (e.g., `Invoice_Apple_Jan2025.pdf`).
4. **Action execution:** The agent natively renames the files in the OS and moves them into structurally organized sub-folders, handling collision logic gracefully.

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
* **Execution & AI Subsystems:** Embbeded Python / PTY Subprocesses
  * *Purpose:* Rust commands headless Python scripts or ONNX runtimes via standard input/output when heavy parsing (PDF scraping using `PyMuPDF`, Pandas Excel manipulation) or embedding generation is required.

---

## 4. The Core Paradigms

1. **Cartography (Scanning):** Rust's blazing-fast background threads (via libraries like `walkdir`) continuously index designated folders, scraping new files without disrupting the user's primary CPU thread.
2. **Attunement (Context Assembly):** Rust intelligently pulls the necessary constraints, historical data, and relevant document vectors from SQLite to build perfect, token-optimized prompts before contacting the local LLM reasoning engine. This now includes a full token-budget system: conversation history, pinned memory facts, retrieved chunks, and document summaries are assembled in priority order, with overflow dropped by relevance score.
3. **Process Execution:** True autonomous capability. Rust utilizes `std::process::Command` to dynamically spawn tests, invoke local Python parser scripts, trigger `ffmpeg`, or execute `kubectl` commands based on the agent's decision logic.
4. **Output Parsing Control Loop:** The system captures the `stdout`/`stderr` of all spawned background processes in real-time. If a Python extraction process fails, Rust captures the error, pipes it back to the agent for "self-correction", and dynamically streams the logs to the React UI for the user to monitor.
5. **Persistent Memory:** ALIV remembers. Every query/answer exchange is stored as a conversation turn in SQLite and recalled in future sessions. Users can pin any extracted fact to permanent memory with a citation. Document summaries are cached after ingestion so long documents never overflow the LLM context window. The system never forgets unless explicitly told to.

---

## 4.1 Memory Architecture

ALIV maintains three layers of persistent local memory — all stored in SQLite, never leaving the device:

### Conversation Memory
- Multi-turn chat history persists across app restarts
- Each session is a named `conversation` (auto-titled from first query)
- At query time: last N turns are loaded and injected as `[CONVERSATION HISTORY]`
- No re-stating context every session — ALIV knows what you asked yesterday

### Document Summary Cache
- Post-ingestion: Ollama generates 150-200 word document summaries per file
- Section-level summaries (~60-80 words) for large documents
- Used when raw chunk count would overflow the context window
- Summaries are invalidated and regenerated when the source file changes

### User Memory Facts
- Pin any extracted answer: *"Remember this: Vendor is Acme Corp."*
- Facts are recalled automatically in future queries when relevant
- Scopes: permanent (default), week (expires in 7 days), session (current only)
- Source citations preserved with each pinned fact

### Token Budget System
```
model_context_window (8192)
 - reserved output (1024)
 - system prompt (256)
 ────────────────────────
 available = 6912 tokens

Priority allocation:
  Instructions:           256  tokens
  Memory facts:           691  tokens  (10%)
  Conversation history:  1382  tokens  (20%)
  Retrieved chunks:      4147  tokens  (60%)
  Document summaries:     691  tokens  (10%, overflow reserve)
```

---

## 5. Market Strategy, Validation, and Commercial Readiness

### Target Market (Beachhead ICP)
ALIV's initial go-to-market focus is:
1. Small and mid-size legal firms.
2. Boutique accounting and tax advisory firms.
3. In-house legal and finance teams handling sensitive contracts and invoices.

These segments share the same buying trigger: high-value document workflows with strict confidentiality constraints that make cloud-first AI tooling difficult to adopt.

### Positioning
ALIV is a local-first autonomous document intelligence vault for legal and finance teams that need AI-powered retrieval, extraction, and file actions without moving sensitive documents to external cloud systems.

### Competitive Framing
1. **Against enterprise DMS platforms:** ALIV offers faster AI-centric workflow value and lower implementation friction.
2. **Against OCR-only automation tools:** ALIV combines semantic retrieval, reasoning, and autonomous file-system actions in one loop.
3. **Against general cloud copilots:** ALIV is designed for local/offline-sensitive deployment and direct local file operations.

### Pricing Hypotheses (To Validate)
1. Team/workspace annual subscription with add-ons for advanced automation and policy/audit controls.
2. Hybrid model: platform fee + seat fee + indexed data volume tier.

Initial willingness-to-pay assumption should be tested through paid pilots before broader rollout.

### 90-Day Validation Plan
1. **Weeks 1-3 (Problem Validation):** Run 20 interviews across legal and finance users; quantify current manual effort, cycle time, and error rates.
2. **Weeks 4-7 (Pilot Setup):** Secure 5 design partners; scope each pilot to two workflows: clause extraction/reporting and inbox organization with approval gates.
3. **Weeks 8-12 (Pilot Execution):** Run pilots, review KPIs weekly, and present ROI plus paid continuation proposals.

### Pilot KPI Thresholds
1. Time saved per target workflow: greater than 40 percent.
2. Extraction precision on critical fields: at least 95 percent.
3. Retrieval relevance (top results): at least 85 percent analyst-rated relevance.
4. Harmful autonomous action rate: less than 1 percent.

### Go / No-Go Criteria
Proceed to scale if at least 3 of 5 pilots meet KPI targets and indicate paid continuation. Pivot if trust remains low, local performance is insufficient on target hardware, or onboarding burden is too high.

### Product Safety Requirements for Commercial Trust
1. Approval-by-default mode for high-impact file operations.
2. Dry-run previews before rename/move actions.
3. Reversible audit log with one-click undo.
4. Citation-required extraction output for legal and finance workflows.

---

## 6. Product-Only Validation Track (No Buyer Dependency)

This track validates ALIV as a working product before customer development.

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
