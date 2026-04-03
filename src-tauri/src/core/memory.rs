use serde::{Deserialize, Serialize};

/// Granularity for document summary cache.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SummaryGranularity {
    Document,
    Section,
}

impl SummaryGranularity {
    pub fn as_str(&self) -> &'static str {
        match self {
            SummaryGranularity::Document => "document",
            SummaryGranularity::Section => "section",
        }
    }
}

/// Role in a conversation turn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TurnRole {
    User,
    Assistant,
}

impl TurnRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            TurnRole::User => "user",
            TurnRole::Assistant => "assistant",
        }
    }
}

/// Scope for a pinned memory fact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FactScope {
    Session,
    Week,
    Permanent,
}

impl FactScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            FactScope::Session => "session",
            FactScope::Week => "week",
            FactScope::Permanent => "permanent",
        }
    }
}

/// A single assembled context block for prompt injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBlock {
    pub block_type: String, // "memory_fact" | "history_turn" | "chunk" | "summary" | "instruction"
    pub content: String,
    pub token_estimate: usize,
    pub source_id: Option<String>, // chunk_id, fact_id, turn_id, or None
    pub relevance_score: Option<f32>,
}

/// Result of context assembly for a single query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledContext {
    pub conversation_id: String,
    pub query: String,
    pub blocks: Vec<ContextBlock>,
    pub total_token_estimate: usize,
    pub token_budget: usize,
    pub overflow_dropped: usize, // number of blocks dropped due to budget
}

/// Token budget configuration for context assembly.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    pub model_context_window: usize,
    pub reserved_output_tokens: usize,
    pub system_prompt_tokens: usize,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            model_context_window: 8192,
            reserved_output_tokens: 1024,
            system_prompt_tokens: 256,
        }
    }
}

impl TokenBudget {
    /// Total tokens available for context content (excludes output reserve and system prompt).
    pub fn available(&self) -> usize {
        self.model_context_window
            .saturating_sub(self.reserved_output_tokens)
            .saturating_sub(self.system_prompt_tokens)
    }

    /// Token allocation for each slot.
    pub fn allocation(&self) -> BudgetAllocation {
        let avail = self.available();
        BudgetAllocation {
            instructions: 256,
            memory_facts: avail / 10,
            conversation_history: avail / 5,
            retrieved_chunks: (avail * 6) / 10,
            overflow_summaries: avail / 10,
        }
    }
}

/// How the available token budget is split across context categories.
#[derive(Debug, Clone)]
pub struct BudgetAllocation {
    pub instructions: usize,
    pub memory_facts: usize,
    pub conversation_history: usize,
    pub retrieved_chunks: usize,
    pub overflow_summaries: usize,
}

/// Stateless memory service. All persistence is handled by the Database infra layer.
pub struct MemoryService;

impl MemoryService {
    /// Rough token estimate: 1 token ≈ 4 characters.
    /// Accurate within ~15% for English prose — sufficient for budget gating.
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() / 4).max(1)
    }

    /// Derive a conversation title from the first user query (first 80 chars).
    pub fn derive_title(first_query: &str) -> String {
        let trimmed = first_query.trim();
        if trimmed.len() <= 80 {
            trimmed.to_string()
        } else {
            let prefix: String = trimmed.chars().take(78).collect();
            format!("{}…", prefix)
        }
    }

    /// Assemble context blocks in priority order within the token budget.
    ///
    /// Assembly order (highest priority first):
    ///   1. Instructions (always included, fixed allocation)
    ///   2. Memory facts (relevant pinned facts, newest-first within allocation)
    ///   3. Conversation history (newest-first, stops at history allocation)
    ///   4. Retrieved chunks (highest-score-first, stops at chunk allocation)
    ///   5. Document summaries (used if chunks don't fill chunk allocation)
    ///
    /// Returns AssembledContext with overflow_dropped count for telemetry.
    pub fn assemble_context(
        query: &str,
        conversation_id: &str,
        instruction_block: &str,
        memory_fact_blocks: Vec<ContextBlock>,
        history_blocks: Vec<ContextBlock>,
        chunk_blocks: Vec<ContextBlock>,
        summary_blocks: Vec<ContextBlock>,
        budget: &TokenBudget,
    ) -> AssembledContext {
        let alloc = budget.allocation();
        let mut blocks: Vec<ContextBlock> = Vec::new();
        let mut total_tokens = 0usize;
        let mut overflow_dropped = 0usize;

        // 1. Instructions — always include
        let instr = ContextBlock {
            block_type: "instruction".into(),
            content: instruction_block.to_string(),
            token_estimate: Self::estimate_tokens(instruction_block).min(alloc.instructions),
            source_id: None,
            relevance_score: None,
        };
        total_tokens += instr.token_estimate;
        blocks.push(instr);

        // 2. Memory facts — fill up to memory_facts allocation
        let mut fact_tokens = 0usize;
        for fact in memory_fact_blocks {
            if fact_tokens + fact.token_estimate <= alloc.memory_facts {
                fact_tokens += fact.token_estimate;
                total_tokens += fact.token_estimate;
                blocks.push(fact);
            } else {
                overflow_dropped += 1;
            }
        }

        // 3. Conversation history — newest-first up to history allocation
        let mut history_tokens = 0usize;
        for turn in history_blocks {
            if history_tokens + turn.token_estimate <= alloc.conversation_history {
                history_tokens += turn.token_estimate;
                total_tokens += turn.token_estimate;
                blocks.push(turn);
            } else {
                overflow_dropped += 1;
            }
        }

        // 4. Retrieved chunks — highest-score-first up to chunk allocation
        let mut chunk_tokens = 0usize;
        for chunk in chunk_blocks {
            if chunk_tokens + chunk.token_estimate <= alloc.retrieved_chunks {
                chunk_tokens += chunk.token_estimate;
                total_tokens += chunk.token_estimate;
                blocks.push(chunk);
            } else {
                overflow_dropped += 1;
            }
        }

        // 5. Document summaries — fill remaining chunk budget if chunks didn't fill it
        let remaining_for_summaries = alloc
            .retrieved_chunks
            .saturating_sub(chunk_tokens)
            .min(alloc.overflow_summaries);
        let mut summary_tokens = 0usize;
        for summary in summary_blocks {
            if summary_tokens + summary.token_estimate <= remaining_for_summaries {
                summary_tokens += summary.token_estimate;
                total_tokens += summary.token_estimate;
                blocks.push(summary);
            } else {
                overflow_dropped += 1;
            }
        }

        AssembledContext {
            conversation_id: conversation_id.to_string(),
            query: query.to_string(),
            blocks,
            total_token_estimate: total_tokens,
            token_budget: budget.available(),
            overflow_dropped,
        }
    }

    /// Format assembled context blocks into a single prompt string for the LLM.
    ///
    /// Format:
    ///   [MEMORY]\n<facts>\n[HISTORY]\n<turns>\n[CONTEXT]\n<chunks>\n[QUESTION]\n<query>
    pub fn format_prompt(context: &AssembledContext) -> String {
        let mut facts = Vec::new();
        let mut history = Vec::new();
        let mut chunks = Vec::new();
        let mut instructions = Vec::new();

        for block in &context.blocks {
            match block.block_type.as_str() {
                "instruction" => instructions.push(block.content.clone()),
                "memory_fact" => facts.push(format!("• {}", block.content)),
                "history_turn" => history.push(block.content.clone()),
                "chunk" | "summary" => chunks.push(block.content.clone()),
                _ => {}
            }
        }

        let mut parts: Vec<String> = Vec::new();

        if !instructions.is_empty() {
            parts.push(instructions.join("\n"));
        }
        if !facts.is_empty() {
            parts.push(format!("[MEMORY]\n{}", facts.join("\n")));
        }
        if !history.is_empty() {
            parts.push(format!("[CONVERSATION HISTORY]\n{}", history.join("\n")));
        }
        if !chunks.is_empty() {
            parts.push(format!("[DOCUMENT CONTEXT]\n{}", chunks.join("\n\n---\n\n")));
        }

        parts.push(format!("[QUESTION]\n{}", context.query));

        parts.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens_short() {
        assert_eq!(MemoryService::estimate_tokens("hello"), 1);
    }

    #[test]
    fn test_estimate_tokens_long() {
        let text = "a".repeat(400);
        assert_eq!(MemoryService::estimate_tokens(&text), 100);
    }

    #[test]
    fn test_derive_title_short() {
        let title = MemoryService::derive_title("Find expiry dates");
        assert_eq!(title, "Find expiry dates");
    }

    #[test]
    fn test_derive_title_truncates() {
        let long = "a".repeat(100);
        let title = MemoryService::derive_title(&long);
        assert!(title.len() <= 81); // 79 chars + ellipsis char
        assert!(title.ends_with('…'));
    }

    #[test]
    fn test_budget_available() {
        let budget = TokenBudget::default();
        assert_eq!(budget.available(), 8192 - 1024 - 256);
    }

    #[test]
    fn test_assemble_context_respects_budget() {
        let budget = TokenBudget {
            model_context_window: 200,
            reserved_output_tokens: 50,
            system_prompt_tokens: 10,
        };
        // 140 tokens available

        let big_chunks: Vec<ContextBlock> = (0..20)
            .map(|i| ContextBlock {
                block_type: "chunk".into(),
                content: "x".repeat(400), // 100 tokens each
                token_estimate: 100,
                source_id: Some(format!("chunk-{i}")),
                relevance_score: Some(0.9),
            })
            .collect();

        let ctx = MemoryService::assemble_context(
            "test query",
            "conv-1",
            "Answer accurately.",
            vec![],
            vec![],
            big_chunks,
            vec![],
            &budget,
        );

        assert!(ctx.total_token_estimate <= budget.available() + 256 /* instr */);
        assert!(ctx.overflow_dropped > 0);
    }

    #[test]
    fn test_format_prompt_includes_sections() {
        let ctx = AssembledContext {
            conversation_id: "c1".into(),
            query: "What are the penalties?".into(),
            blocks: vec![
                ContextBlock {
                    block_type: "instruction".into(),
                    content: "Be concise.".into(),
                    token_estimate: 5,
                    source_id: None,
                    relevance_score: None,
                },
                ContextBlock {
                    block_type: "memory_fact".into(),
                    content: "Vendor is Acme Corp.".into(),
                    token_estimate: 5,
                    source_id: Some("fact-1".into()),
                    relevance_score: None,
                },
                ContextBlock {
                    block_type: "chunk".into(),
                    content: "Penalty clause: 10% per month.".into(),
                    token_estimate: 8,
                    source_id: Some("chunk-1".into()),
                    relevance_score: Some(0.95),
                },
            ],
            total_token_estimate: 18,
            token_budget: 512,
            overflow_dropped: 0,
        };

        let prompt = MemoryService::format_prompt(&ctx);
        assert!(prompt.contains("[MEMORY]"));
        assert!(prompt.contains("[DOCUMENT CONTEXT]"));
        assert!(prompt.contains("[QUESTION]"));
        assert!(prompt.contains("What are the penalties?"));
    }
}
