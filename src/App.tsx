import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";

import { CommandBar } from "./components/CommandBar";
import { ContextPanel } from "./components/ContextPanel";
import { TerminalWorkspace } from "./components/TerminalWorkspace";
import { TopBar } from "./components/TopBar";
import { WorkspaceSidebar } from "./components/WorkspaceSidebar";
import {
  loadActiveWorkspaceId,
  loadWorkspaces,
  saveActiveWorkspaceId,
  saveWorkspaces
} from "./lib/storage";
import type {
  SessionOutputEvent,
  SessionOutputLine,
  SessionSummary,
  Workspace
} from "./lib/types";

const FALLBACK_VERSION = "0.1.0";

function App() {
  const [version, setVersion] = useState(FALLBACK_VERSION);
  const [workspaces, setWorkspaces] = useState<Workspace[]>(loadWorkspaces);
  const [activeWorkspaceId, setActiveWorkspaceId] = useState<string | null>(
    loadActiveWorkspaceId
  );
  const [pickingWorkspace, setPickingWorkspace] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [sessionsByWorkspace, setSessionsByWorkspace] = useState<
    Record<string, SessionSummary>
  >({});
  const [outputBySession, setOutputBySession] = useState<
    Record<string, SessionOutputLine[]>
  >({});
  const [startingSession, setStartingSession] = useState(false);

  const activeWorkspace = useMemo(
    () => workspaces.find((workspace) => workspace.id === activeWorkspaceId),
    [activeWorkspaceId, workspaces]
  );

  const activeSession = activeWorkspace
    ? sessionsByWorkspace[activeWorkspace.id]
    : undefined;

  useEffect(() => {
    void getVersion().then(setVersion).catch(() => setVersion(FALLBACK_VERSION));
  }, []);

  useEffect(() => {
    saveWorkspaces(workspaces);
  }, [workspaces]);

  useEffect(() => {
    saveActiveWorkspaceId(activeWorkspaceId);
  }, [activeWorkspaceId]);

  useEffect(() => {
    if (activeWorkspaceId && !activeWorkspace) {
      setActiveWorkspaceId(workspaces.at(0)?.id ?? null);
    }
  }, [activeWorkspace, activeWorkspaceId, workspaces]);

  // Selecting (or adding) a workspace activates its terminal automatically —
  // no separate manual step. Closing a session explicitly (the X on the tab)
  // does not re-trigger this, since that only depends on the workspace id,
  // not on whether a session currently exists.
  useEffect(() => {
    if (activeWorkspaceId) {
      void createSession();
    }
  }, [activeWorkspaceId]);

  useEffect(() => {
    const unlisten = listen<SessionOutputEvent>("session-output", (event) => {
      const { sessionId, stream, chunk } = event.payload;
      setOutputBySession((current) => ({
        ...current,
        [sessionId]: [...(current[sessionId] ?? []), { stream, chunk }]
      }));
    });

    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, []);

  async function createSession() {
    if (!activeWorkspace || sessionsByWorkspace[activeWorkspace.id]) {
      return;
    }

    setStartingSession(true);
    setNotice(null);

    try {
      const session = await invoke<SessionSummary>("create_session", {
        workspace: activeWorkspace
      });
      setSessionsByWorkspace((current) => ({
        ...current,
        [session.workspaceId]: session
      }));
    } catch (error) {
      setNotice(
        error instanceof Error
          ? error.message
          : "Failed to start the terminal session."
      );
    } finally {
      setStartingSession(false);
    }
  }

  async function closeSession() {
    const session = activeSession;
    if (!session) {
      return;
    }

    try {
      await invoke("close_session", { sessionId: session.id });
    } catch {
      // The process may have already exited; local state is cleared regardless.
    }

    setSessionsByWorkspace((current) => {
      const next = { ...current };
      delete next[session.workspaceId];
      return next;
    });
    setOutputBySession((current) => {
      const next = { ...current };
      delete next[session.id];
      return next;
    });
  }

  async function addWorkspace() {
    setPickingWorkspace(true);
    setNotice(null);

    try {
      const workspace = await invoke<Workspace | null>("pick_workspace");
      if (!workspace) {
        return;
      }

      const existing = workspaces.find(
        (item) => item.path.toLowerCase() === workspace.path.toLowerCase()
      );
      if (existing) {
        setActiveWorkspaceId(existing.id);
        setNotice(`${existing.name} is already registered.`);
        return;
      }

      setWorkspaces((current) => [...current, workspace]);
      setActiveWorkspaceId(workspace.id);
    } catch (error) {
      setNotice(
        error instanceof Error
          ? error.message
          : "The native folder picker is available in the Tauri desktop app."
      );
    } finally {
      setPickingWorkspace(false);
    }
  }

  function removeWorkspace(workspaceId: string) {
    const session = sessionsByWorkspace[workspaceId];
    if (session) {
      void invoke("close_session", { sessionId: session.id }).catch(() => undefined);
      setSessionsByWorkspace((current) => {
        const next = { ...current };
        delete next[workspaceId];
        return next;
      });
      setOutputBySession((current) => {
        const next = { ...current };
        delete next[session.id];
        return next;
      });
    }

    setWorkspaces((current) =>
      current.filter((workspace) => workspace.id !== workspaceId)
    );
  }

  const environmentLabel = activeWorkspace
    ? `${activeWorkspace.profile.runtimeName} / ${activeWorkspace.profile.shell}`
    : undefined;

  return (
    <div className="app-shell">
      <TopBar
        version={version}
        activeWorkspaceName={activeWorkspace?.name}
        environmentLabel={environmentLabel}
      />

      <div className="workspace-layout">
        <WorkspaceSidebar
          workspaces={workspaces}
          activeWorkspaceId={activeWorkspaceId}
          busy={pickingWorkspace}
          onAdd={() => void addWorkspace()}
          onSelect={setActiveWorkspaceId}
          onRemove={removeWorkspace}
        />

        <div className="primary-column">
          {notice ? (
            <div className="application-notice" role="status">
              {notice}
            </div>
          ) : null}
          <TerminalWorkspace
            workspace={activeWorkspace}
            session={activeSession}
            output={activeSession ? outputBySession[activeSession.id] ?? [] : []}
            startingSession={startingSession}
            onCreateSession={() => void createSession()}
            onCloseSession={() => void closeSession()}
          />
          <CommandBar workspace={activeWorkspace} session={activeSession} />
        </div>

        <ContextPanel workspace={activeWorkspace} />
      </div>
    </div>
  );
}

export default App;
