# Phase 4 Defect Backlog

Prioritized by product impact and phase-exit risk.

## Critical
- [x] None open.

## High
- [ ] Ingestion benchmark full mode reused existing DB state and produced false gate failures. Fixed by resetting benchmark DB for non-incremental runs in benchmarks/run_ingestion_benchmark.py.

## Medium
- [ ] `cosine_similarity` in src-tauri/src/infra/db.rs is currently unused and should be removed or wired to retrieval path.
- [ ] Extraction benchmark has one remaining failed question (`Q037`) that should be root-caused and corrected.

## Low
- [ ] Consolidate legacy phase documents so status trackers are consistent across docs.
