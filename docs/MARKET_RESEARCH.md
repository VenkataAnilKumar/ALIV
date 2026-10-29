# ALIV: Problem Validation & Competitive Landscape

*Research snapshot: October 2026. Third-party pricing figures come from review and aggregator sites unless noted. Confirm them with each vendor before using them in a pitch.*

---

## 1. The product today: what the code actually does

| Claimed capability (README / PRD) | State in repo | Note |
|---|---|---|
| Ingest PDF / DOCX / EML / images with OCR | ✅ Python workers (PyMuPDF, docx, email, Tesseract) | Solid foundation |
| Hybrid BM25 + vector retrieval with citations | ✅ `merge_hybrid_hits` in `core/retrieval.rs` | Works |
| Token-budget context assembly + persistent memory | ✅ `MemoryService::assemble_context` | Differentiated and well-built |
| Safe file actions: preview → approve → execute → undo | ✅ `core/actions.rs`, audit log | Strongest differentiator |
| **Cited LLM answer to a natural-language question** | ⚠️ **Not wired** | `run_retrieval_query` returns hits plus an `assembled_prompt`, but nothing sends it to Ollama. Only `summarization_worker.py` calls the LLM |
| **"Continuously monitors folders" / "detects new files instantly"** | ⚠️ Not implemented | No `notify` crate. Ingestion is a `walkdir` scan the user triggers |
| **"Writes an Excel summary to the Desktop"** (PRD Flow A) | ⚠️ Not implemented | Extraction exports JSON/CSV only |
| Structured extraction templates with confidence | ✅ `extractors/templates.py` | Mostly heuristic. Check whether the scores are calibrated |

**Implication:** the headline demo query ("find agreements expiring in 90 days and extract the penalty clauses") currently returns evidence chunks, not a synthesized answer. Close this gap before any pilot, since every competitor below already answers questions.

---

## 2. Is the problem real?

**Yes. The evidence is strong, but the pain is narrower than the PRD states.**

- **Confidentiality blocks adoption.** In the ABA 2024 Tech Survey, about 55% of respondents named confidentiality/security as their top cloud concern ([ABA](https://www.americanbar.org/groups/law_practice/resources/tech-report/2024/2024-cloud-computing-techreport/)). The Law360 Pulse 2025 AI survey found 54% of lawyers use GenAI, and ranked client confidentiality and data privacy as a top concern ([Law360](https://www.law360.com/pulse/articles/2299612)).
- **Ethics rules make it a duty.** ABA Formal Opinion 512 says lawyers must assess confidentiality risk before putting client data into GenAI tools.
- **The market is already moving on-device.** SpotDraft and LexBlog published 2026 guides on on-device contract review ([SpotDraft](https://www.spotdraft.com/blog/on-device-ai-contract-review), [LexBlog](https://www.lexblog.com/2026/07/07/on-device-ai-for-contract-review-in-2026-for-legal-teams/)). New startups such as LocalApps (NL, founded 2025) target small firms with on-device legal AI ([Stanford CodeX](https://techindex.law.stanford.edu/companies/localapps)).

**Caveats that matter for ALIV:**
1. **Large firms aren't blocked. They buy enterprise cloud with contractual guarantees** (Harvey, Luminance, Kira, Copilot under a firm tenant). The "can't use cloud AI" pain is sharpest for **solo practitioners, small firms (2–30 lawyers), accounting/tax boutiques, and regulated in-house teams** with client-imposed no-cloud clauses. The PRD's beachhead ICP is right. Keep it there.
2. **Accuracy is a bigger concern than privacy.** In ABA survey summaries, accuracy (75%) ranked above data privacy (47%). Lead with *citations and verifiability*, not only "offline".
3. **Small local models are weak at long-document reasoning.** Guides warn that 3B models struggle with 100-page contracts ([aimadefor](https://www.aimadefor.com/blog/local-ai-for-lawyers/)). The default `llama3.2:3b` is a demo model, not a production one. Plan a hardware tier (e.g. 14–32B on Apple Silicon / 24 GB GPU).

---

## 3. Competitive landscape

### A. Enterprise legal AI (cloud, high price)
| Product | Focus | Price signal | Threat to ALIV |
|---|---|---|---|
| **Harvey** | Full-stack legal AI for AmLaw firms | ~$100k+/yr contracts, no public price | Low: different buyer |
| **Kira (Litera)** | Contract clause extraction, M&A diligence | Est. $500–5k/mo. Litera reports use at ~64% of AmLaw 100 | Medium: owns "extraction" mindshare |
| **Luminance** | Contract review/negotiation | Quote only | Low |
| **Spellbook** | Contract drafting/review in Word | ~$99–299/user/mo, 7-day trial | Medium: cheap enough for small firms |

Sources: [costbench](https://costbench.com/compare/harvey-ai-vs-luminance/), [dupple](https://dupple.com/learn/best-ai-contract-review), [spellbook](https://spellbook.com/learn/luminance-pricing)

### B. Local / private "chat with your documents" (closest substitutes)
| Product | Strengths | Gaps vs ALIV |
|---|---|---|
| **AnythingLLM** (free, OSS) | Desktop + Docker, workspaces, agents, Ollama | Generic: no legal templates, no file actions, no audit |
| **GPT4All LocalDocs** (free) | Easiest setup for non-technical users | Collection-level only, no extraction |
| **Elephas** (Mac, ~$5–20/mo) | Offline "Super Brain" knowledge bases, polished UX | Mac-only, credit-limited, no file actions or extraction schemas |
| **DEVONthink** (Mac) | Long-time document manager for lawyers, added local AI in 2025 | Mac-only. AI is an add-on, not agentic |
| **LocalApps** (new, NL) | Purpose-built on-device legal AI for small firms | **Most direct competitor. Track closely** |
| **Copilot+ PC semantic search** | Built into Windows, NPU, offline | Search only, no answers/extraction. Indexing has been unreliable ([PCMag via Yahoo](https://tech.yahoo.com/computing/articles/semantic-search-makes-copilot-pc-140000240.html)) |

Sources: [toolhalla](https://toolhalla.ai/compare/anythingllm-vs-gpt4all), [toolradar Elephas](https://toolradar.com/tools/elephas/pricing), [DEVONtechnologies](https://devontechnologies.com/blog/20250513-local-ai-in-devonthink)

### C. AI file renaming / inbox organizers (Flow B substitutes)
| Product | Notes |
|---|---|
| **Renamer.ai** | Windows + Mac + web, OCR, 25+ file types, markets to accounting/legal ([renamer.ai](https://renamer.ai/solutions)) |
| **NameQuick** | Mac menu-bar app, local or cloud AI ([Setapp](https://setapp.com/de/apps/namequick)) |
| **PDF AI Renamer** | Free, local, Mac ([App Store](https://apps.apple.com/app/id6746876116)) |
| **Hazel** | Rule-based Mac automation, no content understanding |

The renamer roundups note that **none handle invoice- or contract-specific parsing well** ([renamer.ai comparison](https://renamer.ai/best-ai-file-renamer)). That's an opening for ALIV.

### D. Self-hosted document management + AI
- **Paperless-ngx + Paperless-AI / Paperless-GPT**: free, Docker, Ollama auto-tagging, RAG chat ([technotim](https://docs.technotim.com/posts/paperless-ngx-local-ai/)). Popular with technical users and small accounting shops. It needs a server and Docker, so it's not suitable for most lawyers.

---

## 4. Where ALIV can win

ALIV's defensible position is the **intersection** of capabilities that no single competitor ships together:

```
            local / offline
                 ▲
   AnythingLLM ● │ ● ALIV  ← cited answers + extraction + safe file actions
   GPT4All     ● │ ● LocalApps
   Elephas     ● │
 ────────────────┼────────────────► legal/finance-specific
   Copilot      ●│        ● Kira / Luminance / Harvey / Spellbook
                 │
              cloud
```

**Real differentiators (keep investing):**
1. **Reversible, approval-gated file actions with an audit log.** No local chat tool and no renamer does this. It fits "defensible process" expectations in legal work.
2. **Domain extraction to structured output** (expiry, penalty, vendor, amount) with citations, plus export to a register/spreadsheet.
3. **Persistent, scoped matter memory.**

**Not differentiators (table stakes):** "offline", "uses Ollama", "chat with PDFs". Free OSS tools already do these.

---

## 5. Recommendations

1. **Finish the core loop first.** Wire `assembled_prompt` → Ollama (streamed) → cited answer. Add a real folder watcher (`notify` crate). Ship XLSX export. These are claimed in the docs but missing from the code.
2. **Lead with a sharp first workflow:** *"Contract/lease register in 10 minutes"*. The user points ALIV at a folder and gets a verified spreadsheet of parties, dates, renewal/notice deadlines, and penalties, each cell linked to its source page. This is concrete and demo-able, and AnythingLLM/Elephas can't do it.
3. **Make "matter = workspace" the core unit.** Lawyers think in matters/clients. Add ethical-wall separation between workspaces.
4. **Address accuracy head-on.** Show per-field citations with click-to-page, an "unverified" state, and the benchmark numbers. Recommend a 14B+ model on capable hardware, and stay honest about limits on 3B models.
5. **Cross-platform is a real gap.** Elephas, DEVONthink, NameQuick, and PDF AI Renamer are Mac-only. Tauri gives ALIV a **Windows** advantage, and most small law and accounting firms run Windows.
6. **Pricing anchor:** price between free OSS tools and Spellbook, around **$30–60/user/mo** or a one-time license plus updates (local-software buyers like perpetual licenses). Validate this in pilots.
7. **Watch LocalApps and Microsoft.** If Copilot+ adds on-device Q&A over local files, generic local search becomes a commodity. ALIV's moat is the legal/finance workflow layer, not retrieval.

---

## 6. Interview questions for the 20 validation interviews (PRD §5)
- Do any client agreements or engagement letters forbid sending documents to third-party AI? How often?
- How do you build a contract/lease register today? Roughly how many hours per matter?
- What would you need to see before trusting an extracted date or amount without re-reading it?
- What OS and hardware do your staff use (RAM/GPU)? Who installs software?
- Would you pay per seat, per firm, or one-time? What do you pay for Clio, PracticePanther, or similar tools?
