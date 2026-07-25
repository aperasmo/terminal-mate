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
  stream: OutputStream;
  chunk: string;
}

export interface SessionOutputLine {
  stream: OutputStream;
  chunk: string;
}

export type RiskLevel = "safe" | "caution" | "highRisk" | "blocked";

export interface RiskDecision {
  level: RiskLevel;
  reason: string;
}

