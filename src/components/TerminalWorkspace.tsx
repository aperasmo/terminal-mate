import { Plus, SquareTerminal, X } from "lucide-react";
import { useEffect, useRef } from "react";

import type { SessionOutputLine, SessionSummary, Workspace } from "../lib/types";

interface TerminalWorkspaceProps {
  workspace?: Workspace;
  session?: SessionSummary;
  output: SessionOutputLine[];
  startingSession: boolean;
  onCreateSession: () => void;
  onCloseSession: () => void;
}

export function TerminalWorkspace({
  workspace,
  session,
  output,
  startingSession,
  onCreateSession,
  onCloseSession
}: TerminalWorkspaceProps) {
  const outputRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const element = outputRef.current;
    if (element) {
      element.scrollTop = element.scrollHeight;
    }
  }, [output]);

  return (
    <main className="terminal-workspace">
      <div className="session-tabs">
        {workspace ? (
          <div className="session-tab active">
            <SquareTerminal size={15} />
            {workspace.name}
            {session ? (
              <button
                type="button"
                className="close-session-tab"
                aria-label="Close terminal session"
                title="Close terminal session"
                onClick={onCloseSession}
              >
                <X size={14} />
              </button>
            ) : null}
          </div>
        ) : (
          <span className="session-tabs-label">Terminal sessions</span>
        )}
        <button
          className="new-session-button"
          type="button"
          disabled={!workspace || Boolean(session) || startingSession}
          aria-label="New terminal session"
          title={session ? "Session already running" : "New terminal session"}
          onClick={onCreateSession}
        >
          <Plus size={17} />
        </button>
      </div>

      <section
        className={`terminal-surface${session ? " has-session" : ""}`}
        aria-label="Terminal"
      >
        {session ? (
          <div className="terminal-output" role="log" aria-live="polite" ref={outputRef}>
            {output.map((line, index) => (
              <div key={index} className={`terminal-line terminal-line-${line.stream}`}>
                {line.chunk}
              </div>
            ))}
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
}
