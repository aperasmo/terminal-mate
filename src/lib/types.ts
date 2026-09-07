export type RuntimeType = "local" | "wsl" | "ssh" | "container" | "cloud";

export interface ExecutionProfile {
  hostOs: string;
  runtime: RuntimeType;
  runtimeName: string;
  targetOs: string;
  shell: string;
  workingDirectory: string;
  architecture: string;
  privilege: string;
}

export interface Workspace {
  id: string;
  name: string;
  path: string;
  profile: ExecutionProfile;
}

export type OutputStream = "stdout" | "stderr";

export interface SessionSummary {
  id: string;
  workspaceId: string;
  shell: string;
}

export interface SessionOutputEvent {
  sessionId: string;
  runId: string | null;
  command?: string | null;
  stream: OutputStream;
  chunk: string;
}

export interface SessionOutputLine {
  stream: OutputStream;
  chunk: string;
  receivedAtMs: number;
  runId: string | null;
  commandSeparator?: boolean;
  commandStart?: boolean;
}

export type RiskLevel = "safe" | "caution" | "highRisk" | "blocked";

export interface RiskDecision {
  level: RiskLevel;
  reason: string;
  editablePath: string | null;
  externalTerminal: boolean;
}

export interface CommandRunSummary {
  id: string;
  sessionId: string;
  command: string;
  logPath: string;
  startedAtMs: number;
}

export interface CommandRunFinishedEvent {
  id: string;
  sessionId: string;
  exitCode: number;
  success: boolean;
  stoppedByUser: boolean;
  logPath: string;
  workingDirectory: string;
  startedAtMs: number;
  finishedAtMs: number;
  durationMs: number;
}

export type CommandRunStatus =
  "running" | "succeeded" | "failed" | "stopped" | "interrupted";

export interface CommandHistoryEntry {
  id: string;
  sessionId: string;
  request: string;
  command: string;
  riskLevel: RiskLevel;
  approval: "automatic" | "approved";
  status: CommandRunStatus;
  startedAt: string;
  finishedAt?: string;
  durationMs?: number;
  exitCode?: number;
  logPath: string;
}

export interface CommandFailureDiagnosis {
  kind: string;
  title: string;
  explanation: string;
  nextSteps: string[];
  suggestedRequest?: string;
  failedSubscriptionId?: string;
}

export interface AzureSubscription {
  name: string;
  id: string;
  tenantId: string;
  isDefault: boolean;
}

export interface CreatedVirtualMachine {
  vmName: string;
  resourceGroup: string;
}

export interface AzureVirtualMachine {
  name: string;
  resourceGroup: string;
  powerState: string | null;
}

export type ResolvedCommandSource = "intent" | "aiIntent" | "direct";

export interface ResolvedCommand {
  command: string;
  source: ResolvedCommandSource;
  note: string | null;
  executionMode: "execute" | "explainOnly";
  explanation: string | null;
}
