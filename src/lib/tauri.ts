import { invoke } from '@tauri-apps/api/core'
import type * as T from '../types/contracts'
import { parseIngestionTelemetry, parseRuntimeSnapshot } from './parsers'

export const registerWorkspace = (req: T.WorkspaceFolderInput) =>
  invoke<T.CommandAck>('register_workspace_folder', { input: req })

export const getRuntimeSnapshot = async () => {
  const ack = await invoke<T.CommandAck>('get_runtime_status')
  return parseRuntimeSnapshot(ack.detail)
}

export const runIngestion = (req: T.IngestionRequest) =>
  invoke<T.CommandAck>('run_ingestion', { request: req })

export const getIngestionTelemetry = async () => {
  const ack = await invoke<T.CommandAck>('get_ingestion_telemetry')
  return parseIngestionTelemetry(ack.detail)
}

export const retrieveAndAssemble = (req: T.RetrievalRequest) =>
  invoke<T.RetrievalResponse>('run_retrieval_query', { request: req })

export const runExtraction = (req: T.ExtractionRequest) =>
  invoke<T.CommandAck>('run_extraction', { request: req })

export const generateActionRecommendations = (req: T.ActionRecommendationRequest) =>
  invoke<T.ActionPreviewList>('generate_action_recommendations', { request: req })

export const getActionPreviewQueue = () =>
  invoke<T.ActionPreviewList>('get_action_preview_queue')

export const evaluateActionRisk = (req: T.ActionRiskRequest) =>
  invoke<T.ActionPreviewList>('evaluate_action_risk', { request: req })

export const approveActions = (req: T.ActionApprovalRequest) =>
  invoke<T.CommandAck>('approve_actions', { request: req })

export const executeActions = (req: T.ActionExecutionRequest) =>
  invoke<T.ActionExecutionResult>('execute_actions', { request: req })

export const undoActionBatch = (req: T.UndoBatchRequest) =>
  invoke<T.UndoResult>('undo_action_batch', { request: req })

export const getActionAuditLog = (req: T.AuditLogRequest) =>
  invoke<T.AuditLogResponse>('get_action_audit_log', { request: req })

export const startConversation = (req: T.StartConversationRequest) =>
  invoke<T.ConversationStub>('start_conversation', { request: req })

export const listConversations = (req: T.ListConversationsRequest) =>
  invoke<T.ConversationStub[]>('list_conversations', { request: req })

export const getConversation = (req: T.GetConversationRequest) =>
  invoke<T.ConversationTurnView[]>('get_conversation', { request: req })

export const saveMemoryFact = (req: T.SaveMemoryFactRequest) =>
  invoke<T.CommandAck>('save_memory_fact', { request: req })

export const listMemoryFacts = (req: T.ListConversationsRequest) =>
  invoke<T.MemoryFactView[]>('list_memory_facts', { request: req })

export const deleteMemoryFact = (req: T.DeleteMemoryFactRequest) =>
  invoke<T.CommandAck>('delete_memory_fact', { request: req })

export const buildLongContext = (req: T.BuildLongContextRequest) =>
  invoke<T.LongContextResponse>('build_long_context', { request: req })

export const runBenchmarkSuite = (req: T.BenchmarkRequest) =>
  invoke<T.CommandAck>('run_benchmark_suite', { request: req })

export const getBenchmarkRunMetrics = (req: T.RunMetricsRequest) =>
  invoke<T.CommandAck>('get_run_metrics', { request: req })
