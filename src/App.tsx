import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useRef, useState } from "react";

import { CommandBar } from "./components/CommandBar";
import { ContextPanel } from "./components/ContextPanel";
import { FileEditorModal } from "./components/FileEditorModal";
import { HelpModal } from "./components/HelpModal";
import {
  SettingsModal,
  type AiPlannerSettings,
} from "./components/SettingsModal";
import { TerminalWorkspace } from "./components/TerminalWorkspace";
import { TopBar } from "./components/TopBar";
import { WorkspaceSidebar } from "./components/WorkspaceSidebar";
import {
  loadActiveWorkspaceId,
  loadCommandHistory,
  loadWorkspaces,
  saveActiveWorkspaceId,
  saveCommandHistory,
  saveWorkspaces,
} from "./lib/storage";
import {
  appendTerminalOutput,
  type TerminalOutputBuffer,
} from "./lib/terminalOutput";
import { prepareTerminalOutput } from "./lib/terminalTranscript";
import { defaultHelpRequest, type HelpRequest } from "./lib/helpRequests";
import { HISTORY_STORAGE_LIMIT } from "./lib/runHistory";
import type {
  CommandHistoryEntry,
  CommandRunFinishedEvent,
  CommandRunSummary,
  RiskDecision,
  SessionOutputEvent,
  SessionOutputLine,
  SessionSummary,
  Workspace,
} from "./lib/types";

const FALLBACK_VERSION = "0.1.0";

function nextAvailableWorkspaceAlias(
  baseName: string,
  workspaces: Workspace[],
) {
  const aliases = new Set(
    workspaces.map((workspace) => workspace.name.toLocaleLowerCase()),
  );
  if (!aliases.has(baseName.toLocaleLowerCase())) {
    return baseName;
  }

  let suffix = 2;
  while (aliases.has(`${baseName} (${suffix})`.toLocaleLowerCase())) {
    suffix += 1;
  }
  return `${baseName} (${suffix})`;
}

function App() {
  const [version, setVersion] = useState(FALLBACK_VERSION);
  const [workspaces, setWorkspaces] = useState<Workspace[]>(loadWorkspaces);
  const [activeWorkspaceId, setActiveWorkspaceId] = useState<string | null>(
    loadActiveWorkspaceId,
  );
  const [pickingWorkspace, setPickingWorkspace] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [helpRequest, setHelpRequest] = useState<HelpRequest | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [aiPlannerEnabled, setAiPlannerEnabled] = useState(false);
  const [editingFile, setEditingFile] = useState<{
    path: string;
    workingDirectory: string;
  } | null>(null);
  const [sessionsByWorkspace, setSessionsByWorkspace] = useState<
    Record<string, SessionSummary>
  >({});
  const [outputBySession, setOutputBySession] = useState<
    Record<string, TerminalOutputBuffer>
  >({});
  const [startingSession, setStartingSession] = useState(false);
  const [switchingRuntime, setSwitchingRuntime] = useState(false);
  const [activeRunBySession, setActiveRunBySession] = useState<
    Record<string, CommandRunSummary>
  >({});
  const [lastFinishedBySession, setLastFinishedBySession] = useState<
    Record<string, CommandRunFinishedEvent>
  >({});
  const [currentDirectoryBySession, setCurrentDirectoryBySession] = useState<
    Record<string, string>
  >({});
  const [historyByWorkspace, setHistoryByWorkspace] =
    useState<Record<string, CommandHistoryEntry[]>>(loadCommandHistory);
  const [recalledRequest, setRecalledRequest] = useState<{
    request: string;
    token: number;
  } | null>(null);
  const finishedRunBuffer = useRef<Record<string, CommandRunFinishedEvent>>({});
  const knownRunIds = useRef(new Set<string>());
  const workspaceIdBySession = useRef<Record<string, string>>({});
  const pendingOutputBySession = useRef<Record<string, SessionOutputLine[]>>(
    {},
  );
  const lastOutputRunIdBySession = useRef<Record<string, string | null>>({});
  const outputFlushTimer = useRef<number | null>(null);

  const activeWorkspace = useMemo(
    () => workspaces.find((workspace) => workspace.id === activeWorkspaceId),
    [activeWorkspaceId, workspaces],
  );

  const activeSession = activeWorkspace
    ? sessionsByWorkspace[activeWorkspace.id]
    : undefined;

  useEffect(() => {
    void invoke<AiPlannerSettings>("get_ai_planner_settings")
      .then((settings) => setAiPlannerEnabled(settings.enabled))
      .catch(() => setAiPlannerEnabled(false));
  }, []);

  const workspaceWarnings = useMemo(() => {
    const warnings: Record<string, boolean> = {};
    for (const workspace of workspaces) {
      const session = sessionsByWorkspace[workspace.id];
      if (!session) {
        continue;
      }
      // A run in progress supersedes whatever the previous run's outcome
      // was; only a finished, non-user-stopped failure counts as something
      // that still needs attention.
      if (activeRunBySession[session.id]) {
        continue;
      }
      const lastFinished = lastFinishedBySession[session.id];
      warnings[workspace.id] = Boolean(
        lastFinished && !lastFinished.success && !lastFinished.stoppedByUser,
      );
    }
    return warnings;
  }, [
    workspaces,
    sessionsByWorkspace,
    activeRunBySession,
    lastFinishedBySession,
  ]);

  useEffect(() => {
    void getVersion()
      .then(setVersion)
      .catch(() => setVersion(FALLBACK_VERSION));
  }, []);

  useEffect(() => {
    saveWorkspaces(workspaces);
  }, [workspaces]);

  useEffect(() => {
    saveActiveWorkspaceId(activeWorkspaceId);
  }, [activeWorkspaceId]);

  useEffect(() => {
    saveCommandHistory(historyByWorkspace);
  }, [historyByWorkspace]);

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
  }, [
    activeWorkspaceId,
    activeWorkspace?.profile.runtime,
    activeWorkspace?.profile.shell,
  ]);

  useEffect(() => {
    const flushPendingOutput = () => {
      const pending = pendingOutputBySession.current;
      pendingOutputBySession.current = {};
      outputFlushTimer.current = null;

      if (Object.keys(pending).length === 0) {
        return;
      }

      setOutputBySession((current) => {
        const next = { ...current };
        for (const [sessionId, lines] of Object.entries(pending)) {
          next[sessionId] = appendTerminalOutput(current[sessionId], lines);
        }
        return next;
      });
    };

    const unlisten = listen<SessionOutputEvent>("session-output", (event) => {
      const { sessionId } = event.payload;
      const prepared = prepareTerminalOutput(
        event.payload,
        lastOutputRunIdBySession.current[sessionId] ?? null,
        Date.now(),
      );
      if (prepared.lines.length === 0) {
        return;
      }

      lastOutputRunIdBySession.current[sessionId] = prepared.latestRunId;
      const pending = pendingOutputBySession.current[sessionId] ?? [];
      pending.push(...prepared.lines);
      pendingOutputBySession.current[sessionId] = pending;

      if (outputFlushTimer.current === null) {
        outputFlushTimer.current = window.setTimeout(flushPendingOutput, 50);
      }
    });

    return () => {
      if (outputFlushTimer.current !== null) {
        window.clearTimeout(outputFlushTimer.current);
        outputFlushTimer.current = null;
      }
      pendingOutputBySession.current = {};
      lastOutputRunIdBySession.current = {};
      void unlisten.then((dispose) => dispose());
    };
  }, []);

  useEffect(() => {
    const unlisten = listen<CommandRunFinishedEvent>(
      "command-finished",
      (event) => {
        const finished = event.payload;
        if (knownRunIds.current.has(finished.id)) {
          knownRunIds.current.delete(finished.id);
        } else {
          finishedRunBuffer.current[finished.id] = finished;
        }
        setActiveRunBySession((current) => {
          const next = { ...current };
          delete next[finished.sessionId];
          return next;
        });
        setLastFinishedBySession((current) => ({
          ...current,
          [finished.sessionId]: finished,
        }));
        const workspaceId = workspaceIdBySession.current[finished.sessionId];
        if (workspaceId) {
          setHistoryByWorkspace((current) => ({
            ...current,
            [workspaceId]: (current[workspaceId] ?? []).map((entry) =>
              entry.id === finished.id
                ? {
                    ...entry,
                    status: finished.stoppedByUser
                      ? "stopped"
                      : finished.success
                        ? "succeeded"
                        : "failed",
                    finishedAt: new Date(finished.finishedAtMs).toISOString(),
                    durationMs: finished.durationMs,
                    exitCode: finished.exitCode,
                    logPath: finished.logPath,
                  }
                : entry,
            ),
          }));
        }
        if (finished.workingDirectory) {
          setCurrentDirectoryBySession((current) => ({
            ...current,
            [finished.sessionId]: finished.workingDirectory,
          }));
        }
      },
    );

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
        workspace: activeWorkspace,
      });
      setSessionsByWorkspace((current) => ({
        ...current,
        [session.workspaceId]: session,
      }));
      workspaceIdBySession.current[session.id] = session.workspaceId;
      setCurrentDirectoryBySession((current) => ({
        ...current,
        [session.id]: activeWorkspace.profile.workingDirectory,
      }));
    } catch (error) {
      setNotice(
        error instanceof Error
          ? error.message
          : "Failed to start the terminal session.",
      );
    } finally {
      setStartingSession(false);
    }
  }

  async function runCommand(
    request: string,
    command: string,
    decision: RiskDecision,
  ) {
    if (!activeSession || !activeWorkspace) {
      throw new Error("Start a terminal session before running a command.");
    }

    const summary = await invoke<CommandRunSummary>("run_command", {
      sessionId: activeSession.id,
      command,
    });
    const bufferedFinish = finishedRunBuffer.current[summary.id];
    if (bufferedFinish) {
      delete finishedRunBuffer.current[summary.id];
    } else {
      knownRunIds.current.add(summary.id);
      setActiveRunBySession((current) => ({
        ...current,
        [summary.sessionId]: summary,
      }));
    }
    setHistoryByWorkspace((current) => ({
      ...current,
      [activeWorkspace.id]: [
        {
          id: summary.id,
          sessionId: summary.sessionId,
          request,
          command: summary.command,
          riskLevel: decision.level,
          approval: decision.level === "safe" ? "automatic" : "approved",
          status: bufferedFinish
            ? bufferedFinish.stoppedByUser
              ? "stopped"
              : bufferedFinish.success
                ? "succeeded"
                : "failed"
            : "running",
          startedAt: new Date(summary.startedAtMs).toISOString(),
          finishedAt: bufferedFinish
            ? new Date(bufferedFinish.finishedAtMs).toISOString()
            : undefined,
          durationMs: bufferedFinish?.durationMs,
          exitCode: bufferedFinish?.exitCode,
          logPath: summary.logPath,
        } satisfies CommandHistoryEntry,
        ...(current[activeWorkspace.id] ?? []),
      ].slice(0, HISTORY_STORAGE_LIMIT),
    }));
    if (!bufferedFinish) {
      setLastFinishedBySession((current) => {
        const next = { ...current };
        delete next[summary.sessionId];
        return next;
      });
    }
    return summary;
  }

  async function navigateSessionDirectory(directory: string) {
    if (!activeSession) {
      throw new Error("Start a terminal session before changing directory.");
    }

    const workingDirectory = await invoke<string>(
      "set_session_working_directory",
      {
        sessionId: activeSession.id,
        workingDirectory: directory,
      },
    );
    setCurrentDirectoryBySession((current) => ({
      ...current,
      [activeSession.id]: workingDirectory,
    }));
    return workingDirectory;
  }

  async function stopCommand() {
    if (!activeSession) {
      return;
    }
    await invoke("stop_command", { sessionId: activeSession.id });
  }

  async function sendSessionInput(input: string) {
    if (!activeSession) {
      throw new Error("No terminal session is active.");
    }
    await invoke("send_session_input", {
      sessionId: activeSession.id,
      input,
    });
  }

  async function openLog(path: string) {
    await invoke("open_log_file", { path });
  }

  function recallRequest(request: string) {
    setRecalledRequest((current) => ({
      request,
      token: (current?.token ?? 0) + 1,
    }));
  }

  async function closeSession() {
    const session = activeSession;
    if (!session) {
      return;
    }

    if (activeRunBySession[session.id]) {
      await invoke("stop_command", { sessionId: session.id }).catch(
        () => undefined,
      );
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
    delete workspaceIdBySession.current[session.id];
    delete pendingOutputBySession.current[session.id];
    delete lastOutputRunIdBySession.current[session.id];
    setOutputBySession((current) => {
      const next = { ...current };
      delete next[session.id];
      return next;
    });
    setActiveRunBySession((current) => {
      const next = { ...current };
      delete next[session.id];
      return next;
    });
    setLastFinishedBySession((current) => {
      const next = { ...current };
      delete next[session.id];
      return next;
    });
    setCurrentDirectoryBySession((current) => {
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

      const registeredWorkspace = {
        ...workspace,
        name: nextAvailableWorkspaceAlias(workspace.name, workspaces),
      };
      setWorkspaces((current) => [...current, registeredWorkspace]);
      setActiveWorkspaceId(registeredWorkspace.id);
    } catch (error) {
      setNotice(
        error instanceof Error
          ? error.message
          : "The native folder picker is available in the Tauri desktop app.",
      );
    } finally {
      setPickingWorkspace(false);
    }
  }

  function renameWorkspace(workspaceId: string, requestedAlias: string) {
    const alias = requestedAlias.trim().replace(/\s+/g, " ");
    if (!alias) {
      return "Enter a workspace alias.";
    }

    const duplicateAlias = workspaces.some(
      (workspace) =>
        workspace.id !== workspaceId &&
        workspace.name.toLocaleLowerCase() === alias.toLocaleLowerCase(),
    );
    if (duplicateAlias) {
      return "Choose a different alias; this name is already in use.";
    }

    setWorkspaces((current) =>
      current.map((workspace) =>
        workspace.id === workspaceId
          ? { ...workspace, name: alias }
          : workspace,
      ),
    );
    return null;
  }

  async function changeWorkspaceRuntime(runtime: "local" | "wsl") {
    if (
      !activeWorkspace ||
      activeWorkspace.profile.runtime === runtime ||
      switchingRuntime
    ) {
      return;
    }

    if (activeSession && activeRunBySession[activeSession.id]) {
      setNotice(
        "Stop the running command before switching this workspace runtime.",
      );
      return;
    }

    setSwitchingRuntime(true);
    setNotice(null);
    try {
      const configured = await invoke<Workspace>(
        "configure_workspace_runtime",
        { workspace: activeWorkspace, runtime },
      );

      await closeSession();
      setWorkspaces((current) =>
        current.map((workspace) =>
          workspace.id === configured.id ? configured : workspace,
        ),
      );
    } catch (error) {
      setNotice(
        typeof error === "string"
          ? error
          : error instanceof Error
            ? error.message
            : "Failed to switch the workspace runtime.",
      );
    } finally {
      setSwitchingRuntime(false);
    }
  }

  function removeWorkspace(workspaceId: string) {
    const session = sessionsByWorkspace[workspaceId];
    if (session) {
      if (activeRunBySession[session.id]) {
        void invoke("stop_command", { sessionId: session.id }).catch(
          () => undefined,
        );
      }
      void invoke("close_session", { sessionId: session.id }).catch(
        () => undefined,
      );
      setSessionsByWorkspace((current) => {
        const next = { ...current };
        delete next[workspaceId];
        return next;
      });
      delete workspaceIdBySession.current[session.id];
      delete pendingOutputBySession.current[session.id];
      delete lastOutputRunIdBySession.current[session.id];
      setOutputBySession((current) => {
        const next = { ...current };
        delete next[session.id];
        return next;
      });
      setActiveRunBySession((current) => {
        const next = { ...current };
        delete next[session.id];
        return next;
      });
      setLastFinishedBySession((current) => {
        const next = { ...current };
        delete next[session.id];
        return next;
      });
      setCurrentDirectoryBySession((current) => {
        const next = { ...current };
        delete next[session.id];
        return next;
      });
    }

    setHistoryByWorkspace((current) => {
      const next = { ...current };
      delete next[workspaceId];
      return next;
    });
    setWorkspaces((current) =>
      current.filter((workspace) => workspace.id !== workspaceId),
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
        onHelp={() => setHelpRequest(defaultHelpRequest())}
        onSettings={() => setSettingsOpen(true)}
        aiPlannerEnabled={aiPlannerEnabled}
      />

      <div className="workspace-layout">
        <WorkspaceSidebar
          workspaces={workspaces}
          activeWorkspaceId={activeWorkspaceId}
          busy={pickingWorkspace}
          warnings={workspaceWarnings}
          onAdd={() => void addWorkspace()}
          onSelect={setActiveWorkspaceId}
          onRemove={removeWorkspace}
          onRename={renameWorkspace}
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
            output={
              activeSession
                ? (outputBySession[activeSession.id]?.lines ?? [])
                : []
            }
            failedRunIds={
              activeWorkspace
                ? (historyByWorkspace[activeWorkspace.id] ?? [])
                    .filter((entry) => entry.status === "failed")
                    .map((entry) => entry.id)
                : []
            }
            hiddenLineCount={
              activeSession
                ? (outputBySession[activeSession.id]?.hiddenLineCount ?? 0)
                : 0
            }
            activeCommandRunning={Boolean(
              activeSession && activeRunBySession[activeSession.id],
            )}
            startingSession={startingSession}
            onCreateSession={() => void createSession()}
            onCloseSession={() => void closeSession()}
          />
          <CommandBar
            workspace={activeWorkspace}
            session={activeSession}
            activeRun={
              activeSession ? activeRunBySession[activeSession.id] : undefined
            }
            lastFinished={
              activeSession
                ? lastFinishedBySession[activeSession.id]
                : undefined
            }
            lastHistoryEntry={
              activeWorkspace && activeSession
                ? (historyByWorkspace[activeWorkspace.id] ?? []).find(
                    (entry) =>
                      entry.sessionId === activeSession.id &&
                      entry.id === lastFinishedBySession[activeSession.id]?.id,
                  )
                : undefined
            }
            currentDirectory={
              activeSession
                ? currentDirectoryBySession[activeSession.id]
                : activeWorkspace?.profile.workingDirectory
            }
            recalledRequest={recalledRequest}
            onRunCommand={(request, command, decision) =>
              runCommand(request, command, decision)
            }
            onNavigateDirectory={(directory) =>
              navigateSessionDirectory(directory)
            }
            onStopCommand={() => stopCommand()}
            onSendSessionInput={(input) => sendSessionInput(input)}
            onOpenLog={(path) => openLog(path)}
            onOpenHelp={setHelpRequest}
            onOpenFileEditor={(path, workingDirectory) =>
              setEditingFile({ path, workingDirectory })
            }
            onOpenExternalTerminal={(workingDirectory, shell, command) => {
              void invoke("open_external_terminal", {
                workingDirectory,
                shell,
                command,
              }).catch((error) => {
                setNotice(
                  error instanceof Error
                    ? error.message
                    : "Could not open an external terminal.",
                );
              });
            }}
          />
        </div>

        <ContextPanel
          workspace={activeWorkspace}
          currentDirectory={
            activeSession
              ? currentDirectoryBySession[activeSession.id]
              : activeWorkspace?.profile.workingDirectory
          }
          history={
            activeWorkspace
              ? (historyByWorkspace[activeWorkspace.id] ?? [])
              : []
          }
          runtimeSwitching={switchingRuntime}
          runtimeSwitchDisabled={Boolean(
            startingSession ||
            (activeSession && activeRunBySession[activeSession.id]),
          )}
          onRuntimeChange={(runtime) => void changeWorkspaceRuntime(runtime)}
          onReuseRequest={recallRequest}
          onOpenLog={(path) => openLog(path)}
        />
      </div>
      {helpRequest ? (
        <HelpModal
          initialRequest={helpRequest}
          onClose={() => setHelpRequest(null)}
          onUseCommand={(command) => {
            setHelpRequest(null);
            recallRequest(command);
          }}
        />
      ) : null}
      {settingsOpen ? (
        <SettingsModal
          onClose={() => setSettingsOpen(false)}
          onSaved={(settings) => setAiPlannerEnabled(settings.enabled)}
        />
      ) : null}
      {editingFile ? (
        <FileEditorModal
          path={editingFile.path}
          workingDirectory={editingFile.workingDirectory}
          onClose={() => setEditingFile(null)}
        />
      ) : null}
    </div>
  );
}

export default App;
