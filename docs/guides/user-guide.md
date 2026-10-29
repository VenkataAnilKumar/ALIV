# ALIV user guide

ALIV keeps each client engagement in a **matter**: a private vault of documents on your computer. You can ask questions and get answers that cite the documents, review the key terms ALIV extracted, approve file renames, and see an audit log of everything that happened.

## 1. Set up

1. **Install ALIV** and open it.
2. **Choose a model.** For confidential work, download a local model when ALIV offers one on first run, or later under Settings → Model providers → llama.cpp. Local models run on your computer and nothing leaves it. A cloud provider (OpenAI, Anthropic and others) can be added under Settings → Model providers, but only matters set to *Hybrid* may use it (see §2).
3. **Optional, for scanned documents:** install Tesseract OCR, plus Poppler for scanned PDFs.
   - macOS: `brew install tesseract poppler`
   - Debian/Ubuntu: `sudo apt install tesseract-ocr poppler-utils`
   - Windows: the [UB Mannheim installer](https://github.com/UB-Mannheim/tesseract/wiki)

   Without it, scans are listed as unreadable with that reason. Everything else works.

## 2. Create a matter

**Matters → New matter.** Give it a name, the client, a type (Legal or Finance) and a **model policy**:

| Policy | What it means |
|---|---|
| **Local only** (default) | Passages from this matter are only ever sent to a model on this computer. A cloud model is refused, and the refusal is logged. |
| **Hybrid** | Cloud models may be used. Only the passages matching a question are sent, never whole documents, and every call is logged. |

You can change the policy later under the matter's settings (⋯ → Settings).

## 3. Add documents

In the matter, click **Add folder** and pick one or more folders. ALIV reads them on your computer:

- **Supported files:** PDF, Word (.docx), Excel (.xlsx), PowerPoint (.pptx), HTML, text, Markdown and email (.eml).
- **Scans:** scanned PDFs and images (PNG, JPEG, TIFF) are read with OCR when Tesseract is installed. Each shows an **OCR nn%** badge with how confident the reading was.
- **Re-adding a folder** only re-reads files that changed.
- **Files that can't be read** stay listed under **Documents** with the reason.

Your files are never moved or changed by adding them.

## 4. The register

**Register** lists what ALIV extracted from each document:

- **Fields:** type, parties, effective and expiry dates, renewal-notice deadline, amount, notice period, governing law and penalty clause.
- **Sources:** click any value to see the passage it came from.
- **Confidence:** each value has a confidence score. Low-confidence values, including values read from poor scans, appear under **Needs review**.
- **Filters:** *Due in 90 days* and *Passed* show deadlines at a glance.
- **Export CSV** saves the register for a spreadsheet.

Each document also has a one-line **summary** under Documents (type, parties, key dates, amount), built from these fields.

## 5. Ask

Type a question in **Ask**, for example *"Which contracts renew in the next 90 days?"* or *"Can we end the services agreement early?"*

- **How sources are found:** ALIV finds the most relevant passages by keyword and, when the local embedding model is available, by meaning. "End early" will find a "termination for convenience" clause. Under the answer, ALIV says which kind of search it used.
- **Citations:** the model answers only from those passages and cites each statement as **[1]**, **[2]** and so on. Click a citation to read the source.
- **No model selected:** deadline questions are answered straight from the register, and other questions show the matching passages.
- **Large matters:** ALIV sends what fits the model's context window, most relevant first, and says how many matching sources were left out.

The pill above the question box shows where the answer will be computed: **Runs on this computer**, or **Cloud · matching passages only, logged**.

## 6. Approvals

ALIV can propose tidy file names such as `Lease_AcmeProperties_2025-03-01.pdf`:

- **Proposals only:** nothing on disk changes until you approve a proposal and click **Apply approved**.
- **No overwrites:** ALIV never overwrites an existing file.
- **Undo:** every applied rename can be undone.

## 7. Audit log

**Audit log** records every import, policy change, model call (allowed or refused), proposal and file change. Entries are chained with SHA-256, and **Verify integrity** checks that none has been altered since it was written.

## 8. Vault health

**Matters → Vault health** shows:
- how many documents are indexed;
- why any could not be read;
- whether OCR is available;
- how much of each matter search covers by meaning;
- whether the audit chain verifies.

**Copy report** copies it for support. It contains counts and reasons only, never document text. App logs are under Settings → General.

## Where your data is

- **The vault:** a single file, `vault.db`, in `ALIV/data/aliv/` under your system's application-data folder. You can move the data folder under Settings → General.
- **Your documents:** stay where they are. ALIV indexes them in place.

See [Security and privacy](security.md) for what is stored and what can leave your computer.
