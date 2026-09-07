import type { CommandHistoryEntry } from "./types";

/**
 * How many runs are actually kept in persisted history per workspace. This
 * is the durable record, deliberately far larger than what the compact
 * "Recent runs" panel displays at once: localStorage easily holds this many
 * small entries (well under a megabyte even for many workspaces), and
 * losing history just because the panel is small would defeat the point of
 * persisting it at all. The true permanent record is each run's own
 * on-disk log file, which TerminalMate never deletes — this cap only
 * bounds the convenience index used for the in-app panel and "run again"
 * shortcuts, not the actual audit trail.
 */
export const HISTORY_STORAGE_LIMIT = 200;

/** How many rows the "Recent runs" panel shows at once. */
export const HISTORY_DISPLAY_LIMIT = 10;

export interface DisplayRunEntry {
  key: string;
  entry: CommandHistoryEntry;
  repeatCount: number;
}

/**
 * Collapses consecutive runs of the identical literal command into one
 * display row carrying a repeat count, so re-running the same command
 * several times in a row doesn't crowd the compact "Recent runs" panel out
 * with copies of itself. `history` is expected newest-first, and the
 * collapsed row keeps the newest occurrence's status/timestamp/log — the
 * most relevant one to show.
 *
 * Deliberately only collapses *consecutive* repeats, not every occurrence
 * anywhere in history — the same command run again later, after something
 * else happened in between, is a separate, meaningful event and stays its
 * own row. This mirrors how shell history deduplication conventionally
 * works (e.g. bash's `HISTCONTROL=ignoredups`), not a global merge that
 * would hide a distinct later failure or retry.
 */
export function collapseConsecutiveDuplicateRuns(
  history: CommandHistoryEntry[],
): DisplayRunEntry[] {
  const collapsed: DisplayRunEntry[] = [];

  for (const entry of history) {
    const last = collapsed[collapsed.length - 1];
    if (last && last.entry.command === entry.command) {
      last.repeatCount += 1;
      continue;
    }
    collapsed.push({ key: entry.id, entry, repeatCount: 1 });
  }

  return collapsed;
}
