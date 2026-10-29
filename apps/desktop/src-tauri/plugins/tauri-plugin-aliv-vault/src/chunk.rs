//! Splits document text into overlapping passages for search and citation.

/// A passage and its byte span in the source text.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub ordinal: usize,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

pub const TARGET_CHARS: usize = 900;
const OVERLAP_CHARS: usize = 150;

/// Greedy paragraph packing: paragraphs are joined until the target size is
/// reached; a paragraph longer than the target is cut at whitespace. Each
/// chunk after the first starts `OVERLAP_CHARS` before the previous end so a
/// clause straddling a boundary is still found whole in one of them.
pub fn chunk_text(text: &str) -> Vec<Chunk> {
    let text_len = text.len();
    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < text_len {
        let hard_end = floor_char_boundary(text, (start + TARGET_CHARS).min(text_len));
        let end = if hard_end >= text_len {
            text_len
        } else {
            best_break(text, start, hard_end)
        };
        let slice = text[start..end].trim();
        if !slice.is_empty() {
            chunks.push(Chunk {
                ordinal: chunks.len(),
                start,
                end,
                text: slice.to_string(),
            });
        }
        if end >= text_len {
            break;
        }
        let next = floor_char_boundary(text, end.saturating_sub(OVERLAP_CHARS));
        start = if next <= start { end } else { next };
    }
    chunks
}

/// Index of the chunk containing byte `offset`, preferring the earliest one.
pub fn chunk_for_offset(chunks: &[Chunk], offset: usize) -> Option<usize> {
    chunks
        .iter()
        .find(|c| offset >= c.start && offset < c.end)
        .map(|c| c.ordinal)
}

fn best_break(text: &str, start: usize, hard_end: usize) -> usize {
    let window = &text[start..hard_end];
    let min = window.len() / 2;
    for sep in ["\n\n", "\n", ". ", " "] {
        if let Some(pos) = window.rfind(sep) {
            if pos >= min {
                return start + pos + sep.len();
            }
        }
    }
    hard_end
}

fn floor_char_boundary(text: &str, mut idx: usize) -> usize {
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_is_one_chunk() {
        let chunks = chunk_text("Hello world.");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "Hello world.");
    }

    #[test]
    fn long_text_splits_with_overlap_and_covers_everything() {
        let para = "The tenant shall pay rent monthly. ".repeat(80);
        let chunks = chunk_text(&para);
        assert!(chunks.len() > 2);
        for w in chunks.windows(2) {
            assert!(w[1].start < w[0].end, "chunks overlap");
        }
        assert_eq!(chunks.last().unwrap().end, para.len());
        assert!(chunks.iter().all(|c| c.text.len() <= TARGET_CHARS));
    }

    #[test]
    fn handles_multibyte_text() {
        let text = "Montant dû: 1 000 € — échéance prochaine. ".repeat(60);
        let chunks = chunk_text(&text);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn offset_maps_to_chunk() {
        let text = "a ".repeat(2000);
        let chunks = chunk_text(&text);
        assert_eq!(chunk_for_offset(&chunks, 0), Some(0));
        assert_eq!(chunk_for_offset(&chunks, text.len() - 1), Some(chunks.len() - 1));
    }
}
