# PRD-002 Retrieval and Extraction

## 1. Goal
Answer natural-language questions over local corpus with high relevance and precise structured extraction.

## 2. User Story
As a user, I can ask ALIV contract and invoice questions and receive cited answers and structured outputs I can trust.

## 3. Functional Requirements
1. Query understanding layer that builds hybrid retrieval plan:
- Metadata filter query (SQLite)
- Semantic vector query (sqlite-vec)
2. Ranking and evidence selection for top chunks.
3. Citation binding from output to source document and chunk.
4. Structured extraction templates for:
- Expiry date
- Vendor name
- Penalty clause text
5. Output formatters:
- Human-readable answer with citations
- Structured JSON output
- CSV or Excel export trigger interface

## 4. Non-Functional Requirements
1. Deterministic evaluation mode for benchmark runs.
2. Query p95 latency target under 8 seconds on reference machine.
3. Graceful degradation when no confident answer is found.

## 5. Acceptance Criteria
1. Top-5 retrieval relevance >= 85% on 50-question benchmark.
2. Extraction precision >= 95% on critical fields.
3. Every non-empty answer includes at least one valid citation.
4. No-citation answers are automatically marked low confidence.

## 5.1 Metric Calculation Rules
1. Retrieval relevance (top-5) passes for a query when at least one gold evidence chunk is present in top-5 results.
2. Extraction precision is computed per field and macro-averaged across expiry date, vendor name, and penalty clause.
3. Benchmarks with missing gold label for a field are excluded from that field's precision denominator.

## 6. Confidence and Guardrails
1. Confidence score per extracted field.
2. Low-confidence threshold triggers review flag.
3. Contradictory evidence from multiple documents must be surfaced.
4. Responses with confidence below threshold must not be marked as final answer.

## 7. Telemetry
1. Query latency p50 and p95.
2. Retrieval hit rate and citation count.
3. Field-level precision, failure class, and confidence distribution.

## 8. Test Cases
1. Ambiguous vendor names.
2. Multiple contracts with similar terms.
3. Missing expiry date in source.
4. OCR-noisy penalty clause extraction.

## 9. Dependencies
1. PRD-001 indexed corpus and metadata quality.
2. Prompt templates for extraction and citation generation.

## 10. Done Definition
PRD-002 is done when retrieval and extraction thresholds pass and low-confidence behavior is reliable.
