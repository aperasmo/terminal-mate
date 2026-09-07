import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import ts from "typescript";

const sourceUrl = new URL("../src/lib/terminalTranscript.ts", import.meta.url);
const source = await readFile(sourceUrl, "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2022,
  },
  fileName: "terminalTranscript.ts",
}).outputText;
const moduleUrl = `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`;
const { prepareTerminalOutput } = await import(moduleUrl);

const outputEvent = (runId, chunk, stream = "stdout") => ({
  sessionId: "session-1",
  runId,
  stream,
  chunk,
});

const commandEvent = (runId, command) => ({
  sessionId: "session-1",
  runId,
  command,
  stream: "stdout",
  chunk: "",
});

const blank = prepareTerminalOutput(outputEvent("run-2", "   "), "run-1", 100);
assert.deepEqual(blank, { lines: [], latestRunId: "run-1" });

const first = prepareTerminalOutput(outputEvent("run-1", "first"), null, 101);
assert.equal(first.lines.length, 1);
assert.equal(first.lines[0].commandSeparator, undefined);
assert.equal(first.latestRunId, "run-1");

const firstCommand = prepareTerminalOutput(
  commandEvent("run-command-1", "npm run build"),
  null,
  101,
);
assert.equal(firstCommand.lines.length, 1);
assert.equal(firstCommand.lines[0].chunk, "npm run build");
assert.equal(firstCommand.lines[0].commandStart, true);
assert.equal(firstCommand.lines[0].commandSeparator, undefined);

const nextCommand = prepareTerminalOutput(
  commandEvent("run-command-2", "npx shadcn@latest init"),
  "run-command-1",
  102,
);
assert.equal(nextCommand.lines.length, 2);
assert.equal(nextCommand.lines[0].commandSeparator, true);
assert.equal(nextCommand.lines[1].commandStart, true);
assert.equal(nextCommand.lines[1].chunk, "npx shadcn@latest init");

const sameRun = prepareTerminalOutput(
  outputEvent("run-1", "second", "stderr"),
  "run-1",
  102,
);
assert.equal(sameRun.lines.length, 1);
assert.equal(sameRun.lines[0].commandSeparator, undefined);

const nextRun = prepareTerminalOutput(
  outputEvent("run-2", "next command"),
  "run-1",
  103,
);
assert.equal(nextRun.lines.length, 2);
assert.equal(nextRun.lines[0].commandSeparator, true);
assert.equal(nextRun.lines[1].chunk, "next command");
assert.equal(nextRun.latestRunId, "run-2");

const sessionOutput = prepareTerminalOutput(
  outputEvent(null, "session ready"),
  "run-2",
  104,
);
assert.equal(sessionOutput.lines.length, 1);
assert.equal(sessionOutput.lines[0].commandSeparator, undefined);
assert.equal(sessionOutput.latestRunId, "run-2");

console.log("Terminal transcript preparation: 7 scenarios passed.");
