import { HISTORY_STORAGE_LIMIT } from "./runHistory";
import type {
  CommandHistoryEntry,
  CommandRunStatus,
  RiskLevel,
  Workspace,
} from "./types";

const WORKSPACES_KEY = "terminal-mate.workspaces.v1";
const ACTIVE_WORKSPACE_KEY = "terminal-mate.active-workspace.v1";
const COMMAND_HISTORY_KEY = "terminal-mate.command-history.v1";
const DIRECTORY_HISTORY_KEY = "terminal-mate.directory-history.v1";

export interface StoredDirectoryHistory {
  entries: string[];
  index: number;
}
const RUN_STATUSES = new Set<CommandRunStatus>([
  "running",
  "succeeded",
  "failed",
  "stopped",
  "interrupted",
]);
const RISK_LEVELS = new Set<RiskLevel>([
  "safe",
  "caution",
  "highRisk",
  "blocked",
]);

export function loadWorkspaces(): Workspace[] {
  try {
    const value = localStorage.getItem(WORKSPACES_KEY);
    if (!value) {
      return [];
    }

    const parsed: unknown = JSON.parse(value);
    return Array.isArray(parsed)
      ? (parsed as Workspace[]).map(normalizeStoredWorkspace)
      : [];
  } catch {
    return [];
  }
}

function normalizeStoredWorkspace(workspace: Workspace): Workspace {
  if (!isWindowsPath(workspace.path)) {
    return workspace;
  }

  const hasValidWslProfile =
    workspace.profile.runtime === "wsl" &&
    workspace.profile.shell.toLowerCase() === "bash" &&
    workspace.profile.workingDirectory.startsWith("/");
  if (hasValidWslProfile) {
    return workspace;
  }

  return {
    ...workspace,
    profile: {
      ...workspace.profile,
      hostOs: "windows",
      runtime: "local",
      runtimeName: "Windows",
      targetOs: "windows",
      shell: "powershell",
      workingDirectory: workspace.path,
      privilege: "standard-user",
    },
  };
}

function isWindowsPath(path: string): boolean {
  return /^(?:\\\\\?\\)?[a-z]:[\\/]/i.test(path);
}

export function saveWorkspaces(workspaces: Workspace[]): void {
  localStorage.setItem(WORKSPACES_KEY, JSON.stringify(workspaces));
}

export function loadActiveWorkspaceId(): string | null {
  return localStorage.getItem(ACTIVE_WORKSPACE_KEY);
}

export function saveActiveWorkspaceId(workspaceId: string | null): void {
  if (workspaceId) {
    localStorage.setItem(ACTIVE_WORKSPACE_KEY, workspaceId);
  } else {
    localStorage.removeItem(ACTIVE_WORKSPACE_KEY);
  }
}

export function loadCommandHistory(): Record<string, CommandHistoryEntry[]> {
  try {
    const value = localStorage.getItem(COMMAND_HISTORY_KEY);
    if (!value) {
      return {};
    }

    const parsed: unknown = JSON.parse(value);
    if (!isRecord(parsed)) {
      return {};
    }

    return Object.fromEntries(
      Object.entries(parsed).flatMap(([workspaceId, entries]) => {
        if (!Array.isArray(entries)) {
          return [];
        }

        const validEntries = entries
          .filter(isCommandHistoryEntry)
          .map(markInterruptedRun)
          .slice(0, HISTORY_STORAGE_LIMIT);

        return validEntries.length > 0
          ? ([[workspaceId, validEntries]] as const)
          : [];
      }),
    );
  } catch {
    return {};
  }
}

export function saveCommandHistory(
  historyByWorkspace: Record<string, CommandHistoryEntry[]>,
): void {
  try {
    const limited = Object.fromEntries(
      Object.entries(historyByWorkspace)
        .map(([workspaceId, entries]) => [
          workspaceId,
          entries.slice(0, HISTORY_STORAGE_LIMIT),
        ])
        .filter(([, entries]) => entries.length > 0),
    );
    localStorage.setItem(COMMAND_HISTORY_KEY, JSON.stringify(limited));
  } catch {
    // History persistence must never prevent command execution.
  }
}

export function loadDirectoryHistory(): Record<string, StoredDirectoryHistory> {
  try {
    const value = localStorage.getItem(DIRECTORY_HISTORY_KEY);
    if (!value) {
      return {};
    }

    const parsed: unknown = JSON.parse(value);
    if (!isRecord(parsed)) {
      return {};
    }

    return Object.fromEntries(
      Object.entries(parsed).flatMap(([key, history]) => {
        if (!isRecord(history) || !Array.isArray(history.entries)) {
          return [];
        }
        const entries = history.entries.filter(
          (entry): entry is string => typeof entry === "string" && entry.length > 0,
        );
        if (entries.length === 0 || typeof history.index !== "number") {
          return [];
        }
        const index = Math.min(
          Math.max(0, Math.trunc(history.index)),
          entries.length - 1,
        );
        return [[key, { entries, index }] as const];
      }),
    );
  } catch {
    return {};
  }
}

export function saveDirectoryHistory(
  history: Record<string, StoredDirectoryHistory>,
): void {
  try {
    localStorage.setItem(DIRECTORY_HISTORY_KEY, JSON.stringify(history));
  } catch {
    // Directory navigation must keep working when browser storage is unavailable.
  }
}

function isCommandHistoryEntry(value: unknown): value is CommandHistoryEntry {
  if (!isRecord(value)) {
    return false;
  }

  return (
    typeof value.id === "string" &&
    typeof value.sessionId === "string" &&
    typeof value.request === "string" &&
    typeof value.command === "string" &&
    typeof value.riskLevel === "string" &&
    RISK_LEVELS.has(value.riskLevel as RiskLevel) &&
    (value.approval === "automatic" || value.approval === "approved") &&
    typeof value.status === "string" &&
    RUN_STATUSES.has(value.status as CommandRunStatus) &&
    typeof value.startedAt === "string" &&
    typeof value.logPath === "string" &&
    (value.finishedAt === undefined || typeof value.finishedAt === "string") &&
    (value.durationMs === undefined || typeof value.durationMs === "number") &&
    (value.exitCode === undefined || typeof value.exitCode === "number")
  );
}

function markInterruptedRun(entry: CommandHistoryEntry): CommandHistoryEntry {
  return entry.status === "running"
    ? { ...entry, status: "interrupted" }
    : entry;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
