# Phase 6 Release Checklist

## Packaging

- Run `npm run build` from the repo root.
- Run `cargo build` from `src-tauri/`.
- Run `cargo tauri build` and confirm a bundle artifact is produced under `src-tauri/target/release/bundle/`.

## Critical Paths

- Launch the desktop app and confirm the shell renders without blank screens.
- Confirm the onboarding modal appears on a first-run profile with no registered workspace.
- Register a workspace from onboarding and verify the workspace count updates.
- Confirm Ollama status reflects the local runtime state in the shell and onboarding step 3.
- Generate a diagnostics report and confirm raw JSON, stat cards, and log tail populate.
- Export the diagnostics report and verify the downloaded file opens as JSON.
- Trigger a controlled React crash in development and confirm the error boundary recovery screen appears.

## Operational Validation

- Run ingestion on a registered workspace and confirm success plus new log lines.
- Generate action recommendations and confirm preview queue counts refresh.
- Execute and undo at least one action batch in a safe test workspace.
- Load benchmark metrics from an existing results file and confirm the page renders without parse failures.

## Release Notes

- Review `CHANGELOG.md` for accuracy against the current build.
- Record any platform-specific installer caveats before external distribution.