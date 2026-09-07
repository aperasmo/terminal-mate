import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import ts from "typescript";

const sourceUrl = new URL("../src/lib/sequentialCommands.ts", import.meta.url);
const source = await readFile(sourceUrl, "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2022,
  },
  fileName: "sequentialCommands.ts",
}).outputText;
const moduleUrl = `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`;
const { parseSequentialCommands, shouldAutoApproveSequentialCommand } =
  await import(moduleUrl);

const uvCommands = [
  "uv run python -m py_compile scripts\\freeze_answer_candidate_v2.py",
  "uv run python -m scripts.freeze_answer_candidate_v2",
];
assert.deepEqual(
  parseSequentialCommands(uvCommands.join("\n"), "powershell"),
  uvCommands,
);

const fourCommands = [
  "uv run python -m py_compile scripts\\first.py",
  "uv run python -m scripts.first",
  "uv run python -m py_compile scripts\\second.py",
  "uv run python -m scripts.second",
];
assert.deepEqual(
  parseSequentialCommands(fourCommands.join("\n"), "powershell"),
  fourCommands,
);

assert.deepEqual(
  parseSequentialCommands("Get-ChildItem `\n  -Recurse", "powershell"),
  ["Get-ChildItem -Recurse"],
);
assert.deepEqual(
  parseSequentialCommands(
    [
      "Get-ChildItem backend\\app -Recurse -Filter *.py |",
      'Select-String -Pattern "OpenAI|AsyncOpenAI" |',
      "Select-Object Path,LineNumber,Line",
    ].join("\n"),
    "powershell",
  ),
  [
    'Get-ChildItem backend\\app -Recurse -Filter *.py | Select-String -Pattern "OpenAI|AsyncOpenAI" | Select-Object Path,LineNumber,Line',
  ],
);
assert.deepEqual(parseSequentialCommands("printf foo \\\n  bar", "bash"), [
  "printf foo bar",
]);

const foreachScript = [
  '$files = @("first.py", "second.py")',
  "foreach ($file in $files) {",
  "  Write-Output $file",
  "}",
].join("\n");
const parsedForeach = parseSequentialCommands(foreachScript, "powershell");
assert.equal(parsedForeach?.length, 1);
assert.match(parsedForeach?.[0] ?? "", /^& \{/);

const rangePipelineScript = [
  "$lines = Get-Content src\\index.css",
  "",
  "80..180 | ForEach-Object {",
  '  "{0,4}: {1}" -f ($_ + 1), $lines[$_]',
  "}",
].join("\n");
const parsedRangePipeline = parseSequentialCommands(
  rangePipelineScript,
  "powershell",
);
assert.equal(parsedRangePipeline?.length, 1);
assert.match(parsedRangePipeline?.[0] ?? "", /^& \{/);
assert.match(parsedRangePipeline?.[0] ?? "", /\$lines = Get-Content/);
assert.match(parsedRangePipeline?.[0] ?? "", /80\.\.180 \| ForEach-Object/);

const scopedEnvironmentScript = [
  '$env:CORS_ORIGIN="https://waypoint.example.com"',
  "Set-Location .\\backend",
  'uv run python -c "from app.config import get_settings; print(get_settings().cors_origin)"',
  "Set-Location ..",
  "Remove-Item Env:CORS_ORIGIN",
].join("\n");
const parsedEnvironmentScript = parseSequentialCommands(
  scopedEnvironmentScript,
  "powershell",
);
assert.equal(parsedEnvironmentScript?.length, 1);
assert.match(parsedEnvironmentScript?.[0] ?? "", /^& \{/);
assert.match(
  parsedEnvironmentScript?.[0] ?? "",
  /Remove-Item Env:CORS_ORIGIN/,
);

const dockerInspectionScript = [
  "$container = (docker inspect waypoint-db | ConvertFrom-Json)[0]",
  "",
  '$container.Mounts | Where-Object { $_.Destination -eq "/var/lib/postgresql/data" } | Select-Object Type, Name, Source, Destination',
].join("\n");
const parsedDockerInspection = parseSequentialCommands(
  dockerInspectionScript,
  "powershell",
);
assert.equal(parsedDockerInspection?.length, 1);
assert.match(parsedDockerInspection?.[0] ?? "", /^& \{/);
assert.match(parsedDockerInspection?.[0] ?? "", /\$container\.Mounts/);

const randomPasswordScript = [
  "$bytes = New-Object byte[] 32",
  "[System.Security.Cryptography.RandomNumberGenerator]::Fill($bytes)",
  "$password = [Convert]::ToBase64String($bytes).TrimEnd('=').Replace('+','-').Replace('/','_')",
  "$password",
].join("\n");
const parsedRandomPassword = parseSequentialCommands(
  randomPasswordScript,
  "powershell",
);
assert.equal(parsedRandomPassword?.length, 1);
assert.match(parsedRandomPassword?.[0] ?? "", /^& \{/);

const restRequestScript = [
  "$body = @{",
  '  question = "Can I work 40 hours during university holidays on my student visa?"',
  "} | ConvertTo-Json -Compress",
  '$response = Invoke-RestMethod -Uri "http://localhost:8100/ask" -Method Post -ContentType "application/json" -Body $body',
  "$response | ConvertTo-Json -Depth 10",
].join("\n");
const parsedRestRequest = parseSequentialCommands(
  restRequestScript,
  "powershell",
);
assert.equal(parsedRestRequest?.length, 1);
assert.match(parsedRestRequest?.[0] ?? "", /^& \{/);
assert.match(parsedRestRequest?.[0] ?? "", /-Body \$body/);

assert.deepEqual(
  parseSequentialCommands("$value = 1\nnpm run build", "powershell"),
  ["$value = 1", "npm run build"],
);

assert.equal(
  shouldAutoApproveSequentialCommand("autoApproveCaution", "caution"),
  true,
);
for (const level of ["safe", "highRisk", "blocked"]) {
  assert.equal(
    shouldAutoApproveSequentialCommand("autoApproveCaution", level),
    false,
  );
}
assert.equal(
  shouldAutoApproveSequentialCommand("reviewEach", "caution"),
  false,
);

console.log(
  "Sequential command parser and approval policy: 15 scenarios passed.",
);
