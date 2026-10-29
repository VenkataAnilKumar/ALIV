# Brand Identity
## Who ALIV is, how it speaks, how it looks

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active

---

## 1. The name

**ALIV**: *Autonomous Local Intelligence Vault*.

| Part | Meaning |
|---|---|
| **Autonomous** | Agents do multi-step work: tables, workflows, monitors. People approve what changes. |
| **Local** | The matter vault lives on the firm's machines. Local models are the default. |
| **Intelligence** | Cited answers, extraction, drafting and review, at Harvey-class depth. |
| **Vault** | Each matter is a private, audited container. Nothing mixes across clients. |

Write it as **ALIV**, all capitals, no punctuation. Say it like "alive".

## 2. Positioning

**One line:** *The hybrid AI platform that keeps client matters on your machines.*

**Statement:** for law firms, in-house teams and finance functions that need Harvey-class AI but can't send every client document to someone else's cloud, ALIV is a hybrid intelligent agent platform. It runs agents, review tables and workflows over a private vault on the firm's own hardware. Unlike cloud-only legal AI, ALIV decides per matter and per step what may leave the machine, redacts it first, and proves every call in a tamper-evident audit log.

**Pillars:**

| Pillar | Proof point |
|---|---|
| **Private by default** | New matters are Local only; local embeddings; documents read in place |
| **Hybrid by policy** | Per-matter, per-step routing; redaction before cloud; route shown every time |
| **Cited and checkable** | Every answer, cell and draft traces to a passage |
| **In control** | Agents propose, people approve; undo for file actions |
| **Provable** | Hash-chained audit log, Verify integrity, exportable trail |

## 3. Personality

| ALIV is | ALIV is not |
|---|---|
| Precise | Hype-driven |
| Calm | Alarmist |
| Candid about limits | Over-promising ("100% accurate", "unhackable") |
| Professional, plain-spoken | Jargon-heavy or cute |
| Respectful of expert judgement | Replacing the lawyer |

## 4. Voice and tone

- **Say what happens, and where.** "Runs on this computer." "Sent to Anthropic after redaction."
- **Prefer the user's words:** matter, clause, passage, register, deadline, playbook.
- **Show, don't claim.** Use a citation instead of "accurate" and an audit entry instead of "secure".
- **Short sentences, sentence case, active voice.**
- **Adjust tone to the moment:** neutral for routine work, direct for refusals and errors, never blaming the user.

| Situation | Write | Don't write |
|---|---|---|
| Policy refusal | "This matter is Local only, so the cloud model wasn't used. Pick a local model or switch the matter to Hybrid." | "Error: request blocked." |
| Low confidence | "Low confidence. Check the source." | "AI may be wrong!" |
| Empty matter | "Add a folder to start. Files stay where they are." | "Nothing here yet 🎉" |
| Marketing | "Harvey-class review tables, on your hardware." | "The world's most secure legal AI." |

## 5. Visual identity

| Element | Spec |
|---|---|
| **Primary colour** | Teal `#0e7490` (light), `#22b8cf` (dark). Trust, calm, clarity |
| **Neutrals** | Greys from the app's `oklch` tokens; white and near-black backgrounds |
| **Semantic colours** | Emerald (local, success), sky (hybrid, info), amber (attention), red (blocked, error), violet (edited, agent) |
| **Type** | Inter for UI and marketing; tabular numbers in data |
| **Logo** | The ALIV mark in `apps/desktop/src-tauri/icons` (app icon set for macOS, Windows, Linux) |
| **Iconography** | Lucide line icons, 1.5 px stroke; lock = Local only, cloud = Hybrid |
| **Imagery** | Real product screens (`docs/assets/`), never stock photos of gavels or robots |

Details and tokens: [Design System](../05-design/ALIV_Design-System.md).

## 6. Messaging by audience

| Audience | Lead with | Proof |
|---|---|---|
| Partners, GCs | Client confidentiality and outside-counsel guidelines | Local only by default, per-matter policy, audit export |
| Associates, analysts | Hours saved on review and drafting | Review tables, workflows, cited answers |
| Legal ops, IT | Control and evidence | Gateway, redaction, Firm Server, health report |
| Finance controllers | Fewer payment errors, deadlines met | Invoice-to-contract check, covenant tracker |

## 7. Claims we can and can't make

| ✅ Can say (true today) | ⏳ Say only when shipped | ❌ Never say |
|---|---|---|
| Documents are indexed on your computer | Every model call goes through one gateway (R1) | "Nothing can ever leak" |
| New matters are Local only by default | Identities are redacted before cloud calls (R1) | "100% accurate" |
| Answers cite their sources | Vault encrypted at rest (GA) | "Replaces lawyers" |
| Tamper-evident audit log | SOC 2 (only once audited) | "Certified compliant" without a certificate |
| Skills and MCP are supported (base app) | Web search, MCP and skills follow matter policy (R1) | "Every tool is safe" |
| | Connectors to iManage, SharePoint… (R3) | "Integrates with everything" |

The source of truth for security claims is [`security.md`](../guides/security.md).
