# Design System
## Tokens, components and patterns

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active. Documents what ships in `apps/desktop/web-app` and sets rules for new screens.

---

## 1. Principles

1. **Calm and precise.** Legal and finance users read closely. Use restrained colour and plenty of whitespace, and save emphasis for what needs attention (deadlines, deviations, refusals).
2. **Provenance is part of the UI.** Citations, confidence, OCR % and "runs on this computer / cloud" are visible components, not tooltips only.
3. **State is never ambiguous.** Every item shows whether it's proposed, approved, applied, undone, running, failed, edited or uncited.
4. **Same in light and dark.** Every token has a dark value. Contrast is checked in both.
5. **Built on the base components.** New screens use the shared `components/ui` primitives (shadcn/Radix pattern) and the Matters `primitives.tsx`, not bespoke widgets.

## 2. Tokens

Defined as CSS variables in `web-app/src/index.css` (`:root` and `.dark`) and exposed to Tailwind.

### Colour

| Token | Light | Dark | Use |
|---|---|---|---|
| `--primary` | `#0e7490` (teal 700) | `#22b8cf` | Primary actions, links, focus accents, brand |
| `--background` | `oklch(1 0 0)` | `oklch(0.18 0 0)` | App background |
| `--foreground` | `oklch(0.145 0 0)` | `oklch(0.985 0 0)` | Body text |
| `--muted` / `--muted-foreground` | `oklch(0.97 0 0)` / grey | `oklch(0.269 0 0)` / grey | Secondary surfaces and text |
| `--border` | `oklch(0.922 0 0)` | white 10% | Dividers, inputs |
| `--destructive` | `oklch(0.577 0.245 27)` | `oklch(0.704 0.191 22)` | Delete, refusals, errors |

### Semantic tones (`Pill` and status)

| Tone | Light | Meaning in ALIV |
|---|---|---|
| `success` (emerald) | emerald 700 on 10% | Local only, indexed, executed, audit verified, compliant |
| `info` (sky) | sky 700 on 10% | Hybrid / cloud, approved, upcoming, OCR ≥ 75% |
| `warning` (amber) | amber 700 on 10% | Due soon, skipped, low confidence, OCR < 75%, deviation |
| `danger` (red) | red 700 on 10% | Passed deadline, blocked by policy, failed, audit broken, missing clause |
| `accent` (violet) | violet 700 on 10% | Undone, edited, agent-generated |
| `neutral` | muted | Proposed, rejected, n/a |

**Rule:** colour never carries meaning alone. Every pill has a word or an icon.

### Type

| Token | Value | Use |
|---|---|---|
| `--font-sans` | Inter | All UI text |
| `--text-xs` | ~12 px | Pills, metadata, captions |
| `--text-sm` | ~14 px | Body, tables (default) |
| base | ~16 px | Answer text, documents |
| `tabular-nums` | — | All numbers in tables and stats |

The base size follows the user's interface setting (`--font-size-base`). Never hard-code px.

### Shape and space

| Token | Value |
|---|---|
| `--radius` | 0.625 rem (10 px); `sm` −4, `md` −2, `xl` +4 |
| Spacing | Tailwind 4 px scale; panels `p-3`/`p-4`, page gutter `px-4 md:px-8` |
| Elevation | Borders, not shadows, for cards. Shadows only for popovers and dialogs |

## 3. Components

### Base (`components/ui`)
Button (default, outline, ghost, destructive; sizes `sm`, `icon`, `icon-xs`) · Input · Textarea · Dialog · Sheet · Drawer · Dropdown menu · Popover · Hover card · Tooltip · Tabs · Switch · Radio group · Slider · Progress · Skeleton · Separator · Sonner (toasts) · Sidebar · Kbd.

### Matters primitives (`aliv/components/primitives.tsx`)

| Component | Purpose | Variants |
|---|---|---|
| `Pill` | Status and metadata chip | the six tones |
| `PolicyPill` | Matter policy | Local only (lock, success) / Hybrid (cloud, info) |
| `RiskPill` | File-action risk | low / medium / high |
| `ActionStatusPill` | Proposal state | proposed, approved, rejected, executed, undone, failed |
| `DaysPill` | Deadline urgency | passed, soon, upcoming, later |
| `StatCard` | Overview number with hint and icon | — |
| `Section` | Titled panel with description and actions | — |
| `EmptyState` | Icon, title, body, action | — |
| `Tabs` | Matter tabs with badges | — |

### Planned components (R1–R3)

| Component | For | Behaviour |
|---|---|---|
| `RoutePill` | Every model step | "This computer" (success) / "Cloud · redacted" (info) / "Blocked" (danger); hover shows model and redaction level |
| `CitationChip` | Answers, cells, drafts | `[n]`, focusable; click opens the source drawer at the passage |
| `ConfidenceMeter` | Cells, register | Three-step bar + %, warning tone under 0.6 |
| `SourceDrawer` | Everywhere | Right-hand drawer: document, passage with the cited span highlighted, open-file action |
| `ReviewGrid` | Review tables | Virtualised grid; sticky first column; cell states (pending, running, answered, not found, n/a, edited, stale, error); comments badge |
| `RunLog` | Workflows and agents | Ordered steps with RoutePill, duration, sources count, output link; expandable |
| `RedactionPreview` | Cloud steps | Side-by-side original vs redacted, placeholders highlighted |
| `ApprovalCard` | Approvals inbox | Proposal, diff or preview, approve / reject, undo after apply |
| `StepEditor` | Agent Builder | Step type, instructions, tools, output, optional toggle |

## 4. Patterns

- **Citations:** inline `[n]`. The source list sits beside the answer. The active citation is highlighted in both places. Uncited sentences get a dotted underline and the label "uncited".
- **Proposals:** anything that changes files or leaves the app is a card in Approvals. The primary action is "Approve". "Apply" is a separate step, and "Undo" is available afterwards.
- **Long jobs:** a progress bar with counts (`142 / 300 cells`). Jobs can be cancelled and resumed. A toast appears when they finish.
- **Empty states:** say what will appear and give the first action (e.g. "Add folder").
- **Errors:** say what happened, why, and what to do ("Scanned document: install Tesseract OCR to read it").
- **Routing transparency:** any screen that runs a model shows the RoutePill *before* running and records the route *after*.
- **Keyboard:** all actions are reachable by keyboard. `⌘/Ctrl+K` opens the command palette (planned). Grids support arrow keys and Enter to open the source.

## 5. Accessibility

- WCAG 2.1 AA contrast for text and pills in both themes.
- A visible focus ring (`--ring`) on all interactive elements.
- Pills and states have text, not colour alone. Icons have `aria-label`s.
- Grids expose row and column headers to screen readers. Streamed answers announce completion, not every token.
- Respect reduced-motion settings (no shimmer on skeletons).

## 6. Content style

- Plain English, sentence case, short sentences.
- Name the thing: "Local only", "Hybrid", "matter", "passage", "register".
- Numbers with units and dates in ISO or the user's locale. Never "soon" without a date.
- Never claim certainty the system lacks. Show confidence and citations instead of "accurate".
