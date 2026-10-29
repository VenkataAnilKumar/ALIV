//! Meaning-based search next to the keyword index.
//!
//! The vault does not run an embedding model itself: the app embeds passages
//! with its local engine and hands the vectors over (`store_embeddings`), and
//! embeds each question before searching. Keeping the model outside means the
//! vault works the same with any engine, and with none: without a query
//! vector, search is keyword-only.
//!
//! Vectors are stored unit-length, so similarity is a dot product. A matter is
//! scanned in full; at 384 dimensions that is a few milliseconds per thousand
//! passages, well inside what one matter holds.

/// Weight of the keyword ranking in the fused score; the meaning ranking gets
/// the rest. Keyword matches stay slightly ahead because legal and finance
/// questions often quote a defined term exactly.
pub const KEYWORD_WEIGHT: f64 = 0.55;
/// Reciprocal-rank-fusion damping: a hit at rank r contributes w / (K + r).
pub const RRF_K: f64 = 60.0;
/// Each ranking contributes this many candidates per requested result.
pub const CANDIDATES_PER_RESULT: usize = 4;
/// Largest vector accepted. Common local embedders use 384 to 1024.
pub const MAX_DIM: usize = 4096;

/// Checks a vector and scales it to unit length. `None` for an empty, too
/// long, all-zero or non-finite vector, which would rank arbitrarily.
pub fn normalize(v: &[f32]) -> Option<Vec<f32>> {
    if v.is_empty() || v.len() > MAX_DIM || v.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let norm = v.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    if norm == 0.0 {
        return None;
    }
    Some(v.iter().map(|x| (*x as f64 / norm) as f32).collect())
}

pub fn to_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

pub fn from_bytes(b: &[u8]) -> Vec<f32> {
    b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
}

/// Dot product of two unit vectors: their cosine similarity.
pub fn dot(a: &[f32], b: &[f32]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (*x as f64) * (*y as f64)).sum()
}

/// Fuses a keyword ranking and a meaning ranking (each best first) into one
/// list of ids, best first, with their fused scores.
pub fn fuse(keyword: &[i64], meaning: &[i64]) -> Vec<(i64, f64)> {
    let mut scores: Vec<(i64, f64)> = Vec::new();
    let mut add = |id: i64, s: f64| match scores.iter_mut().find(|(i, _)| *i == id) {
        Some(slot) => slot.1 += s,
        None => scores.push((id, s)),
    };
    for (rank, id) in keyword.iter().enumerate() {
        add(*id, KEYWORD_WEIGHT / (RRF_K + rank as f64 + 1.0));
    }
    for (rank, id) in meaning.iter().enumerate() {
        add(*id, (1.0 - KEYWORD_WEIGHT) / (RRF_K + rank as f64 + 1.0));
    }
    // Stable on ties, so equal scores keep keyword order.
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vectors_are_normalized_and_bad_ones_refused() {
        let v = normalize(&[3.0, 4.0]).unwrap();
        assert!((v[0] - 0.6).abs() < 1e-6 && (v[1] - 0.8).abs() < 1e-6);
        assert!(normalize(&[]).is_none());
        assert!(normalize(&[0.0, 0.0]).is_none());
        assert!(normalize(&[1.0, f32::NAN]).is_none());
        assert!(normalize(&vec![1.0; MAX_DIM + 1]).is_none());
    }

    #[test]
    fn bytes_round_trip() {
        let v = vec![0.25f32, -1.5, 3.0];
        assert_eq!(from_bytes(&to_bytes(&v)), v);
    }

    #[test]
    fn a_hit_in_both_rankings_beats_one_in_either() {
        let fused = fuse(&[1, 2, 3], &[4, 2, 5]);
        assert_eq!(fused[0].0, 2, "{fused:?}");
        let ids: Vec<i64> = fused.iter().map(|(i, _)| *i).collect();
        assert_eq!(ids.len(), 5);
        // Keyword-only and meaning-only hits at the same rank: keyword first.
        let pos = |id| ids.iter().position(|x| *x == id).unwrap();
        assert!(pos(1) < pos(4));
    }

    #[test]
    fn without_a_meaning_ranking_the_keyword_order_stands() {
        let fused = fuse(&[7, 3, 9], &[]);
        assert_eq!(fused.iter().map(|(i, _)| *i).collect::<Vec<_>>(), vec![7, 3, 9]);
    }
}
