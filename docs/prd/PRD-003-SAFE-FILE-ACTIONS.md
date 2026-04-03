# PRD-003 Safe File Actions

## 1. Goal
Execute document rename and move workflows safely with preview, approval, and rollback controls.

## 2. User Story
As a user, I can let ALIV organize messy files while retaining control and the ability to undo mistakes.

## 3. Functional Requirements
1. Suggest standardized filenames using pattern:
- {DocumentType}_{VendorName}_{Date}
2. Suggest destination folders based on doc type and rules.
3. Preview-first action queue with before/after diff.
4. Approve single action or bulk action sets.
5. Collision handling with deterministic suffix rules.
6. Undo action support for every executed operation.
7. Action audit log with timestamp, reason, and source evidence.

## 4. Safety Requirements
1. Default mode is dry-run.
2. High-risk operations require explicit confirmation.
3. Undo must be tested for rename and move.
4. Failed action should not stop remaining safe actions unless configured.

## 5. Non-Functional Requirements
1. Real-time progress reporting for action batches.
2. Idempotent execution behavior for retries.
3. Error events logged with recovery guidance.

## 6. Acceptance Criteria
1. 100 dry-run recommendations evaluated with harmful action rate < 1%.
2. Collision logic prevents unintended overwrite in 100% of tested cases.
3. Undo operation success rate is 100% on test set.
4. Action log is complete and queryable for every executed action.

## 6.1 Harmful Action Taxonomy
1. Wrong destination folder for document class.
2. Wrong normalized filename values (document type, vendor, or date).
3. Any operation that risks overwrite or data loss.
4. Any irreversible operation without valid undo path.

## 6.2 Safety Blocking Rules
1. Any data-loss-risk action blocks phase exit immediately.
2. Harmful action rate is computed on approved dry-run recommendations before live execution mode.
3. Live execution mode remains disabled until two consecutive safety passes.

## 7. Telemetry
1. Recommended actions, approved actions, executed actions.
2. Harmful action count and type.
3. Undo usage and success rate.
4. Collision events and resolved strategy.

## 8. Test Cases
1. Duplicate target filenames.
2. Read-only destination folders.
3. Interrupted move operation.
4. Mixed file types in one batch.

## 9. Dependencies
1. PRD-001 for doc metadata.
2. PRD-002 for doc type and entity extraction quality.

## 10. Done Definition
PRD-003 is done when safety thresholds pass, undo is reliable, and audit logs are complete.
