# Changelog

## 0.1.0 - Release Candidate

- Added the first full desktop workbench flow across workspace registration, ingestion, retrieval, extraction, safe action previews, memory, and benchmark inspection.
- Added a richer 2026-style operator UI with a refreshed shell, glass panels, stronger information hierarchy, and route-level work surfaces.
- Added structured desktop logging, a diagnostics export flow, a first-run onboarding modal, and a frontend error boundary for release support.
- Enabled Tauri bundling metadata and installer targets for Windows, macOS, and Linux development packaging.

### Known Limitations

- Ollama availability is still assumed to be local and operator-managed; ALIV reports reachability but does not install or launch Ollama.
- Bundle signing, notarization, and production artwork are not configured for public distribution yet.
- Diagnostics export currently captures the local log tail and runtime/database state, but it does not bundle full worker subprocess logs.

### Upgrade Notes

- Existing local databases remain compatible; Phase 4 migration tracking remains in place via `schema_migrations`.
- The new onboarding modal only appears once per machine profile and can be reset by removing the `aliv_onboarded` key from local storage.