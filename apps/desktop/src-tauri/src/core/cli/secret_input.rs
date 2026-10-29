//! Masked secret entry for the TUI's `/login` overlay.
//!
//! The value is never echoed verbatim; callers render [`mask`] instead, so the
//! user gets proof that a keystroke or paste landed without the key reaching the
//! screen or the scrollback.

/// Longest run of mask characters rendered. Bounds the field so a long paste
/// cannot overflow a TUI box or wrap a terminal line, and hides the key's exact
/// length from anyone reading over a shoulder.
const MASK_MAX: usize = 32;

/// Bounded stand-in for a secret of `char_count` characters.
pub fn mask(char_count: usize) -> String {
    "*".repeat(char_count.min(MASK_MAX))
}

/// Plain text from the OS clipboard, for the Ctrl-V fallback below.
pub fn clipboard_text() -> Result<String, String> {
    arboard::Clipboard::new()
        .and_then(|mut clip| clip.get_text())
        .map_err(|e| e.to_string())
}

/// Whitespace terminals add around a paste is dropped; interior whitespace is
/// kept so `sanitize_key` can reject a mis-paste with a clear reason rather than
/// silently mangling the key.
pub fn pasted(text: &str) -> &str {
    text.trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_is_one_star_per_char_up_to_the_cap() {
        assert_eq!(mask(0), "");
        assert_eq!(mask(9), "*********");
        assert_eq!(mask(MASK_MAX).chars().count(), MASK_MAX);
        // A long key must not leak its length or overflow the field.
        assert_eq!(mask(4096).chars().count(), MASK_MAX);
    }

    #[test]
    fn interior_whitespace_survives_a_paste_so_it_can_be_rejected() {
        assert_eq!(pasted(" sk-a b \n"), "sk-a b");
    }
}
