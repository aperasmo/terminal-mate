import { invoke } from "@tauri-apps/api/core";
import { CornerDownLeft, ShieldAlert, ShieldX } from "lucide-react";
import { FormEvent, useState } from "react";

import type { RiskDecision, SessionSummary, Workspace } from "../lib/types";

interface CommandBarProps {
  workspace?: Workspace;
  session?: SessionSummary;
}

type Phase = "idle" | "classifying" | "pending-approval" | "executing";

const RISK_LABEL: Record<RiskDecision["level"], string> = {
  safe: "SAFE",
  caution: "CAUTION",
  highRisk: "HIGH RISK",
  blocked: "BLOCKED"
};

export function CommandBar({ workspace, session }: CommandBarProps) {
  const [message, setMessage] = useState("");
  const [phase, setPhase] = useState<Phase>("idle");
  const [pendingCommand, setPendingCommand] = useState<string | null>(null);
  const [pendingDecision, setPendingDecision] = useState<RiskDecision | null>(null);
  const [notice, setNotice] = useState<{ level: RiskDecision["level"]; text: string } | null>(
    null
  );

  const enabled = Boolean(workspace) && Boolean(session) && phase === "idle";

  async function executeNow(command: string) {
    if (!session) {
      return;
    }

    setPhase("executing");
    try {
      await invoke("execute_command", { sessionId: session.id, command });
    } catch (error) {
      setNotice({
        level: "blocked",
        text: error instanceof Error ? error.message : "The command could not run."
      });
    } finally {
      setMessage("");
      setPendingCommand(null);
      setPendingDecision(null);
      setPhase("idle");
    }
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const command = message.trim();
    if (!command || !enabled) {
      return;
    }

    setNotice(null);
    setPhase("classifying");

    let decision: RiskDecision;
    try {
      decision = await invoke<RiskDecision>("classify_command_text", { command });
    } catch (error) {
      setNotice({
        level: "blocked",
        text: error instanceof Error ? error.message : "Could not classify this command."
      });
      setPhase("idle");
      return;
    }

    if (decision.level === "safe") {
      await executeNow(command);
      return;
    }

    if (decision.level === "blocked") {
      setNotice({ level: "blocked", text: decision.reason });
      setMessage("");
      setPhase("idle");
      return;
    }

    setPendingCommand(command);
    setPendingDecision(decision);
    setPhase("pending-approval");
  }

  function cancelPending() {
    setPendingCommand(null);
    setPendingDecision(null);
    setPhase("idle");
  }

  const placeholder = !workspace
    ? "Add a workspace to begin..."
    : !session
      ? "Start a terminal session to run commands..."
      : "Type a command or describe a task...";

  return (
    <footer className="command-area">
      {notice ? (
        <div className={`command-notice risk-${notice.level}`} role="status">
          <ShieldX size={14} />
          {notice.text}
        </div>
      ) : null}

      {pendingDecision && pendingCommand ? (
        <div className={`command-approval risk-${pendingDecision.level}`} role="alertdialog">
          <div className="command-approval-summary">
            <ShieldAlert size={15} />
            <span className="risk-badge">{RISK_LABEL[pendingDecision.level]}</span>
            <span className="command-approval-reason">{pendingDecision.reason}</span>
          </div>
          <code className="command-approval-text">{pendingCommand}</code>
          <div className="command-approval-actions">
            <button type="button" className="approval-cancel" onClick={cancelPending}>
              Cancel
            </button>
            <button
              type="button"
              className="approval-approve"
              onClick={() => void executeNow(pendingCommand)}
              disabled={phase === "executing"}
            >
              Approve and run
            </button>
          </div>
        </div>
      ) : null}

      <form className="command-bar" onSubmit={(event) => void handleSubmit(event)}>
        <span className="terminal-prompt" aria-hidden="true">
          <span className="terminal-prompt-runtime">
            {workspace?.profile.runtimeName ?? "terminal"}
          </span>
          <span className="terminal-prompt-path">
            :~/{workspace?.name ?? "workspace"}
          </span>
          <span className="terminal-prompt-symbol">$</span>
        </span>
        <input
          value={pendingCommand ?? message}
          onChange={(event) => {
            setNotice(null);
            setMessage(event.target.value);
          }}
          disabled={!enabled}
          placeholder={placeholder}
          aria-label="Command request"
          autoCapitalize="off"
          autoComplete="off"
          spellCheck={false}
        />
        <span className="command-policy" title="Every command is risk-classified before it runs">
          {phase === "classifying" ? "CHECKING..." : "PLAN FIRST"}
        </span>
        <button
          className="submit-command"
          type="submit"
          disabled={!enabled || message.trim().length === 0}
          aria-label="Classify and run command"
          title="Classify and run command"
        >
          <CornerDownLeft size={16} />
        </button>
      </form>
    </footer>
  );
}
