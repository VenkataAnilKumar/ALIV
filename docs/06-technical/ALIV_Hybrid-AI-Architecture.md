# Hybrid AI Architecture
## Model gateway, router, redaction and agent runtime

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Design (R1–R3). Current state is described where it exists.

---

## 1. Goal

ALIV uses **strong cloud models where a matter allows it, and local models everywhere else**, without the cloud ever seeing more than it needs, and with every call on the record. This document describes the components that make that true: the **Model Gateway** (FS-8), **Redaction** (FS-9), the **Router** (FS-10), and the **Agent Runtime** that workflows, tables and agents run on (FS-11/12/16/17).

## 2. Current state (R0)

- **Ask** builds its prompt in the web layer.
- **The policy check:** the vault's `authorize_model_call` checks the matter policy and audits the call.
- **The model call:** the web layer then sends the request itself.
- **Embeddings:** computed by the local llama.cpp engine.
- **Already in the Rust core (from the base app):** a provider proxy and OpenAI-compatible local server (`src-tauri/src/core/server/proxy.rs`), and provider-key storage in the OS keychain with an encrypted-file fallback (`provider_secrets.rs`). The gateway builds on these rather than starting from scratch.

**Gap:** the cloud request is sent from the web layer after the check, so the check guards the Ask flow but is not unbypassable. R1 moves all calls into the Rust core.

## 3. Target components

```
 Experiences (web-app)                     Rust core (src-tauri)
 ───────────────────────                   ───────────────────────────────────────────────
 Ask · Review Tables · Workflows ──┐       ┌──────────── Agent Runtime ─────────────┐
 Agent Builder · Monitors          │ IPC   │ plan → for each step:                  │
                                   └──────▶│   gather sources (vault tools)         │
                                           │   route (Router)                       │
                                           │   call (Gateway)                       │
                                           │   verify citations                     │
                                           │   write outputs, propose actions       │
                                           └───────────┬────────────────────────────┘
                                                       │ ModelRequest{matter, step_type,
                                                       │   sources[], instructions}
                                           ┌───────────▼────────────────────────────┐
                                           │ Model Gateway                          │
                                           │ 1. policy check (matter, step)         │
                                           │ 2. Router → target model + location    │
                                           │ 3. if cloud: Redactor.redact()         │
                                           │ 4. provider adapter (stream)           │
                                           │ 5. if cloud: Redactor.restore()        │
                                           │ 6. audit entry (hash chain)            │
                                           └───┬───────────────────────────┬────────┘
                                   local       │                           │  cloud (Hybrid only)
                         ┌─────────────────────▼──┐             ┌──────────▼──────────────┐
                         │ llama.cpp / MLX engines │             │ OpenAI-compatible,      │
                         │ (chat + embeddings)     │             │ Anthropic adapters      │
                         └─────────────────────────┘             │ keys in OS keychain     │
                                                                 └─────────────────────────┘
```

**The invariant:** the web layer and agents hold **no provider keys** and make **no provider HTTP calls**. Only the gateway does.

## 4. Model Gateway (FS-8)

```rust
// Sketch of the interface (Rust, in the vault/core crate)
pub struct ModelRequest {
    pub matter_id: String,
    pub step: StepType,            // Extract | Classify | Summarise | Answer | Draft | Reason | Compare | Embed
    pub run_id: Option<String>,    // workflow / agent run, for the audit trail
    pub system: String,
    pub sources: Vec<SourceRef>,   // passage ids, register rows, library clauses
    pub prompt: String,
    pub max_output_tokens: u32,
}

pub enum Route { Local { model: String }, Cloud { provider: String, model: String } }

pub trait Gateway {
    fn plan(&self, req: &ModelRequest) -> Result<RoutePlan>;            // for previews: route + redacted text
    fn complete(&self, req: ModelRequest, out: impl FnMut(Chunk)) -> Result<Completion>;
    fn embed(&self, matter_id: &str, texts: &[String]) -> Result<Vec<Vec<f32>>>; // local only, by rule
}
```

**Flow of one `complete` call:**
1. Load the matter and its policy. If the matter is missing, refuse.
2. `Router::decide(policy, step, firm_defaults, hardware)` → `Route`.
3. If `Route::Cloud` and the policy is *Local only*: **refuse**, audit `model_call_blocked`, and return an error.
4. Assemble the prompt from `sources`, with the context budget for the target model's window.
5. **Cloud only:** `Redactor::redact(prompt, matter_entities)` → redacted text and a mapping. Hash the redacted text for the audit entry.
6. Stream from the adapter.
7. **Cloud only:** `Redactor::restore(chunk, mapping)` for each chunk before it reaches the UI.
8. Append an audit entry:

   ```
   {matter, run_id, step, route, provider, model, tokens_in, tokens_out, redaction_level,
    redacted_sha256, sources:[ids], outcome}
   ```

**Embeddings** are always local (rule), so meaning search works for Local only matters.

## 5. Router (FS-10)

**Default routing:**

| Step type | Local only | Hybrid (default) |
|---|---|---|
| Embed | local | local (always) |
| Extract, Classify | local | local |
| Summarise | local | local; cloud if the document exceeds the local context |
| Answer (Ask) | local | cloud |
| Compare | local | cloud |
| Draft, Reason | local | cloud |

**Override levels**, each narrower one wins:
- firm defaults;
- the matter;
- the workflow or agent step.

A step can force local even in a Hybrid matter, but it can **never** force cloud in a Local only matter.

**Local model choice** uses a hardware probe (RAM/VRAM, Apple Silicon), picking from tiers:

| Tier | Example class | Use |
|---|---|---|
| Small | 3–4B | extraction and classification on laptops |
| Medium | 7–14B | answers and summaries |
| Large | 24–32B+ | drafting locally on workstations or Firm Server |

The exact models follow the bundled catalog.

## 6. Redaction (FS-9)

**Entity sources:**
1. **Known entities from the vault:** register parties, signatories, email senders and recipients. These are exact and high-precision.
2. **Local NER:** a small local model or rules for people, organisations, emails, phone numbers, IBAN/account numbers and addresses.
3. **Optional:** amounts and dates (strict level).

**Placeholders** are stable within one call (`[PARTY_1]`, `[PERSON_2]`, `[AMOUNT_1]`), so the model can still reason about who did what. The mapping lives in memory for the call and is never sent or logged. The audit log keeps only the redacted text's hash, plus the full redacted text when the matter's audit level is "full".

**Quality gate:** a labelled redaction test set (synthetic contracts and emails). A release needs **≥ 98% recall** for parties and people, and none of the matter's register parties may leak.

**User control:**
- a **preview** shows exactly what will be sent;
- strict mode;
- turning redaction off requires Hybrid plus an explicit per-matter setting, which is audited.

## 7. Agent Runtime

Shared by Review Tables, Workflows, Agents and Monitors.

```
Definition (workflow/agent JSON, versioned)
  inputs → steps[] → outputs
  step = { id, type: StepType, instructions, tools[], output: Text|Table|Doc|Proposal, optional: bool }

Run
  run_id, definition_version, matter_id, trigger (user | schedule | folder | date)
  for step in steps:
     sources = tools(step)                  # vault search, register, table read, library, knowledge base
     result  = gateway.complete(step → ModelRequest)
     verify(result)                         # every claim cites an id from `sources`; flag otherwise
     persist(result)                        # table cells, documents, proposals
     audit(step, route, ids, outcome)
```

**Rules:**
- **Proposals only:** tools that change things (file actions, outbound messages, redlines applied to a file) create **proposals** in the approval queue. Agents never apply them.
- **Deterministic tools:** tools are vault functions such as search, register, table operations, file proposals and notifications. They are not free-form code execution.
- **Resumable runs:** a run can resume after a crash; completed steps are not re-run.
- **Review Tables** are a special run: one step per column, fanned out per document, run in parallel locally (bounded by CPU/GPU) and sequentially for cloud (rate limits). Each cell is a `ModelRequest` whose sources are that document's top passages.

## 8. Data stored

| Table (new in R1–R3) | Purpose |
|---|---|
| `model_calls` | Gateway audit details. The hash chain links them via `audit` |
| `entities` | Known parties and people per matter (from the register) for redaction |
| `review_tables`, `review_columns`, `review_cells` | Tables, columns (question, type, condition) and cells (value, citation, confidence, edits, comments) |
| `definitions` | Workflow and agent definitions with versions |
| `runs`, `run_steps` | Run history, route per step, outputs |
| `playbooks`, `positions` | Playbooks and their positions |
| `clauses` | Clause library |
| `knowledge_bases` | Read-only collections (documents live in the same passage store, flagged) |

The full schema is in the [Data Model](ALIV_Data-Model.md).

## 9. Security properties

| Property | Mechanism |
|---|---|
| A Local only matter never reaches a cloud model | Gateway refuses any cloud route; there are no provider keys or HTTP clients in the web layer; tested per step type |
| The cloud sees the minimum | Top-k sources only, redacted, within a budget |
| Every call is accountable | Audit entry per call with route, model, sizes and source ids; chain integrity check |
| Agents can't act on their own | Changes are proposals; approval is required and audited |
| Keys are protected | OS keychain; never serialised to the web layer |

## 10. Testing

- **Gateway policy matrix:** every step type × policy × route, asserting refusal or allowance and the audit entry.
- **Redaction:** recall and precision on the labelled set, run in CI with a threshold gate.
- **Agent runtime:** deterministic fake models replay fixed outputs; citation verification; resume after a forced crash.
- **No provider calls from the web layer:** a CI check fails if web code imports provider SDKs or hard-codes provider URLs outside the gateway adapters.
