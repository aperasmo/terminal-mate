# TerminalMate

TerminalMate is an organized, safety-first terminal workspace for developers,
students, and technical users who know what they want to accomplish but may not
know the correct commands.

The application keeps projects and persistent terminal sessions together, turns
plain-English requests into structured intents, selects commands for the active
environment, and applies proportional safety controls before execution.

## Current Status

Phase 2 WSL and infrastructure-intent milestone, version `0.2.1`.

The initial scaffold includes:

- Tauri 2 desktop shell;
- React and TypeScript workspace interface;
- native folder selection and execution-profile detection;
- locally persisted workspace registration with editable aliases;
- multiple independent workspace entries for the same folder, useful when one
  path needs different names, runtimes, terminal sessions, or run histories;
- Python 3.12 FastAPI sidecar;
- authenticated health and structured-intent endpoints;
- deterministic plain-English intents for navigation, file discovery and
  inspection, folder creation, text search, process inspection, and port
  inspection;
- an optional local-first generative AI planner for requests that the trusted
  deterministic matcher cannot understand; AI may select only a validated
  action and typed parameters, never supply executable shell text;
- a visible **AI planner: On/Off** control in the top bar, with session-only
  OpenAI-compatible provider settings and clear `LOCAL MATCH` / `AI PLAN`
  provenance before execution;
- persistent PowerShell terminal sessions with a live working directory;
- a persisted runtime selector for each workspace, allowing one workspace to
  use native Windows PowerShell while another uses WSL Bash;
- real WSL2 Bash sessions, Windows-to-WSL working-directory mapping, and
  shell-specific deterministic intent rendering;
- deterministic Azure CLI and Terraform intents with typed parameters,
  PowerShell/Bash rendering, and Azure sign-in prerequisite checks;
- a cross-shell Azure/Terraform regression matrix that locks supported
  plain-English requests to deterministic PowerShell and Bash commands;
- direct exact-command input through the same environment checks, safety
  classification, approval controls, execution logs, and recovery guidance;
- multiline paste support for Bash `\`, PowerShell backtick continuations,
  balanced PowerShell expressions, variable-backed `foreach` and
  `ForEach-Object` scripts, ordinary variable-backed PowerShell workflows,
  process-scoped PowerShell environment workflows, and independent sequential
  command queues;
- searchable in-app Help covering everyday, Azure, Terraform, and SSH requests,
  with safety labels and one-click reuse in the active command line;
- command-line Help shortcuts: `Help`, `Help Azure`, `Help Terraform`, and
  `Help SSH` open the matching reference directly; unsupported topics such as
  `Help AWS` are identified clearly instead of being sent for execution;
- an Explain Only boundary for sensitive infrastructure operations, showing
  the exact command and rationale without offering an execution button;
- proportional command policy classification and explicit approval for
  caution and high-risk actions;
- foreground command execution with live output, log files, cancellation, and
  workspace-specific run history that persists across app restarts;
- Windows child-process output captured through stdout/stderr pipes with UTF-8
  Python output, byte-safe fallback decoding, and regression coverage for
  single log lines larger than 50,000 characters;
- responsive workspace switching through batched terminal updates and a
  bounded 1,000-line visible transcript, while complete foreground-command
  output remains available in its log file;
- live command elapsed time, final run durations, clickable Recent runs that
  restore the original request to the command line, and one-click copying of
  the complete current command log;
- plain-English recovery guidance for missing paths, unavailable commands,
  permission failures, network failures, unavailable Azure subscriptions, and
  unknown nonzero exits;
- semantic command outcomes such as `git check-ignore` exit code 1 with no
  output being reported as "Path is not ignored by Git" instead of a failure;
- Windows PowerShell 5.1 compatibility guidance for unavailable APIs such as
  `RandomNumberGenerator.Fill`, with a reviewed `Create().GetBytes(...)` retry;
- permission-tolerant recursive PowerShell discovery that can skip inaccessible
  cache or system folders instead of losing all valid search results;
- guided Azure subscription recovery that safely inspects accessible
  subscriptions, prevents same-subscription retry loops, routes subscription
  changes and device-code sign-in through normal approval, and requires a new
  High Risk confirmation before retrying the original command;
- sidecar and Rust integration tests;
- generated application icons for Windows, Linux, and future macOS packaging;
- Windows NSIS installer with the packaged local sidecar.

TerminalMate is still an early build. The latest 10 runs are retained
for each registered workspace across app restarts. Each open terminal keeps
its latest 1,000 lines visible to prevent large build logs from slowing
workspace switching. Select a Recent run to restore its original request to
the command line. Visible output lines include a local `dd/mm/yyyy HH:mm:ss`
timestamp using 24-hour time. A separator marks each new command, and
each command is shown in green before its output so queued and individual
runs remain easy to identify. Whitespace-only output rows are omitted from
the visible transcript. These display changes are applied only by the UI and
are never written to command log files. After a command finishes, **Open log**
opens its complete on-disk
output and **Copy log** copies that complete output rather than only the
visible transcript. Failed commands receive deterministic recovery guidance.
Known Azure subscription failures can continue as a guided workflow: safe
diagnostic reads run automatically, while context changes, sign-in, and the
original infrastructure command retain their normal approval requirements.

Named-path listing requests validate the target first. TerminalMate labels the
target as a file or folder, lists children for a folder, and shows metadata for
a file.

Use the **Windows** / **WSL** selector in Environment Context to change the
active workspace runtime. The choice belongs to that workspace, persists after
restart, and starts a matching PowerShell or Bash session. TerminalMate checks
that WSL is available before accepting WSL mode.

The command line's Back and Forward buttons follow folder-history behavior,
not parent/child guessing. History is retained separately for each workspace
and runtime across app restarts. At a newly registered workspace root, both
buttons are disabled until navigation creates history. After visiting a child
folder and returning to the root, Forward remains available for that visited
folder; entering a new folder after going Back replaces the abandoned forward
branch.

The optional generative planner is configured from **AI planner: Off** or the
Settings button in the top bar. TerminalMate always tries its deterministic
local matcher first. Only an unmatched plain-English request is sent to the
configured OpenAI-compatible endpoint. The returned action is checked against
the same allowlist and parameter schemas, rendered by trusted Rust code for the
active PowerShell or WSL environment, and passed through the existing safety
classification and approval flow. The API key stays in application memory for
the current run and is not written to browser storage or project files.

Use the pencil beside a saved workspace to edit its alias. An alias identifies
the saved terminal context, not the folder itself, so the same path can be
registered more than once. For example, `waypoint/backend` can be saved as both
`backend` and `docker-backend`; each entry keeps its own runtime, terminal
session, warning state, and Recent runs. When a folder is registered again,
TerminalMate gives it a temporary unique alias such as `backend (2)` until it
is renamed.

## Supported Plain-English Requests

Current examples:

- `Where am I?`
- `Take me to the terminal-mate folder.`
- `Go to \backend\tests\` (normalized to `backend\tests` inside the active workspace)
- `Go up one directory.`
- `List the files here.`
- `Show hidden files with name and mode.`
- `List all files and folders, including hidden items, and show their name and mode.`
- `List the folders here.`
- `List files .claude in this folder.`
- `List .claude in this folder.`
- `List folders only.`
- `List folder only.`
- `Show me all the .sh files.`
- `Show all .py.`
- `Find the file package.json.`
- `Show me the contents of package.json.`
- `Show the first 30 lines of README.md.`
- `Show the last lines of app.log.`
- `Preview first/last lines of .env.`
- `Preview first/last lines .env.`
- `Show me lines 20-100 of sample.txt.`
- `Show file info for package.json.`
- `Count all files in this folder recursively.`
- `Search for TODO in this folder.`
- `Create a folder called drafts.`
- `Create folders components, api, services.`
- `Create file freeze_holdout_source_sample.py.`
- `Create file \scripts\freeze_holdout_source_sample.py.`
- `Create files src/index.ts, src/app.ts, README.md.`
- `Delete file scripts/freeze_holdout_source_sample.py.`
- `Copy file freeze_external_adjudication.py from D:\Data\Downloads to \scripts.`
- `Copy file D:\Data\Downloads\freeze_external_adjudication.py to \scripts.`
- `Cut file D:\Data\Downloads\freeze_external_adjudication.py to \scripts.`
- `Back up app/api/routes/ask.py as app/api/routes/ask_before_evidence_adequacy.py.`
- `Replace app/api/routes/ask.py with D:\Data\Downloads\ask_evidence_adequacy_candidate.py.`
- `Back up app/api/routes/ask.py as app/api/routes/ask_before_evidence_adequacy.py, then replace the original with D:\Data\Downloads\ask_evidence_adequacy_candidate.py.`
- `Show me the running processes.`
- `Show listening ports.`
- `What is using port 3000?`
- `Inspect port 3000.`
- `Inspect a port 8000.`
- `Find port 8100.`
- `Locate port 8100.`
- `Show port 8100.`
- `Find process using port 8100.`
- `Which process owns port 8100?`
- `Run scripts/seed_data.py.`
- `Run python collect_manual.py --prefixes R --out ../generic-residence`
- `Run all scripts in this folder.`
- `Run all scripts in this folder by name.`
- `Run all scripts in this folder oldest first.`
- `Run all Python scripts recursively.`
- `Show script execution order in this folder.`

Guided copy and cut actions accept either a complete source file path or a
file name plus a source folder. Their destination must be a folder inside the
active workspace; a leading slash or backslash is treated as workspace-relative.
TerminalMate verifies that the source is a file and the destination is a folder,
and refuses to overwrite an existing destination file. `Cut` maps to a guarded
move, so the source is removed only after the approved move succeeds.

Plural folder and file creation accepts comma-delimited workspace-relative
paths. TerminalMate removes duplicate paths, validates the complete list, and
checks that none of the targets already exists before creating anything. The
whole request remains one reviewed Caution action in PowerShell or WSL Bash.

Backup and replacement are separate explicit intents. Backup paths must stay
inside the active workspace and must not already exist. Replacement sources
may be supplied as complete external file paths, while the target must be an
existing workspace file. Replacement is always High Risk and requires typed
`RUN` confirmation because it overwrites the target. The combined workflow
validates the original, backup location, and replacement source before it
creates the backup and then replaces the original. Natural-language requests
may omit `as` and `with` when the path order remains unambiguous.

Script execution uses the active workspace runtime. PowerShell workspaces
support PowerShell, batch, Python, JavaScript, TypeScript, Ruby, PHP, Perl,
Lua, R, and Julia scripts. WSL workspaces support shell scripts plus Python,
JavaScript, TypeScript, Ruby, PHP, Perl, Lua, R, and Julia. Batch runs are
sequential, stop at the first failure, and require approval. The default order
is newest modified first; `by name`, `oldest first`, and `recursively` make the
ordering and discovery scope explicit. Use `Show script execution order in
this folder` to preview the exact order without running the scripts.
Python commands inherit unbuffered output in both PowerShell and WSL, and
generated Python script plans use `python -u` or `python3 -u`. Progress printed
by long-running Python scripts therefore appears in TerminalMate while the
script is still running instead of arriving only after it exits.
Single-script requests can include interpreter-style arguments. TerminalMate
extracts them into the reviewed `run_script` plan and quotes each argument as
a literal for the active PowerShell or WSL runtime instead of sending the word
`run` to the shell.

Azure CLI examples:

- `Show Azure CLI version.`
- `Show my Azure account.`
- `List Azure subscriptions.`
- `Sign in to Azure with device code.`
- `Clear my Azure login.`
- `Switch Azure subscription to <subscription-id>.`
- `Check if the Storage provider is registered.`
- `Register the Storage provider.`
- `Create Azure resource group <name> in <location>.`
- `List Azure VMs.`
- `Show Azure VM <name> in resource group <group>.`
- `Start Azure VM <name> in resource group <group>.`

Terraform examples:

- `Show Terraform version.`
- `Terraform init.`
- `Terraform fmt.`
- `Terraform validate.`
- `Terraform plan.`
- `List Terraform state.`
- `Show Terraform state <resource-address>.`
- `Show Terraform outputs.`

Azure storage-key retrieval and Azure resource-group deletion are
**Explain Only** in this release. TerminalMate displays the exact manual
command and explains why it is not executed, but never provides an approval
or execute action for it. High-risk supported operations require the user to
type `RUN` before execution.

`Terraform apply`/`Terraform destroy` execute through that same typed-`RUN`
High Risk path rather than Explain Only. TerminalMate renders both with
`-auto-approve`, since Terraform's own interactive confirmation prompt has
no way to be answered inside TerminalMate (there is no channel to send input
to a running command) and would otherwise hang indefinitely after printing
the plan. TerminalMate's typed-`RUN` confirmation is the approval gate in
place of Terraform's own — review the plan output before typing `RUN`.

Typing `terraform apply`/`terraform destroy` directly, without
`-auto-approve` or a piped confirmation, is refused with an explanation
rather than left to hang the same way — add `-auto-approve` (e.g.
`terraform apply -auto-approve`) or pipe a confirmation (e.g.
`echo yes | terraform apply`) to run it directly.

Full-screen interactive editors and monitors (`nano`, `vim`/`vi`, `emacs`,
`pico`, `top`, `htop`) are refused outright, with an explanation. TerminalMate
has no terminal (PTY) attached to a running command — no cursor addressing,
no screen redraw, no keystroke input — so these cannot function at all, not
just behave oddly. To view a file, use `cat <file>` or ask "show me the
contents of `<file>`"; to edit one, use a code editor outside TerminalMate.

When the blocked command names a file directly (`nano main.tf`, `vim
infra/azure/main.tf`), the refusal includes an **Open in built-in editor**
button. This opens the file in a small in-app text editor — a direct
read/write of the file, no shell or terminal involved — with a Save button
that overwrites it. It works the same way for a native Windows workspace or
a WSL one; a WSL working directory is resolved back to its Windows path
before the file is touched.

## Exact Command Input

TerminalMate also accepts valid terminal commands directly. Exact commands do
not bypass the safety boundary: they are classified and receive the same
automatic, approval, typed `RUN`, blocked, or Explain Only treatment as a
plain-English request.

For example, this Bash-style continued Azure CLI command can be pasted into
the command line:

```bash
az group create \
  --name glaucoma-ai-tfstate-rg \
  --location australiaeast
```

TerminalMate normalizes it to one command before showing the safety preview:

```text
az group create --name glaucoma-ai-tfstate-rg --location australiaeast
```

PowerShell backtick continuations are supported in the same way. A PowerShell
pipeline may also continue on the next line after a trailing `|`; TerminalMate
joins the pipeline before planning it. TerminalMate also accepts a naturally
formatted PowerShell expression while parentheses,
brackets, or braces remain balanced, for example:

```powershell
[Environment]::SetEnvironmentVariable(
  "Path",
  "C:\Users\Allan\.local\bin;" + [Environment]::GetEnvironmentVariable("Path", "User"),
  "User"
)
```

A cohesive PowerShell inspection script can include variable assignments
followed by one `foreach` block or a range pipeline using `ForEach-Object`.
TerminalMate preserves the shared PowerShell scope and submits the whole block
to one safety review:

```powershell
$temp = (Get-Content .tmp\collector-validation-manifest.json -Raw | ConvertFrom-Json).pages
$prod = (Get-Content data\manifest.json -Raw | ConvertFrom-Json).pages

foreach ($code in @("R2.40", "U8.25")) {
    $t = $temp | Where-Object section_code -eq $code
    $p = $prod | Where-Object section_code -eq $code

    [PSCustomObject]@{
        Section = $code
        Match   = $t.content_hash -eq $p.content_hash
    }
}
```

The same shared-scope rule applies to ordinary dependent statements. For
example, assigning Docker inspection data to `$container` and reading
`$container.Mounts`, or preparing `$body` before `Invoke-RestMethod`, remains
one reviewed PowerShell script. TerminalMate does not split those statements
into child processes where the variables would be lost.

On Windows PowerShell 5.1, the static
`RandomNumberGenerator.Fill(...)` API is unavailable. If that runtime-specific
failure occurs, TerminalMate explains the compatibility issue and offers an
equivalent `RandomNumberGenerator.Create().GetBytes(...)` command for review.

For example, this assignment and line-range formatter remain one command, so
`$lines` is still defined when the pipeline runs:

```powershell
$lines = Get-Content src\components\BrowseSectionDetail.jsx

100..115 | ForEach-Object {
    "{0,4}: {1}" -f ($_ + 1), $lines[$_]
}
```

Independent commands separated by bare newlines become a visible sequential
queue. TerminalMate runs them one at a time and applies the complete resolver,
environment validation, safety classification, and approval flow to every
entry. Before execution starts, the queue asks whether to review each approval
or auto-approve Caution commands for that pasted batch. `Review each` is the
focused default. Batch auto-approval never applies to High Risk commands,
which still require typed `RUN`, and it never bypasses blocked decisions. The
queue stops immediately when a command is blocked, cancelled, stopped, or
fails, so later commands never run after an unsafe or unsuccessful step.

For example, this paste creates a two-command queue rather than combining both
commands into one shell expression:

```text
git diff --numstat -- scraper/collect_manual.py
git diff --stat -- scraper/collect_manual.py
```

The same rule applies to compile-then-run workflows. The second command starts
only after the first command exits successfully:

```text
uv run python -m py_compile scripts\freeze_answer_candidate_v2.py
uv run python -m scripts.freeze_answer_candidate_v2
```

Larger pastes, including four or more unique commands, follow the same ordered
one-at-a-time execution and stop-on-failure behavior.

This slice focuses on common local developer tasks. SSH and broader DevOps
workflows remain later roadmap phases. Git-specific guided workflows are also
deferred so TerminalMate stays focused on its terminal-first scope.

## Windows Installer

Build the Windows installer from the project root:

```text
npm run installer:windows
```

Output:

```text
src-tauri/target/release/bundle/nsis/TerminalMate_0.2.1_x64-setup.exe
```

This installer is intended for functional review and testing:

- the three-column workspace layout;
- project registration and selection;
- environment, safety, and recent-run context;
- independent Windows PowerShell / WSL Bash runtime selection per workspace;
- persistent terminal sessions;
- command-bar positioning, wording, and visual hierarchy;
- behavior at different window sizes.

## Product Documents

- [Product brief](PRODUCT_BRIEF.md)
- [Architecture](ARCHITECTURE.md)
- [Safety model](SAFETY_MODEL.md)
- [Roadmap](ROADMAP.md)
- [Project status](TERMINAL-MATE-PROJECT_STATUS.md)

## Development

Requirements:

- Node.js 20 or newer;
- Rust toolchain compatible with Tauri 2;
- Python 3.12 or newer for sidecar development;
- Tauri platform prerequisites for the host operating system.

Install frontend dependencies:

```text
npm install
```

Run the browser interface:

```text
npm run dev
```

Run the Tauri desktop shell:

```text
npm run tauri:dev
```

Install and run sidecar tests:

```text
npm run sidecar:test:install
npm run sidecar:test
```

Validation commands:

```text
npm run build
npm run sidecar:test
cd src-tauri
cargo check
cargo test
```

The automated suite currently includes 122 sidecar tests and 176 Rust library
tests. Azure and Terraform coverage verifies both natural-language intent
matching and exact PowerShell/Bash command rendering.
