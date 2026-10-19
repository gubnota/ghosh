import type { RunReport, RunRequest } from "../generated/contracts";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { NodeConnectionRequest, NodeConnectionPreview, NodeConnectionCommit, NodeConnectionResult, NodeTerminalRequest } from "../generated/contracts";
import type { TerminalSession, TerminalPacket, TerminalSize } from "../generated/contracts";
import type { KeyGenerateRequest, KeyRelinkRequest, WorkspaceKeyRequest, WorkspaceKeyPreview, WorkspaceKeyCommit, NodeIdentityRequest, IdentityPreview, KeysSnapshot, KeyImportRequest, KeyImportPreview, KeyCommitRequest, KeyRegistryRequest } from "../generated/contracts";
import type { Node, NodeMutationRequest, NodeCommitRequest, NodeChangePreview, AppError, ApprovalDecisionRequest, OperationsSnapshot, CreateRequest, RecentWorkspace, RenameRequest, RevisionRequest, TrustRequest, WorkspaceEvent, WorkspaceSnapshot } from "../generated/contracts";
import type { AutomationSummary, AutomationDocument, AutomationReadRequest, AutomationMutation, AutomationPreview, AutomationCommit } from "../generated/contracts";
export const desktop = isTauri();
export const api = {
  runStart: (request: RunRequest) => invoke<RunReport>("run_start", { request }),
  runs: (request: RevisionRequest) => invoke<RunReport[]>("runs_list", { request }),
  runStop: (id: string) => invoke<void>("run_stop", { id }),
  automations: (request: RevisionRequest) => invoke<AutomationSummary[]>("automations_list", { request }),
  automationRead: (request: AutomationReadRequest) => invoke<AutomationDocument>("automation_read", { request }),
  automationPreview: (request: AutomationMutation) => invoke<AutomationPreview>("automation_preview", { request }),
  automationCommit: (request: AutomationCommit) => invoke<WorkspaceSnapshot>("automation_commit", { request }),
  connectionPreview: (request: NodeConnectionRequest) => invoke<NodeConnectionPreview>("node_connection_preview", { request }),
  connectionTest: (request: NodeConnectionCommit) => invoke<NodeConnectionResult>("node_connection_test", { request }),
  terminalOpenNode: (request: NodeTerminalRequest) => invoke<TerminalSession>("terminal_open_node", { request }),
  terminalOpen: (dimensions: TerminalSize) => invoke<TerminalSession>("terminal_open_local", { dimensions }),
  terminalPoll: (id: string, ack: number) => invoke<TerminalPacket>("terminal_poll", { id, ack }),
  terminalInput: (id: string, bytes: number[]) => invoke<void>("terminal_input", { id, bytes }),
  terminalResize: (id: string, dimensions: TerminalSize) => invoke<void>("terminal_resize", { id, dimensions }),
  terminalClose: (id: string) => invoke<void>("terminal_close", { id }),
  workspaceKey: (request: RevisionRequest) => invoke<string | null>("workspace_key", { request }),
  previewWorkspaceKey: (request: WorkspaceKeyRequest) => invoke<WorkspaceKeyPreview>("workspace_key_preview", { request }),
  commitWorkspaceKey: (request: WorkspaceKeyCommit) => invoke<WorkspaceSnapshot>("workspace_key_commit", { request }),
  identity: (request: NodeIdentityRequest) => invoke<IdentityPreview>("node_identity", { request }),
  generateKey: (request: KeyGenerateRequest) => invoke<KeysSnapshot>("keys_generate", { request }),
  relinkKey: (request: KeyRelinkRequest) => invoke<KeysSnapshot>("keys_relink", { request }),
  keys: () => invoke<KeysSnapshot>("keys_list"),
  previewKey: (request: KeyImportRequest) => invoke<KeyImportPreview>("keys_preview", { request }),
  commitKey: (request: KeyCommitRequest) => invoke<KeysSnapshot>("keys_commit", { request }),
  defaultKey: (request: KeyRegistryRequest) => invoke<KeysSnapshot>("keys_default", { request }),
  forgetKey: (request: KeyRegistryRequest) => invoke<KeysSnapshot>("keys_forget", { request }),
  pickKey: () => invoke<string | null>("keys_pick"),
  nodes: (request: RevisionRequest) => invoke<Node[]>("nodes_list", { request }),
  previewNode: (request: NodeMutationRequest) => invoke<NodeChangePreview>("nodes_preview", { request }),
  commitNode: (request: NodeCommitRequest) => invoke<WorkspaceSnapshot>("nodes_commit", { request }),
  operations: () => invoke<OperationsSnapshot>("operations_snapshot"),
  cancelJob: (id: string) => invoke<void>("jobs_cancel", { id }),
  decide: (request: ApprovalDecisionRequest) => invoke<void>("approvals_decide", { request }),
  onOperations: (handler: () => void) => listen("operations-changed", handler),
  startup: () => invoke<WorkspaceSnapshot | null>("workspace_startup"),
  close: (request: RevisionRequest) => invoke<void>("workspace_close", { request }),
  forgetRecent: (path: string | null) => invoke<RecentWorkspace[]>("workspace_forget_recent", { path }),
  open: (path: string) => invoke<WorkspaceSnapshot>("workspace_open", { path }),
  create: (request: CreateRequest) => invoke<WorkspaceSnapshot>("workspace_create", { request }),
  snapshot: () => invoke<WorkspaceSnapshot | null>("workspace_snapshot"),
  rename: (request: RenameRequest) => invoke<WorkspaceSnapshot>("workspace_rename", { request }),
  reload: (request: RevisionRequest) => invoke<WorkspaceSnapshot>("workspace_reload", { request }),
  trust: (request: TrustRequest) => invoke<WorkspaceSnapshot>("workspace_trust", { request }),
  recent: () => invoke<RecentWorkspace[]>("workspace_recent"),
  pending: () => invoke<string | null>("workspace_pending"),
  pick: (kind: "workspace" | "parent") => invoke<string | null>("workspace_pick", { kind }),
  onFiles: (handler: (event: WorkspaceEvent) => void) => listen<WorkspaceEvent>("workspace-files-changed", event => handler(event.payload)),
  onOpen: (handler: () => void) => listen("workspace-open-requested", handler),
};
export function errorMessage(error: unknown): string {
  if (typeof error === "object" && error !== null && "code" in error && "message" in error) return (error as AppError).message;
  return "Ghosh could not complete the operation. Try reopening the workspace.";
}
