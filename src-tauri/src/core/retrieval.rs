use std::collections::HashMap;

use crate::models::contracts::RetrievalHit;

#[derive(Debug, Default)]
pub struct RetrievalService;

impl RetrievalService {
    pub fn search_terms(&self, query: &str) -> Vec<String> {
        let mut terms = Vec::new();
        let normalized = query.trim().to_ascii_lowercase();
        if !normalized.is_empty() {
            terms.push(normalized.clone());
        }

        for term in normalized.split_whitespace() {
            if term.len() < 2 || terms.iter().any(|existing| existing == term) {
                continue;
            }
            terms.push(term.to_string());
        }

        terms
    }

    pub fn score_content(&self, content: &str, terms: &[String]) -> i64 {
        let lowered = content.to_ascii_lowercase();
        let mut score = 0_i64;

        for (index, term) in terms.iter().enumerate() {
            if term.is_empty() {
                continue;
            }

            let weight = if index == 0 { 10 } else { 3 };
            let matches = lowered.match_indices(term).count() as i64;
            score += matches * weight;
        }

        score
    }

    pub fn build_preview(&self, content: &str, terms: &[String]) -> String {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let lowered = trimmed.to_ascii_lowercase();
        let first_index = terms
            .iter()
            .filter_map(|term| lowered.find(term))
            .min()
            .unwrap_or(0);

        let preview_radius = 140;
        let start = first_index.saturating_sub(preview_radius / 2);
        let snippet = trimmed.chars().skip(start).take(preview_radius).collect::<String>();

        if start > 0 && snippet.len() < trimmed.len() {
            format!("...{}...", snippet.trim())
        } else if start > 0 {
            format!("...{}", snippet.trim())
        } else if snippet.len() < trimmed.len() {
            format!("{}...", snippet.trim())
        } else {
            snippet.trim().to_string()
        }
    }

    pub fn merge_hybrid_hits(
        &self,
        lexical_hits: Vec<RetrievalHit>,
        semantic_hits: Vec<RetrievalHit>,
        top_k: usize,
    ) -> Vec<RetrievalHit> {
        let mut by_key: HashMap<String, RetrievalHit> = HashMap::new();
        let mut lexical_rank: HashMap<String, usize> = HashMap::new();
        let mut semantic_rank: HashMap<String, usize> = HashMap::new();

        for (idx, hit) in lexical_hits.into_iter().enumerate() {
            let key = format!("{}#{}", hit.file_id, hit.chunk_index);
            lexical_rank.insert(key.clone(), idx + 1);
            by_key.insert(key, hit);
        }

        for (idx, hit) in semantic_hits.into_iter().enumerate() {
            let key = format!("{}#{}", hit.file_id, hit.chunk_index);
            semantic_rank.insert(key.clone(), idx + 1);
            by_key.entry(key).or_insert(hit);
        }

        let mut scored: Vec<(RetrievalHit, f64)> = by_key
            .into_iter()
            .map(|(key, mut hit)| {
                let rank_lex = lexical_rank.get(&key).copied();
                let rank_sem = semantic_rank.get(&key).copied();

                // Reciprocal-rank-fusion with lexical preference for tie stability.
                let lex_rrf = rank_lex.map(|r| 1.0 / (60.0 + r as f64)).unwrap_or(0.0);
                let sem_rrf = rank_sem.map(|r| 1.0 / (60.0 + r as f64)).unwrap_or(0.0);
                let fusion = (0.55 * lex_rrf) + (0.45 * sem_rrf);

                // Expose fused score in existing i64 field for downstream sorting/telemetry.
                hit.score = (fusion * 1_000_000.0) as i64;
                (hit, fusion)
            })
            .collect();

        scored.sort_by(|(left_hit, left), (right_hit, right)| {
            right
                .partial_cmp(left)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right_hit.score.cmp(&left_hit.score))
                .then_with(|| left_hit.source_path.cmp(&right_hit.source_path))
                .then_with(|| left_hit.chunk_index.cmp(&right_hit.chunk_index))
        });

        let mut merged: Vec<RetrievalHit> = scored.into_iter().map(|(hit, _)| hit).collect();
        merged.truncate(std::cmp::max(top_k, 1));
        merged
    }
}
