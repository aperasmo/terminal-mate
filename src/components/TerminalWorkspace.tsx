import { Eye, EyeOff, LoaderCircle, SquareTerminal, X } from "lucide-react";
import { memo, useEffect, useMemo, useRef, useState } from "react";

import type {
  SessionOutputLine,
  SessionSummary,
  Workspace,
} from "../lib/types";
import {
  containsSensitiveOutput,
  maskSensitiveOutput,
} from "../lib/sensitiveOutput";
import { formatTerminalTimestamp } from "../lib/terminalTimestamp";

interface TerminalWorkspaceProps {
  workspace?: Workspace;
  session?: SessionSummary;
  output: SessionOutputLine[];
  failedRunIds: string[];
  hiddenLineCount: number;
  activeCommandRunning: boolean;
  startingSession: boolean;
  onCreateSession: () => void;
  onCloseSession: () => void;
}

export const TerminalWorkspace = memo(function TerminalWorkspace({
  workspace,
  session,
  output,
  failedRunIds,
  hiddenLineCount,
  activeCommandRunning,
  startingSession,
  onCreateSession,
  onCloseSession,
}: TerminalWorkspaceProps) {
  const outputRef = useRef<HTMLDivElement | null>(null);
  const [showSensitiveOutput, setShowSensitiveOutput] = useState(false);
  const hasSensitiveOutput = useMemo(
    () => output.some((line) => containsSensitiveOutput(line.chunk)),
    [output],
  );
  const failedRuns = useMemo(() => new Set(failedRunIds), [failedRunIds]);

  useEffect(() => {
    setShowSensitiveOutput(false);
  }, [session?.id]);

  useEffect(() => {
    const element = outputRef.current;
    if (element) {
      element.scrollTop = element.scrollHeight;
    }
  }, [output, activeCommandRunning]);

  return (
    <main className="terminal-workspace">
      <div className="session-tabs">
        {workspace ? (
          <div
            className={`session-tab active${session ? "" : " clickable"}`}
            role="button"
            tabIndex={0}
            title={
              session ? workspace.name : "Click to start a terminal session"
            }
            onClick={session ? undefined : onCreateSession}
            onKeyDown={(event) => {
              if (!session && (event.key === "Enter" || event.key === " ")) {
                event.preventDefault();
                onCreateSession();
              }
            }}
          >
            <SquareTerminal size={15} />
            {workspace.name}
            {session ? (
              <button
                type="button"
                className="close-session-tab"
                aria-label={
                  activeCommandRunning
                    ? "Stop command and close terminal session"
                    : "Close terminal session"
                }
                title={
                  activeCommandRunning
                    ? "Stop command and close terminal session"
                    : "Close terminal session"
                }
                onClick={(event) => {
                  event.stopPropagation();
                  onCloseSession();
                }}
              >
                <X size={14} />
              </button>
            ) : startingSession ? (
              <span className="session-tab-status">starting...</span>
            ) : null}
          </div>
        ) : (
          <span className="session-tabs-label">Terminal sessions</span>
        )}
      </div>

      <section
        className={`terminal-surface${session ? " has-session" : ""}`}
        aria-label="Terminal"
      >
        {session ? (
          <div
            className="terminal-output"
            role="log"
            aria-live="polite"
            ref={outputRef}
          >
            {hasSensitiveOutput ? (
              <div className="terminal-sensitive-controls">
                <button
                  type="button"
                  className="terminal-sensitive-toggle"
                  aria-pressed={showSensitiveOutput}
                  title={
                    showSensitiveOutput
                      ? "Hide sensitive terminal data"
                      : "Show sensitive terminal data"
                  }
                  onClick={() => setShowSensitiveOutput((current) => !current)}
                >
                  {showSensitiveOutput ? (
                    <EyeOff size={14} aria-hidden="true" />
                  ) : (
                    <Eye size={14} aria-hidden="true" />
                  )}
                  {showSensitiveOutput ? "Hide sensitive data" : "Show masked data"}
                </button>
              </div>
            ) : null}
            {output.map((line, index) =>
              line.commandSeparator ? (
                <div
                  key={index}
                  className="terminal-command-separator"
                  role="separator"
                  aria-label="Next command output"
                />
              ) : (
                <div
                  key={index}
                  className={`terminal-line terminal-line-${line.stream}${
                    line.chunk === "No output returned."
                      ? " terminal-line-empty"
                      : ""
                  }${line.commandStart ? " terminal-line-command" : ""}${
                    line.runId && failedRuns.has(line.runId)
                      ? " terminal-line-failed"
                      : ""
                  }`}
                >
                  <time
                    className="terminal-line-timestamp"
                    dateTime={new Date(line.receivedAtMs).toISOString()}
                  >
                    {formatTerminalTimestamp(line.receivedAtMs)}
                  </time>
                  <span className="terminal-line-content">
                    {showSensitiveOutput
                      ? line.chunk
                      : maskSensitiveOutput(line.chunk).text}
                  </span>
                </div>
              ),
            )}
            {activeCommandRunning ? (
              <div className="terminal-running-indicator" role="status">
                <LoaderCircle size={13} aria-hidden="true" />
                Running. Live output will appear here as the command produces
                it.
              </div>
            ) : null}
            {hiddenLineCount > 0 ? (
              <div className="terminal-buffer-notice" role="status">
                Showing the latest {output.length.toLocaleString()} lines.{" "}
                {hiddenLineCount.toLocaleString()} earlier lines are hidden; use
                Open log for the complete command output.
              </div>
            ) : null}
          </div>
        ) : workspace ? (
          <div className="terminal-empty">
            <span className="terminal-empty-icon">
              <SquareTerminal size={30} />
            </span>
            <h1>{workspace.name}</h1>
            <p>{workspace.profile.workingDirectory}</p>
            <span className="session-readiness">
              {startingSession ? "Starting session..." : "Workspace registered"}
            </span>
          </div>
        ) : (
          <div className="terminal-empty">
            <span className="terminal-empty-icon">
              <SquareTerminal size={30} />
            </span>
            <h1>Ready</h1>
            <p>Add a workspace to prepare its terminal environment.</p>
          </div>
        )}
      </section>
    </main>
  );
}, terminalWorkspacePropsAreEqual);

function terminalWorkspacePropsAreEqual(
  previous: TerminalWorkspaceProps,
  next: TerminalWorkspaceProps,
) {
  return (
    previous.workspace === next.workspace &&
    previous.session === next.session &&
    previous.output === next.output &&
    previous.hiddenLineCount === next.hiddenLineCount &&
    previous.activeCommandRunning === next.activeCommandRunning &&
    previous.startingSession === next.startingSession
  );
}
