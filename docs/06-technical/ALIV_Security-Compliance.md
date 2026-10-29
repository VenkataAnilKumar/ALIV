# Security & Compliance
## Threat model, controls and compliance mapping

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active. Customer-facing summary: [`security.md`](../security.md), which is the source of truth for what is true *today*.

---

## 1. Assets

| Asset | Where | Sensitivity |
|---|---|---|
| Client documents | Firm's disks and shares (read in place) | Highest |
| Vault (passages, fields, summaries, embeddings) | `vault.db` | Highest: derived client data |
| Audit log | `vault.db` `audit` | High: evidence |
| Provider API keys | OS keychain (R1) | High |
| Model weights | App data folder | Low |
| Firm knowledge (playbooks, clauses, KBs) | Vault (R2–R3) | High |

## 2. Trust boundaries

1. **Web UI ↔ Rust core (IPC).** The UI is untrusted for policy. All enforcement is in Rust.
2. **Machine ↔ cloud providers.** Crossed only for Hybrid matters, through the gateway (R1).
3. **Desktop ↔ Firm Server.** HTTPS with OIDC (R4).
4. **Documents ↔ parsers.** Untrusted input (malformed PDFs, images).

## 3. Threats and controls (STRIDE summary)

| Threat | Example | Control | State |
|---|---|---|---|
| **Information disclosure: wrong matter** | Search returns another client's passage | Every query scoped by `matter_id`; tests | ✅ |
| **Information disclosure: cloud** | A Local only matter sent to a cloud model | `authorize_model_call` refuses and audits | ✅ (Ask path) |
| | Code path that bypasses the check | All calls through the Rust gateway; web layer has no keys; CI check | R1 |
| | Identities exposed to the provider in Hybrid | Redaction with placeholders, preview, restore locally | R1 |
| **Information disclosure: device theft** | Laptop stolen | Full-disk encryption (customer); vault encryption at rest | Customer / GA |
| **Information disclosure: support** | Health report leaks text | Counts and reasons only | ✅ |
| **Tampering** | Audit entries edited | SHA-256 hash chain, Verify integrity | ✅ (tamper-evident, not tamper-proof) |
| **Tampering** | CSV formula injection | Cells starting with `= + - @` neutralised | ✅ |
| **Repudiation** | "AI didn't do that" | Audit entry per model call and file action; exportable trail | ✅ / R2 export |
| **Elevation** | Agent renames or sends without consent | Agents only propose; approval and undo | ✅ actions / R3 agents |
| **Elevation** | Prompt injection in a document instructs the agent | Tools are scoped; side effects need approval; documents are data in prompts; outputs cite sources | R2 |
| **DoS / parser bugs** | Malicious PDF crashes ingestion | Per-file errors isolated; resumable ingestion; fuzzing of extractors | ✅ isolation / R2 fuzzing |
| **Spoofing** | Fake update | Signed installers and updates | GA |
| **Spoofing** | User impersonation on the server | SSO (OIDC/SAML), roles, ethical walls | R4 |

## 4. Security engineering practices

- **Memory-safe core** (Rust); no `unsafe` in the vault crate.
- **Dependencies:** pinned lockfiles, `cargo audit` and `yarn npm audit` in CI (GA track), and a licence review for new native libraries.
- **Secrets:** never in the repo, logs or audit; keys in the OS keychain only.
- **Logging:** no document text in logs, crash reports or the audit log.
- **Testing:** policy matrix tests (every step type × policy), redaction recall gate (≥ 98% parties and people), IPC permission test, audit chain tests.
- **Vulnerability reporting:** [`SECURITY.md`](../../apps/desktop/SECURITY.md); fix critical issues within 14 days.

## 5. Compliance mapping

ALIV is software the firm runs, not a hosted service, so many obligations sit with the firm. ALIV's job is to make them easy to meet and to evidence.

| Framework / obligation | What it asks | How ALIV helps | State |
|---|---|---|---|
| **ABA Model Rule 1.6** (confidentiality) and Formal Opinion 512 (generative AI) | Reasonable efforts to prevent disclosure; informed consent for self-learning tools that expose client data | Local only by default; per-matter policy; redaction; no training on customer data; audit evidence | ✅ / R1 |
| **ABA Rule 5.3** (supervising non-lawyer assistance) | Supervise AI output | Citations, confidence, approvals, uncited markers | ✅ / R2 |
| **Outside-counsel guidelines** | Client-imposed AI and data rules | Policy per matter; audit export per matter | ✅ / R2 |
| **GDPR / UK GDPR** | Minimisation, purpose limitation, transfers, Art. 30 records | Data stays on firm machines; cloud only by policy with redaction; audit as a record of processing | ✅ / R1 |
| **SRA (UK) / confidentiality duties** | Protect client information | As above | ✅ |
| **SOC 2 (for ALIV the company)** | Controls over ALIV's own build and release process | Planned for Firm Server customers; no claim until audited | Planned |
| **ISO 27001 alignment** | ISMS | Controls above map to A.5, A.8 (asset, access, crypto, logging) | Planned |
| **EU AI Act** | Transparency for general-purpose AI use | Route, model and provider shown on every output | ✅ / R1 |
| **Finance: SOX ITGC, record retention** | Change control, evidence | Audit chain; approvals for changes | ✅ |

**Rule:** we never claim a certification we don't hold. Marketing follows the "can say" table in the [Brand Identity](../01-brand/ALIV_Brand-Identity.md).

## 6. Known limits (today)

Copied from `security.md` and kept in sync:
1. The vault is not encrypted by ALIV (use full-disk encryption). Fixed in the GA track.
2. The audit log is tamper-evident, not tamper-proof.
3. The policy check guards the Ask flow; the cloud request is sent from the app after approval. Fixed in R1 (gateway).
4. The provider's own handling of text sent to it is governed by the firm's agreement with that provider.
