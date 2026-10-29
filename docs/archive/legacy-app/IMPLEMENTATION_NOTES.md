# Implementation Notes

## Current Scaffold
The repository now includes the initial structural scaffold for:
- Rust and Tauri commands/core/infra/models
- Python worker runtime and adapters
- Database migrations and schema docs

## Immediate Next Step
Implement section 1.2 command handlers and section 3.1 migrations together so the command surface is backed by real persistence as early as possible.

## Constraints
- Local-only processing by default
- Preview-first file actions until safety thresholds pass
- Benchmark thresholds from `docs/prd/PRD-000-MASTER.md` are the source of truth
