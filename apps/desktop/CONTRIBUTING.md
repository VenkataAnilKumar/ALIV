# Contributing to ALIV Desktop

1. Read [README.md](README.md) for the build, test and layout overview.
2. Branch from `main`. Keep each change focused.
3. Before opening a pull request, run the checks for what you touched:
   - Frontend: `yarn test:all`, plus `yarn workspace @aliv/web-app tsc -b` and `yarn workspace @aliv/web-app lint`.
   - Rust: `cargo test --lib` in `src-tauri/` and `cargo test` in any plugin you changed. Also run `cargo clippy --all-targets`.
4. Behavior that touches client documents (ingestion, file actions, model calls) needs tests. It must keep the approval gate, the matter policy and the audit log intact.
5. Describe the change and how you verified it in the pull request.

Report bugs and feature requests at https://github.com/VenkataAnilKumar/ALIV/issues.
