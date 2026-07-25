import type { Workspace } from "./types";

const WORKSPACES_KEY = "terminal-mate.workspaces.v1";
const ACTIVE_WORKSPACE_KEY = "terminal-mate.active-workspace.v1";

export function loadWorkspaces(): Workspace[] {
  try {
    const value = localStorage.getItem(WORKSPACES_KEY);
    if (!value) {
      return [];
    }

    const parsed: unknown = JSON.parse(value);
    return Array.isArray(parsed) ? (parsed as Workspace[]) : [];
  } catch {
    return [];
  }
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

