import {
  Box,
  Cpu,
  Folder,
  HardDrive,
  ShieldCheck,
  Terminal
} from "lucide-react";

import type { Workspace } from "../lib/types";

interface ContextPanelProps {
  workspace?: Workspace;
}

export function ContextPanel({ workspace }: ContextPanelProps) {
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
            <code>{workspace.profile.workingDirectory}</code>
          </section>

          <section className="safety-boundary">
            <ShieldCheck size={19} />
            <div>
              <strong>Execution boundary ready</strong>
              <span>Commands require a validated intent and policy decision.</span>
            </div>
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

