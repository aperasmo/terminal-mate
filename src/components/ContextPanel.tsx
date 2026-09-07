import {
  Box,
  Clock3,
  Cpu,
  FileText,
  Folder,
  HardDrive,
  Monitor,
  ShieldCheck,
  Terminal,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { formatDuration } from "../lib/duration";
import {
  collapseConsecutiveDuplicateRuns,
  HISTORY_DISPLAY_LIMIT,
} from "../lib/runHistory";
import type { CommandHistoryEntry, Workspace } from "../lib/types";

interface ContextPanelProps {
  workspace?: Workspace;
  currentDirectory?: string;
  history: CommandHistoryEntry[];
  runtimeSwitching: boolean;
  runtimeSwitchDisabled: boolean;
  onRuntimeChange: (runtime: "local" | "wsl") => void;
  onReuseRequest: (request: string) => void;
  onOpenLog: (path: string) => Promise<void>;
}

const STATUS_LABEL: Record<CommandHistoryEntry["status"], string> = {
  running: "Running",
  succeeded: "Succeeded",
  failed: "Failed",
  stopped: "Stopped",
  interrupted: "Interrupted",
};

function formatRunTime(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(value));
}

export function ContextPanel({
  workspace,
  currentDirectory,
  history,
  runtimeSwitching,
  runtimeSwitchDisabled,
  onRuntimeChange,
  onReuseRequest,
  onOpenLog,
}: ContextPanelProps) {
  const [clockMs, setClockMs] = useState(Date.now());
  const hasRunningEntry = history.some((entry) => entry.status === "running");
  const displayEntries = useMemo(
    () => collapseConsecutiveDuplicateRuns(history).slice(0, HISTORY_DISPLAY_LIMIT),
    [history],
  );

  useEffect(() => {
    if (!hasRunningEntry) {
      return;
    }
    setClockMs(Date.now());
    const timer = window.setInterval(() => setClockMs(Date.now()), 250);
    return () => window.clearInterval(timer);
  }, [hasRunningEntry]);

  return (
    <aside className="context-panel">
      <div className="context-header">
        <div>
          <span className="eyebrow">Environment context</span>
          <h2>{workspace?.name ?? "No active workspace"}</h2>
        </div>
      </div>

      {workspace ? (
        <div className="context-content">
          <section className="runtime-selector-section">
            <div className="runtime-selector-label">
              <span>Workspace runtime</span>
              {runtimeSwitching ? <small>Switching...</small> : null}
            </div>
            <div
              className="runtime-selector"
              role="group"
              aria-label="Workspace runtime"
            >
              <button
                type="button"
                className={
                  workspace.profile.runtime === "local" ? "active" : ""
                }
                aria-pressed={workspace.profile.runtime === "local"}
                title="Run this workspace with native Windows PowerShell"
                disabled={runtimeSwitchDisabled || runtimeSwitching}
                onClick={() => onRuntimeChange("local")}
              >
                <Monitor size={14} />
                Windows
              </button>
              <button
                type="button"
                className={
                  workspace.profile.runtime === "wsl" ? "active" : ""
                }
                aria-pressed={workspace.profile.runtime === "wsl"}
                title="Run this workspace inside WSL using Bash"
                disabled={runtimeSwitchDisabled || runtimeSwitching}
                onClick={() => onRuntimeChange("wsl")}
              >
                <Terminal size={14} />
                WSL
              </button>
            </div>
          </section>

          <dl className="context-list">
            <div>
              <dt>
                <HardDrive size={15} />
                Host
              </dt>
              <dd>{workspace.profile.hostOs}</dd>
            </div>
            <div>
              <dt>
                <Box size={15} />
                Runtime
              </dt>
              <dd>{workspace.profile.runtimeName}</dd>
            </div>
            <div>
              <dt>
                <Terminal size={15} />
                Shell
              </dt>
              <dd>{workspace.profile.shell}</dd>
            </div>
            <div>
              <dt>
                <Cpu size={15} />
                Architecture
              </dt>
              <dd>{workspace.profile.architecture}</dd>
            </div>
          </dl>

          <section className="context-section">
            <h3>
              <Folder size={15} />
              Working directory
            </h3>
            <code>
              {currentDirectory ?? workspace.profile.workingDirectory}
            </code>
          </section>

          <section className="safety-boundary">
            <ShieldCheck size={19} />
            <div>
              <strong>Execution boundary ready</strong>
              <span>
                Commands require a validated intent and policy decision.
              </span>
            </div>
          </section>

          <section className="run-history">
            <div className="run-history-header">
              <h3>
                <Terminal size={15} />
                Recent runs
              </h3>
              <span>{displayEntries.length}/{HISTORY_DISPLAY_LIMIT}</span>
            </div>
            {displayEntries.length > 0 ? (
              <ol className="run-history-list">
                {displayEntries.map(({ key, entry, repeatCount }) => (
                  <li
                    key={key}
                    className={`run-history-item ${entry.status}`}
                  >
                    <button
                      type="button"
                      className="run-history-reuse"
                      title="Use this request again"
                      aria-label={`Use request again: ${entry.request}`}
                      onClick={() => onReuseRequest(entry.request)}
                    >
                      <div className="run-history-summary">
                        <span className="run-history-status-group">
                          <span className={`run-history-status ${entry.status}`}>
                            {STATUS_LABEL[entry.status]}
                          </span>
                          {repeatCount > 1 ? (
                            <span
                              className="run-history-repeat"
                              title={`Ran ${repeatCount} times in a row`}
                            >
                              ×{repeatCount}
                            </span>
                          ) : null}
                        </span>
                        <time dateTime={entry.startedAt}>
                          {formatRunTime(entry.startedAt)}
                        </time>
                      </div>
                      <code title={entry.request}>{entry.request}</code>
                      {entry.request !== entry.command ? (
                        <code
                          className="run-history-resolved"
                          title={entry.command}
                        >
                          {entry.command}
                        </code>
                      ) : null}
                      <div className="run-history-meta">
                        <span>{entry.riskLevel}</span>
                        <span>{entry.approval}</span>
                        {entry.exitCode !== undefined ? (
                          <span>exit {entry.exitCode}</span>
                        ) : null}
                        {entry.durationMs !== undefined ||
                        entry.status === "running" ? (
                          <span className="run-history-duration">
                            <Clock3 size={11} />
                            {formatDuration(
                              entry.durationMs ??
                                Math.max(
                                  0,
                                  clockMs - Date.parse(entry.startedAt),
                                ),
                            )}
                          </span>
                        ) : null}
                      </div>
                    </button>
                    {entry.status !== "running" ? (
                      <button
                        type="button"
                        className="run-history-log"
                        title="Open command log"
                        aria-label={`Open log for ${entry.command}`}
                        onClick={() => void onOpenLog(entry.logPath)}
                      >
                        <FileText size={12} />
                      </button>
                    ) : null}
                  </li>
                ))}
              </ol>
            ) : (
              <p className="run-history-empty">
                Commands and approvals for this workspace will appear here and
                remain after restart.
              </p>
            )}
          </section>
        </div>
      ) : (
        <div className="context-empty">
          <ShieldCheck size={24} />
          <p>Environment details appear after selecting a workspace.</p>
        </div>
      )}
    </aside>
  );
}
