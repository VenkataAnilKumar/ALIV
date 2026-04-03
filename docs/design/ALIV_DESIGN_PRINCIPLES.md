# ALIV Design Principles and Standards

## 1. Core Principles
1. Local trust is a product feature.
2. Every answer should show evidence.
3. Every action should be reversible.
4. Unsafe states must be obvious.
5. Performance feedback should be immediate.

## 2. Decision Framework
When choosing between options:
1. Prefer higher safety over higher automation.
2. Prefer deterministic behavior over hidden heuristics.
3. Prefer simple defaults with expert override.
4. Prefer explicit confidence and citations over fluent output.

## 3. UX Writing Standards
1. Use clear and operational language.
2. Avoid vague confidence labels without numeric context.
3. Label risky actions explicitly.
4. Keep system messages action-oriented.

## 4. State Design Standards
Required states for all critical flows:
1. idle
2. running
3. blocked
4. degraded
5. failed
6. recovered

Each state must define:
1. user-visible message
2. machine reason
3. recovery action

## 5. Confidence and Evidence Standards
1. Each extracted field must include confidence value.
2. Each non-empty answer must include at least one citation.
3. Low-confidence outputs must not appear as final.
4. Contradictory evidence must be surfaced, not hidden.

## 6. Action Safety Standards
1. Preview-first default for rename and move.
2. Deterministic collision handling strategy.
3. Undo path required before live execution mode.
4. Path allowlist enforced for all file operations.

## 7. Performance Standards
1. Query p95 under 8 seconds target.
2. Ingestion runs should stream progress continuously.
3. UI interactions should remain responsive during background tasks.

## 8. Observability Standards
1. All critical actions emit structured events.
2. Every benchmark run is versioned and reproducible.
3. Failures are categorized with root-cause tags.

## 9. Versioning Standards
1. Version parser workers, extraction templates, and model profiles.
2. Store run metadata with versions for every benchmark result.
3. Do not compare benchmark runs across mismatched version sets.

## 10. Change Management
1. Any change affecting safety, confidence, or thresholds requires design note update.
2. Keep one source of truth in docs and update in same change set as code.
3. Re-run benchmark suite after any critical pipeline change.
