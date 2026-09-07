import {
  Bot,
  CircleHelp,
  Settings,
  ShieldCheck,
  SquareTerminal,
} from "lucide-react";

interface TopBarProps {
  version: string;
  activeWorkspaceName?: string;
  environmentLabel?: string;
  onHelp: () => void;
  onSettings: () => void;
  aiPlannerEnabled: boolean;
}

export function TopBar({
  version,
  activeWorkspaceName,
  environmentLabel,
  onHelp,
  onSettings,
  aiPlannerEnabled,
}: TopBarProps) {
  return (
    <header className="topbar">
      <div className="brand">
        <span className="brand-mark" aria-hidden="true">
          <SquareTerminal size={20} strokeWidth={2} />
        </span>
        <span className="brand-name">TerminalMate</span>
        <span className="version-badge">v{version}</span>
        <span className="preview-badge">UI Preview</span>
      </div>

      <div className="topbar-context" aria-label="Active workspace">
        <span className="workspace-chip">
          <span className="workspace-chip-mark">
            {activeWorkspaceName?.charAt(0).toUpperCase() ?? "-"}
          </span>
          {activeWorkspaceName ?? "No workspace"}
        </span>
        {environmentLabel ? (
          <span className="environment-chip">
            <SquareTerminal size={15} />
            {environmentLabel}
          </span>
        ) : null}
      </div>

      <nav className="topbar-actions" aria-label="Application">
        <span className="safety-status">
          <ShieldCheck size={16} />
          Safety ready
        </span>
        <button
          className={`ai-planner-status${aiPlannerEnabled ? " is-enabled" : ""}`}
          type="button"
          aria-label={`Generative AI planner is ${aiPlannerEnabled ? "on" : "off"}. Open settings.`}
          title="Open generative AI planner settings"
          onClick={onSettings}
        >
          <Bot size={16} />
          AI planner: {aiPlannerEnabled ? "On" : "Off"}
        </button>
        <button
          className="icon-button"
          type="button"
          aria-label="Help"
          title="Help"
          onClick={onHelp}
        >
          <CircleHelp size={19} />
        </button>
        <button
          className="icon-button"
          type="button"
          aria-label="Settings"
          title="Settings"
          onClick={onSettings}
        >
          <Settings size={19} />
        </button>
      </nav>
    </header>
  );
}
