//! What the CLI says when no AI provider is configured.
//!
//! Providers are configured with `aliv config set` (or the TUI's `/login`
//! prompt, which saves an API key for a known provider). An interactive run
//! with nothing configured proceeds and the TUI shows its own notice; a
//! non-interactive run stops here with directions instead.

use std::io::IsTerminal;

/// Reject a non-interactive run with nothing configured: there is no terminal
/// to show the setup notice in and nobody to act on it. No-op otherwise --
/// an interactive run with no provider proceeds and the TUI shows its own
/// notice instead.
pub fn reject_headless_without_provider(
    project_root: Option<&std::path::Path>,
) -> Result<(), String> {
    if super::providers::has_usable_provider(project_root) {
        return Ok(());
    }
    if !std::io::stdin().is_terminal() {
        return Err(headless_message());
    }
    Ok(())
}

/// Why we stopped when there is nobody at the keyboard to configure a provider.
fn headless_message() -> String {
    "no AI provider is configured. Set one with:\n  \
     aliv config set --provider <id> --api-key <key> --base-url <url>\n\
     Running a model locally? Point ALIV at the desktop app's API server:\n  \
     aliv config set --provider aliv --base-url http://localhost:1337/v1 --model <model>\n\
     A key can also be passed per run with --api-key or $ALIV_API_KEY."
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_headless_message_points_at_a_way_forward() {
        let message = headless_message();
        assert!(message.contains("aliv config set"), "{message}");
        assert!(message.contains("--api-key"), "{message}");
    }
}
