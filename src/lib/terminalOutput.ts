import type { SessionOutputLine } from "./types";

export const MAX_VISIBLE_TERMINAL_LINES = 1_000;

export interface TerminalOutputBuffer {
  lines: SessionOutputLine[];
  hiddenLineCount: number;
}

export function appendTerminalOutput(
  current: TerminalOutputBuffer | undefined,
  incoming: SessionOutputLine[],
): TerminalOutputBuffer {
  const lines = current?.lines ?? [];
  const hiddenLineCount = current?.hiddenLineCount ?? 0;

  if (incoming.length === 0) {
    return current ?? { lines, hiddenLineCount };
  }

  if (incoming.length >= MAX_VISIBLE_TERMINAL_LINES) {
    return {
      lines: incoming.slice(-MAX_VISIBLE_TERMINAL_LINES),
      hiddenLineCount:
        hiddenLineCount +
        lines.length +
        incoming.length -
        MAX_VISIBLE_TERMINAL_LINES,
    };
  }

  const overflow = Math.max(
    0,
    lines.length + incoming.length - MAX_VISIBLE_TERMINAL_LINES,
  );

  return {
    lines:
      overflow > 0
        ? [...lines.slice(overflow), ...incoming]
        : [...lines, ...incoming],
    hiddenLineCount: hiddenLineCount + overflow,
  };
}
