# ALIV — Future Product Strategy & Vision

**Status**: Documented for post-Phase 6 implementation. All ideas here are deferred until core phases (1-6) are complete and shipping.

**Last Updated**: 2026-04-02 (updated with Gemma 4 release data)

---

## Overview

This document captures three strategic product directions identified during early development. None of these change the current build plan. They are opportunities to pursue after the release candidate ships.

---

## Strategy 1: Local First, Cloud Next

### The Idea

ALIV starts fully local (Ollama, no cloud). After trust is established, users can optionally connect cloud AI models (OpenAI, Anthropic, Azure) for higher reasoning quality — with full control over what gets sent.

### Why This Fixes the Biggest Weakness

~~The #1 current limitation is local model quality. A 3B parameter model running on-device cannot match GPT-4 on complex legal document reasoning.~~

**Updated (2026-04-02)**: With Gemma 4 31B now available on Ollama, this limitation is substantially resolved. Local model quality has reached near-frontier level for document reasoning tasks. The "local first, cloud next" model is now a premium upgrade path rather than a quality fix — cloud becomes optional, not required.

### How It Works

**Per-workspace sensitivity setting:**
```
Workspace: "Client NDAs"       → Local Only (locked, cannot change)
Workspace: "Internal Memos"    → Hybrid (user decides per query)
Workspace: "Public Research"   → Cloud preferred
```

**Per-query routing:**
When a user runs a query, ALIV routes to the appropriate model based on workspace setting. Over time it learns per-workspace preferences and stops prompting.

**Document sensitivity classifier (always local):**
ALIV auto-tags documents on ingestion:
- Contains "CONFIDENTIAL", "NDA", "attorney-client", "privileged" → flagged local-only
- Everything else → follows workspace preference

The classifier itself always runs locally — no document metadata ever leaves the machine even in hybrid mode.

**LLM routing architecture:**
```
Query
  → SQLite/vector retrieval (always local)
  → LLM Router
      ↘ Ollama (local)         — default
      ↘ OpenAI API (BYOK)      — user opt-in
      ↘ Anthropic API (BYOK)   — user opt-in
      ↘ Azure OpenAI (BYOK)    — enterprise opt-in
```

Everything upstream of the LLM router — retrieval, chunking, embeddings, OCR, file indexing — stays local in all modes.

### Monetization Tiers

| Tier | Mode | Price | Target User |
|---|---|---|---|
| Local | Ollama only, fully offline | Free / one-time $49 | Compliance-first, price-sensitive |
| Pro | BYOK (bring your own API key) | $19/month | Power users wanting GPT-4 quality |
| Team | Managed cloud + shared workspace | $49/user/month | Small firms, team use |

### Build Order (Post Phase 6)

1. **Add BYOK support** — user enters OpenAI/Anthropic key in settings. Low engineering cost, immediate value unlock.
2. **Add LLM abstraction layer** — `LocalLLM` trait in Rust so any backend can be swapped.
3. **Add per-workspace routing rules** — UI toggle per workspace, persisted in SQLite.
4. **Add document sensitivity classifier** — keyword + pattern-based, runs at ingestion time.
5. **Add managed cloud tier** — ALIV hosts the API calls, abstracts key management. Subscription revenue engine.

### Revised Positioning

| Before | After |
|---|---|
| "Local AI tool for compliance teams" | "Privacy-first AI that scales from offline to cloud" |
| Competes only in local/offline niche | Competes across the full market |
| Loses when compliance requirement relaxes | Wins even when compliance is not the reason |

**Tagline**: *"Start private. Scale when ready. Your documents, your choice."*

---

## Strategy 2: Small LLMs Reaching Claude-Level Reasoning

### The Idea

The gap between small local models and frontier cloud models is closing rapidly. When a 7B parameter model achieves Claude-level reasoning quality on document tasks, ALIV's local mode becomes the best possible experience — no tradeoffs.

### Current Landscape (Updated with Gemma 4)

| Model | Parameters | Status | Quality | Runs on Ollama |
|---|---|---|---|---|
| llama3.2:3b | 3B | Available | Basic reasoning | Yes |
| Phi-4 (Microsoft) | 14B | Available | Strong reasoning | Yes |
| Gemma 3 (Google) | 12B | Available | Near GPT-4 | Yes |
| Qwen2.5 | 7B-14B | Available | Strong multilingual | Yes |
| **Gemma 4 E2B / E4B** | **2-4B** | **Available now** | **Offline, near-zero latency** | **Yes** |
| **Gemma 4 26B** | **26B** | **Available now** | **Frontier-class reasoning** | **Yes** |
| **Gemma 4 31B** | **31B** | **Available now** | **Claude-level, MMLU 85.2%** | **Yes** |

**Gemma 4 31B benchmarks** (confirmed, April 2026):
- MMLU: **85.2%** — strong general + legal reasoning
- AIME 2026 Math: **89.2%** — near-frontier reasoning
- LiveCodeBench: **80.0%** — advanced structured output
- GPQA Diamond: **84.3%** — PhD-level domain knowledge

**The target was "3-7B Claude-level in 12-24 months." Gemma 4 31B delivers Claude-level reasoning today. The thesis has been validated ahead of schedule.**

Gemma 4 also ships with:
- **140 language support** — ALIV's legal/finance market is global
- **Native function calling** — enables cleaner Phase 3 agentic file actions
- **Multimodal (audio + vision)** — future OCR replacement path
- Available via Ollama, LM Studio, Hugging Face, Kaggle

### What This Means for ALIV Right Now

```
Before Gemma 4:
  Local mode  → 3B model → "good enough" for simple queries
  Cloud mode  → GPT-4    → needed for complex legal reasoning

After Gemma 4:
  Local mode  → 31B model → Claude-level reasoning, fully offline
                             pull with: ollama pull gemma4:27b
  Cloud mode  → optional  → only for users wanting absolute cutting edge
```

**The product is already the best of both worlds. No tradeoff.**

### Gemma 4 E2B/E4B — The Mobile Opportunity

Edge models running on phones, tablets, and Raspberry Pi with near-zero latency opens a future expansion path:

```
ALIV Desktop (today)  →  ALIV Mobile (future)
  Laptop/workstation       iPad or Android tablet
  Ollama local server      Gemma 4 E2B on-device
  16GB RAM minimum         Works on any modern phone
```

A lawyer reviewing contracts on an iPad, fully offline, with no server dependency — this becomes technically feasible with Gemma 4 E2B.

### Why ALIV Is Positioned Right

Most competitors built cloud-first — hard to support local models as an afterthought. ALIV built local-first — trivial to upgrade the local model. When a better model ships, users update Ollama and pull the new model. The entire product gets smarter overnight with zero code changes.

### Three Futures — All Good for ALIV

| Scenario | What Happens | ALIV Position |
|---|---|---|
| Small LLMs reach Claude quality | Local mode becomes premium experience | Strongest possible |
| Gap narrows but doesn't close | Hybrid mode captures both markets | Strong |
| Gap stays large | Cloud-next tier carries revenue | Viable |

### What To Build To Be Ready

**Model abstraction layer** (implement when adding BYOK, Strategy 1):
```rust
pub trait LocalLLM {
    fn generate(&self, prompt: &str, options: GenerateOptions) -> Result<String>;
    fn model_name(&self) -> &str;
    fn context_window(&self) -> usize;
}

pub struct OllamaBackend { ... }    // today
pub struct LlamaCppBackend { ... }  // tomorrow
pub struct MlxBackend { ... }       // Apple Silicon optimized
```

**Model benchmark runner:**
```
python benchmarks/run_model_eval.py --model gemma4:27b
python benchmarks/run_model_eval.py --model phi4:14b
python benchmarks/run_model_eval.py --model llama3.2:3b
```
Runs existing retrieval + extraction benchmarks against any model. When a new model ships, confirms readiness in 10 minutes. Gemma 4 27B is the recommended default model to test first.

**Context window scaling:**
ALIV's `TokenBudget` in `core/memory.rs` is already parameterized. Just update the ceiling constant when a larger-context model becomes the default.

---

## Strategy 3: Computing Power as a Permanent Tailwind

### The Idea

Increasing consumer hardware capability (RAM, NPUs, faster CPUs) directly benefits ALIV. Unlike cloud AI where more compute means higher infrastructure costs, ALIV's users own the compute. Every hardware generation makes ALIV faster and smarter at zero additional cost.

### Three Forces All Pushing the Same Direction

| Force | Effect on ALIV |
|---|---|
| Better chips (Apple M-series, Qualcomm X Elite, Intel NPU) | Larger models run locally |
| Model distillation + MoE architectures | Same quality in fewer parameters |
| Better quantization (4-bit, 8-bit GGUF) | Model size cut 4x with minimal quality loss |

These forces are independent — any one alone improves ALIV. All three compounding simultaneously is a structural advantage that does not require any product changes.

### Hardware Trajectory

| Hardware | Today | 2 Years |
|---|---|---|
| MacBook M4 Pro | Runs 30B model | Runs 70B model |
| Windows Copilot+ PC | Runs 7-14B | Runs 30B+ |
| Mid-range laptop | Runs 3-7B | Runs 14-30B |
| Budget laptop | Runs 3B | Runs 7-14B |

Every tier gets 2-3 model size classes better every 2 years.

### What Computing Power Unlocks for ALIV

| Capability | Today | With 2x More Compute |
|---|---|---|
| Document batch size | 200 files | 2,000 files |
| Context window | 8,192 tokens | 128,000 tokens |
| Model quality | 3B-7B | 30B-70B |
| Real-time processing | Single file | Parallel ingestion |
| Response latency p95 | 8-15 seconds | Under 2 seconds |
| Embedding model | 384-dim MiniLM | 1536-dim large model |
| OCR | Tesseract 5 | On-device vision model |

All of this improves automatically as users upgrade hardware — zero code changes required.

### The Cloud Cost Problem ALIV Does Not Have

```
Cloud AI cost structure:
  More users → more GPU clusters → more electricity → rising costs
  Costs passed to users via subscriptions and API pricing

ALIV cost structure:
  More users → zero marginal compute cost
  User's hardware does the inference
  Infrastructure costs stay flat regardless of user count
```

As cloud AI scales, provider infrastructure costs grow. ALIV's infrastructure costs stay flat. Structural business model advantage that compounds over time.

### Historical Parallel

| Year | Prediction | What Actually Happened |
|---|---|---|
| 2005 | "Video editing needs workstations" | Final Cut / Premiere ran on every laptop |
| 2012 | "3D rendering needs render farms" | Blender ran on consumer GPUs |
| 2024 | "AI needs cloud servers" | Ollama runs on MacBook today |

ALIV is building for the **Blender moment of AI** — when capable AI moves from cloud infrastructure to local hardware permanently.

### 5-Year Compound Effect (Revised with Gemma 4 Data)

```
2026 — ALIV ships  ← WE ARE HERE
  Hardware: M4 MacBook, 16GB RAM
  Model: Gemma 4 27B / llama3.2:7B (user choice)
  Experience: Claude-level document Q&A, fully offline
              File organizer with LLM-quality rename suggestions
  Status: The "future" scenario is already the present

2027 — Mobile + Edge expansion
  Hardware: M5 MacBook, iPad with NPU, Android flagships
  Model: Gemma 4 E2B on mobile, 70B on desktop
  Experience: ALIV on tablet/phone, no laptop required
              Any legal professional, any device, offline

2028 — NPU standard in all consumer hardware
  Hardware: NPU in every laptop, tablet, and phone
  Model: 70B models real-time on laptop, 30B on phone
  Experience: Indistinguishable from cloud AI quality
             Zero data leaves any device
             Zero API costs forever

2030 — Full vision realized
  Every ALIV user has a private AI that knows every
  document they have ever touched, reasons at GPT-5
  level, costs $0/month in inference, and is 100%
  private across all their devices.
  Cloud AI cannot compete on privacy, cost, or latency.
```

---

## Competitive Landscape Summary

| Force | Cloud AI | ALIV |
|---|---|---|
| Better consumer hardware | Marginal benefit (already fast) | Direct benefit — runs bigger models |
| Smaller smarter models | Reduces their differentiation | Core product improves for free |
| More users | Rising infrastructure costs | Zero marginal inference cost |
| Privacy regulation tightening | Compliance burden grows | Natural structural advantage |
| Enterprise AI fatigue | Trust issues compound | Local = trusted by default |
| API pricing increases | Hurts their users | ALIV users unaffected |

---

## Current Product Positioning vs Future

### Current (Phases 1-6 focus)
*"A hyper-secure, offline-capable autonomous knowledge worker for local documents."*

Target: compliance-first legal and finance professionals who cannot use cloud AI.

### Future (Post Phase 6, all three strategies implemented)
*"Start private. Scale when ready. Your documents, your choice."*

Target: any knowledge worker who values privacy, cost control, or offline capability — not just compliance-mandated users.

---

## Implementation Roadmap (Post Phase 6)

| Priority | Initiative | Strategy | Effort | Notes |
|---|---|---|---|---|
| 1 | LLM abstraction layer (`LocalLLM` trait) | 1 + 2 | Medium | Swap Ollama model without code changes |
| 2 | Gemma 4 as default model | 2 | Small | `ollama pull gemma4:27b`, update config |
| 3 | Model benchmark runner | 2 | Small | Validate any new model in 10 minutes |
| 4 | BYOK support (OpenAI / Anthropic key) | 1 | Small | Cloud becomes optional upgrade |
| 5 | Per-workspace routing rules | 1 | Small | UI toggle, persisted in SQLite |
| 6 | Document sensitivity classifier | 1 | Medium | Keyword-based, runs at ingestion |
| 7 | Context window auto-scaling | 2 + 3 | Small | Gemma 4 supports large context |
| 8 | Parallel file ingestion | 3 | Large | Unlock 2,000+ file batches |
| 9 | ALIV Mobile (Gemma 4 E2B on-device) | 3 | Large | iPad/Android — Tauri mobile or React Native |
| 10 | Managed cloud tier + billing | 1 | Large | Subscription revenue engine |

**Priority shift from original plan**: Item 2 (Gemma 4 as default) moved up — it's a config change with immediate quality gain. Do this on day one post-Phase 6.

Start with 1-3. They are the lowest effort, highest immediate impact, and validate the model abstraction layer before any cloud work begins.

---

## One Risk To Watch

> **AI requires so much compute that even future consumer hardware cannot keep up.**

Theoretically possible if frontier AI keeps scaling to trillion-parameter models indefinitely. However, for **document intelligence specifically** — reading contracts, extracting fields, organizing files — the task complexity does not require frontier scale. Gemma 4 31B already achieves 85.2% MMLU and 84.3% GPQA Diamond running locally today. That is sufficient for every ALIV use case. You do not need GPT-5 to rename a PDF correctly or find penalty clauses in a contract.

## Gemma 4 Quick Reference

For future implementation sessions — Gemma 4 details confirmed April 2026:

| Variant | Use Case | Pull Command |
|---|---|---|
| E2B / E4B | Future mobile / edge | `ollama pull gemma4:e2b` |
| 26B | Balanced quality + speed | `ollama pull gemma4:26b` |
| 31B (flagship) | Best reasoning quality | `ollama pull gemma4:27b` |

**Key capabilities relevant to ALIV**:
- Native function calling — cleaner Phase 3 agentic file actions
- 140 language support — global legal/finance market
- Multimodal (audio + vision) — future path to replace Tesseract OCR
- Built on Gemini 3 research — maximizes intelligence-per-parameter
- Available: Ollama, Hugging Face, LM Studio, Kaggle, Vertex AI

---

*This document is a strategy reference, not a build plan. Do not act on anything here until Phases 1-6 are complete and the product is shipping to real users.*
