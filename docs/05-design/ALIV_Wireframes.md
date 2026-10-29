# Wireframes
## Key screens, current and planned

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active

Low-fidelity layouts for the [UI Spec](ALIV_UI-Spec.md). ✅ marks shipped screens (see the screenshots in `docs/assets/`). The others are planned.

## High-fidelity mockups

Rendered with the app's design tokens ([Design System](ALIV_Design-System.md)). These are **mockups of planned screens**, not screenshots; the data is fictional.

| Screen | Release | Image |
|---|---|---|
| Review table with source drawer (W-5) | R2 | ![Review table](../assets/mockups/review-table.png) |
| Workflow run with redaction preview (W-7, W-8) | R1–R2 | ![Workflow run](../assets/mockups/workflow-run.png) |
| Agent Builder (W-10) | R3 | ![Agent Builder](../assets/mockups/agent-builder.png) |
| Inbox (W-11) | R3 | ![Inbox](../assets/mockups/inbox.png) |
| Admin settings on the desktop (W-13) | R1 | ![Desktop admin](../assets/mockups/admin-desktop.png) |
| Firm Server Admin Console (W-14) | R4 | ![Admin Console](../assets/mockups/admin-console.png) |

---

## W-1 Matters home ✅

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ALIV                                           [Vault health]  [+ New matter] │
├──────────────────────────────────────────────────────────────────────────────┤
│ Matters                                                                      │
│ Each matter is a private vault…                                              │
│                                                                              │
│ Upcoming deadlines (90 days)                                                 │
│  [12 days] Renewal notice · Lease_Acme_2025-03-01.pdf · Acme lease review     │
│  [31 days] Expiry · MSA_Northwind.docx · Vendor contracts                    │
│                                                                              │
│ ┌──────────────────────┐ ┌──────────────────────┐ ┌──────────────────────┐   │
│ │ ⚖ Acme lease review  │ │ ⚖ Project Falcon     │ │ ▣ AP Q4              │   │
│ │ Redstone Inc.        │ │ Falcon Holdings      │ │ Finance              │   │
│ │ 🔒 Local only        │ │ ☁ Hybrid             │ │ 🔒 Local only        │   │
│ │ 42 docs · 3 pending  │ │ 312 docs             │ │ 180 docs             │   │
│ └──────────────────────┘ └──────────────────────┘ └──────────────────────┘   │
└──────────────────────────────────────────────────────────────────────────────┘
```

## W-2 Ask ✅ (with R1 RoutePill)

```
┌─ Acme lease review  🔒 Local only ─────────────────────────────────────── ⋯ ┐
│ Overview  [Ask]  Register  Tables  Workflows  Documents  Approvals  Audit     │
├───────────────────────────────────────────────┬──────────────────────────────┤
│ [Model: Qwen 8B ▾]        (● This computer)   │ Sources                      │
│ ┌───────────────────────────────────────────┐ │ ┌──────────────────────────┐ │
│ │ Ask a question about this matter…   @  [➤]│ │ │[1] Lease_Acme.pdf  p.4   │ │
│ └───────────────────────────────────────────┘ │ │ "Tenant must give notice │ │
│                                               │ │  no later than 60 days…" │ │
│ Q  Which leases renew in 90 days?             │ ├──────────────────────────┤ │
│ A  Two leases renew before 2027-01-01:        │ │[2] Lease_Beta.pdf  p.2   │ │
│    • Acme: notice due 2026-11-02 [1]          │ │ …                        │ │
│    • Beta: notice due 2026-12-15 [2][3]       │ ├──────────────────────────┤ │
│    Qwen 8B · llamacpp · local · 5 sources     │ │[3] register · Lease_Beta │ │
│    Found by keyword and by meaning.           │ └──────────────────────────┘ │
│    [Save as DOCX]  (R2)                       │                              │
└───────────────────────────────────────────────┴──────────────────────────────┘
```

## W-3 Register ✅

```
│ [All 42] [Due in 90 days 5] [Passed 2] [Needs review 4]          [Export CSV] │
│ Document              Type     Parties            Expiry      Notice   Penalty│
│ Lease_Acme_2025.pdf   Lease    Acme · Redstone    2026-03-01  60 days  ✓      │
│ NDA_Meridian.pdf      NDA      Axiom · Meridian   2026-07-15  —        ✓      │
│   └ click a cell → source passage drawer                                      │
```

## W-4 Documents ✅

```
│ Documents   38 indexed · 4 not readable. Search matches by keyword and meaning│
│ Name / summary                                  Type   Status           Pass. │
│ Lease_Acme_2025.pdf                             Lease  [indexed]         12   │
│  Lease between Acme… Effective 2025-03-01; …                                  │
│ scan_017.jpg                                    NDA    [indexed][OCR 91%] 3   │
│ protected.pdf                                   —      [protected]       0    │
```

## W-5 Review table (R2)

```
┌─ Project Falcon  ☁ Hybrid ─────────────────────────────────────────────── ⋯ ┐
│ Tables › DD core terms                     (● local extract) [Run ▸][Export▾]│
│ [+ Column] [Template▾] [Filter: needs review ▾] [Group ▾]   142/300 cells ▓▓░ │
├──────────────────────┬───────────────┬──────────────┬──────────────┬─────────┤
│ Document             │ Change of     │ Consent      │ Liability    │ Notes ✎ │
│                      │ control? (Y/N)│ required? ⋔  │ cap ($)      │ (manual)│
├──────────────────────┼───────────────┼──────────────┼──────────────┼─────────┤
│ MSA_Northwind.pdf    │ Yes [4] ▮▮▮   │ Prior written│ 1x fees [7]  │ check   │
│                      │               │ consent [4]  │ ▮▮▯ 💬2      │         │
│ Supply_Orion.docx    │ No  [2] ▮▮▮   │ n/a          │ — not found  │         │
│ Lic_Vega.pdf         │ Yes [9] ▮▯▯ ⚠ │ ◌ running…   │ $2,000,000 ✱ │         │
│ …                    │               │              │  (edited)    │         │
└──────────────────────┴───────────────┴──────────────┴──────────────┴─────────┘
  ⋔ conditional column (runs where Change of control = Yes)   ⚠ low confidence
  Click any cell → SourceDrawer (passage highlighted · edit · comment · re-run)
```

## W-6 Add column (R2)

```
┌ Add column ───────────────────────────────────────────┐
│ Question  [Is there a cap on liability? If so, amount]│
│ Answer    (•) Amount  ( ) Yes/No  ( ) Date  ( ) Text  │
│           ( ) List    ( ) Classification [options…]   │
│ Only when [Change of control ▾] [= Yes ▾]  (optional) │
│ Route     ● This computer (extract)                   │
│ Preview on 3 rows:                                    │
│   MSA_Northwind   1x fees [7]                         │
│   Supply_Orion    not found                           │
│   Lic_Vega        $2,000,000 [3]                      │
│                               [Cancel] [Add & run all]│
└───────────────────────────────────────────────────────┘
```

## W-7 Workflow run (R2)

```
│ Workflows › Draft client memo                                    [Save DOCX]│
│ Run log                                     │ Output preview                 │
│ ✓ 1 Gather facts       ● local   12 src 4s  │ MEMORANDUM                     │
│ ✓ 2 Issues by severity ● local    8 src 9s  │ To: [Client]                   │
│ ◌ 3 Draft memo         ☁ cloud·redacted     │ Re: Project Falcon diligence   │
│     [Preview what was sent]                 │ 1. Change of control [4][9]…   │
│ ○ 4 Citations check    ● local              │                                │
```

## W-8 Redaction preview (R1)

```
┌ What will be sent to Anthropic · Claude (redaction: standard) ───────────────┐
│ Original (stays here)                │ Sent                                  │
│ Northwind Ltd may terminate if       │ [PARTY_1] may terminate if            │
│ Falcon Holdings is acquired; the     │ [PARTY_2] is acquired; the            │
│ cap is $2,000,000.                   │ cap is [AMOUNT_1].                    │
│ Contact jane.doe@northwind.com       │ Contact [EMAIL_1]                     │
│                         3 parties · 1 amount · 1 email replaced  [Cancel][Send]│
└──────────────────────────────────────────────────────────────────────────────┘
```

## W-9 Playbook result (R2)

```
│ NDA_Meridian.pdf  ·  Playbook: Mutual NDA v3                [Export redlines]│
│ Position                 Result        Evidence           Suggested redline  │
│ Term ≤ 3 years           ✓ compliant   §8 [2]                                 │
│ Residuals clause         ✗ deviation   §5.3 [4]           Delete §5.3 [Apply]│
│ Non-solicit              ! missing     —                  Insert clause [Add]│
│ Governing law: NY        ✗ deviation   §12 [6] (Delaware) Replace… [Apply]   │
```

## W-10 Agent Builder (R3)

```
│ Agents › Lease abstract v1 (draft)                      [Test run] [Publish] │
│ Steps                 │ Step 2: Flag deviations               │ Test run     │
│ 1 Review table  ●     │ Type     [Compare ▾]   Route: ● local │ Matter [▾]   │
│ 2 Flag deviations ●   │ Uses     Playbook: Lease v2           │ ✓ 1  4s      │
│ 3 Draft abstract ☁    │ Instr.   [Compare each lease to…]     │ ◌ 2 …        │
│ 4 Propose rename ⇢    │ Output   (•) Table ( ) Doc            │              │
│ [+ Add step]          │ [ ] Optional step                     │              │
│ ⇢ = proposes only (needs approval)                                            │
```

## W-11 Inbox (R3)

```
│ Inbox                                                [Mark all done]         │
│ Mon 3 Nov · Deadline digest                                                  │
│   [12 days] Renewal notice · Lease_Acme [clause]                    [Done]   │
│   [26 days] Expiry · MSA_Northwind [clause]                          [Done]   │
│ Today · New-document triage                                                  │
│   NDA_Orion.pdf → NDA · 2 deviations (Mutual NDA v3)   [Open] [Snooze]       │
```

## W-12 Matter settings (R1)

```
┌ Matter settings ──────────────────────────────────────┐
│ Policy        ( ) Local only   (•) Hybrid             │
│ Redaction     (•) Standard  ( ) Strict  ( ) Off ⚠     │
│ Cloud limit   [$50] per month                         │
│ Routes        Extract  ● local     Answer  ☁ cloud    │
│               Draft    ☁ cloud     Compare ☁ cloud    │
│                                        [Cancel][Save] │
└───────────────────────────────────────────────────────┘
```

## W-13 Desktop admin: AI policy (R1)

Settings › Admin for whoever manages the firm's install. Defaults for new matters (policy, redaction, allowed providers), routing by step type, hardware tier and local model, and cloud spend. Settings pushed by the firm's policy file (MDM) show as "locked by firm". A matter can be stricter than these defaults, never looser. See the [mockup](../assets/mockups/admin-desktop.png).

## W-14 Firm Server Admin Console (R4)

A browser console: Dashboard, Users & groups, Matters & walls, AI policy, Providers, Spend, Audit, Health & backups, Updates, SSO. The dashboard shows users, matters by policy, model calls by route, calls refused by policy, cloud spend, recent policy events, GPU model server and backups. **Admins see metadata only** (counts, routes, costs, audit entries), never document text. See the [mockup](../assets/mockups/admin-console.png).
