import type { SessionOutputEvent, SessionOutputLine } from "./types";

export interface PreparedTerminalOutput {
  lines: SessionOutputLine[];
  latestRunId: string | null;
}

export function prepareTerminalOutput(
  event: SessionOutputEvent,
  previousRunId: string | null,
  receivedAtMs: number,
): PreparedTerminalOutput {
  const command = event.command?.trim();
  if (!command && !event.chunk.trim()) {
    return { lines: [], latestRunId: previousRunId };
  }

  const lines: SessionOutputLine[] = [];
  if (event.runId && previousRunId && event.runId !== previousRunId) {
    lines.push({
      stream: "stdout",
      chunk: "",
      receivedAtMs,
      runId: event.runId,
      commandSeparator: true,
    });
  }

  if (command) {
    lines.push({
      stream: "stdout",
      chunk: command,
      receivedAtMs,
      runId: event.runId,
      commandStart: true,
    });
  }

  if (event.chunk.trim()) {
    lines.push({
      stream: event.stream,
      chunk: event.chunk,
      receivedAtMs,
      runId: event.runId,
    });
  }

  return {
    lines,
    latestRunId: event.runId ?? previousRunId,
  };
}
