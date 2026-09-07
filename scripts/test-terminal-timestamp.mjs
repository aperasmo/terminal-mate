import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import ts from "typescript";

const sourceUrl = new URL("../src/lib/terminalTimestamp.ts", import.meta.url);
const source = await readFile(sourceUrl, "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2022,
  },
  fileName: "terminalTimestamp.ts",
}).outputText;
const moduleUrl = `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`;
const { formatTerminalTimestamp } = await import(moduleUrl);

const morningTimestamp = new Date(2026, 7, 21, 5, 6, 9).getTime();
assert.equal(formatTerminalTimestamp(morningTimestamp), "21/08/2026 05:06:09");

const afternoonTimestamp = new Date(2026, 7, 21, 17, 46, 3).getTime();
assert.equal(
  formatTerminalTimestamp(afternoonTimestamp),
  "21/08/2026 17:46:03",
);

console.log("Terminal timestamp formatter: 2 scenarios passed.");
