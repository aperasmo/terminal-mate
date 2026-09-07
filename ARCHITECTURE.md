# TerminalMate Architecture

Status: Phase 2 in progress - implemented architecture, updated from the
original Phase 0 proposal to reflect what actually shipped (WSL/Bash adapter,
cross-shell intent rendering, Explain Only execution mode, clarification
flow, and the realized Recovery Assistant)
Date: 25/07/2026, updated 03/08/2026

## Architectural Goal

TerminalMate must keep AI interpretation separate from command execution. The
AI identifies intent and may explain options, but only trusted application code
may translate an approved structured action into a shell command.

## High-Level Flow

```text
Plain-English request
        |
        v
Environment snapshot + AI intent planner
        |
        v
Structured command intent
        |
        v
Schema validation + policy classification
        |
        v
Platform and shell adapter
        |
        v
Command preview + proportional approval
        |
        v
Persistent terminal session execution
        |
        v
Output, audit record, and guided recovery
```

## Technology Direction

TerminalMate uses a local-first sidecar architecture, with a substantial role
for Rust:

- **Tauri:** desktop application shell and secure native bridge;
- **React and TypeScript:** workspace, terminal, approval, history, and
  settings interfaces;
- **Rust:** persistent terminal processes, environment detection, policy
  enforcement, command rendering, execution, cancellation, and output
  streaming;
- **Python and FastAPI sidecar:** natural-language interpretation, AI-provider
  adapters, explanations, and recovery suggestions;
- **Browser `localStorage` (webview-local):** the actual current persistence
  for workspace definitions, the active workspace, and command run history —
  a lighter mechanism than the SQLite store originally sketched in Phase 0,
  adopted because Phase 1/2 did not yet need cross-session querying or
  policies/workflows storage; SQLite remains the plan once that need arrives;
- **Session-scoped Rust state:** the current optional AI-provider endpoint,
  model, enabled state, and API key. The key is never returned to the frontend,
  logged, or persisted; OS credential storage remains the later design for
  durable credentials and SSH references;
- **PyInstaller:** packaged Python sidecar so end users do not install Python.

The terminal rendering component will display process output but will not own
security decisions. Rust remains the execution boundary.

## Core Components

### Workspace Manager

Owns:

- registered projects and aliases;
- default runtime and shell per workspace;
- local, WSL, remote, and container path mappings;
- terminal tab and split-pane layout;
- session restoration metadata.

### Session Manager

Creates and supervises persistent terminal sessions. A persistent process is
required so that directory changes, exported variables, activated environments,
and shell state survive between commands.

Each session has one immutable identity and a refreshable execution profile.

### Environment Profiler

Collects:

- host operating system and architecture;
- runtime type: local, WSL, SSH, container, or cloud shell;
- operating-system distribution and version where available;
- shell executable and version;
- current working directory;
- current user and privilege level;
- available command-line tools and package managers;
- active DevOps contexts, when explicitly enabled.

Environment data is refreshed at session creation, after context-changing
operations, and on demand.

### Intent Planner

Converts a natural-language request into a versioned structured object. It does
not return an executable command as its primary result.

Example:

```json
{
  "schemaVersion": 1,
  "action": "find_files",
  "parameters": {
    "root": ".",
    "pattern": "*.sh",
    "recursive": true
  },
  "targetSessionId": "session-01"
}
```

The planner receives only the minimum environment context required to
understand the request. TerminalMate first runs the deterministic local
matcher. If it cannot match and the user has enabled a provider, the sidecar
sends a constrained action catalogue and the execution profile to an
OpenAI-compatible endpoint. The model must return one allowlisted action with
typed parameters; unknown actions, extra fields, malformed parameters, and raw
shell commands are rejected. Rust independently validates and renders the
result for the active shell. The response carries a `source` of `"local"` or
`"ai"`, surfaced as `LOCAL MATCH` or `AI PLAN` before execution.

Provider failure does not bypass the boundary. The request remains subject to
the ordinary direct-command review path, and no model response is executed as
shell text. All rendered commands still pass through Rust policy
classification, proportional approval, and execution-time checks.

When a request is recognizable but missing a required detail (which file to
preview, which port to inspect, which Azure subscription or Terraform
resource address to use), the planner does not guess. It returns
`requiresClarification: true` with a plain-English prompt for the missing
detail (for example, *"Which port should I inspect? For example: 'Inspect
port 3000.'"*). Rust surfaces this the same way it surfaces an unmatched or
unreachable-sidecar request: as a rejection with the guidance message shown to
the user, never as a guessed command.

### Intent Validator

Rejects:

- unknown action types;
- missing or invalid parameters;
- paths outside the allowed scope;
- unsupported platform and action combinations;
- malformed shell arguments;
- requests that attempt to bypass the policy engine.

The Rust execution boundary accepts exactly one command per submission. The
frontend may decompose a multiline paste into a sequential queue, but it sends
each queue entry through resolution, validation, classification, approval, and
execution independently. It advances only after the matching prior run
succeeds and stops on a block, cancellation, user stop, or non-zero exit.
Recognized Bash `\` and PowerShell `` ` `` line-continuation characters are
folded into one logical command first, so realistic multi-line CLI invocations
(common with `az`, `kubectl`, and `terraform`) are not split accidentally.
Balanced PowerShell expressions are also kept intact. Variable assignments
followed by one `foreach` block or a range pipeline using `ForEach-Object` are
wrapped as one scoped script before queueing, preventing variables from being
lost between child processes. Ordinary dependent statements use the same
scope-preserving boundary: when a later line references a variable assigned
earlier in the paste, the related block is submitted to one reviewed
PowerShell process. Independent lines remain separate sequential commands.

Command completion also applies command-specific exit semantics before the
run is reported. In particular, `git check-ignore` exit code 1 with no output
means the path is not ignored, so it is converted into an informative success
result rather than generic failure guidance. Runtime-specific failures remain
explicit; Windows PowerShell 5.1's missing static
`RandomNumberGenerator.Fill` API produces a compatible, approval-gated retry
using `Create().GetBytes(...)`.

### Adapter Registry

Translates validated intents into commands for the active execution profile.

Implemented adapters:

- Windows with PowerShell;
- WSL with Bash — real WSL2 sessions, selectable and persisted independently
  per workspace alongside native Windows.

Every intent in the current catalogue — navigation, file inspection and
search, process/network inspection, Azure CLI, Terraform, and SSH key
management — renders through both adapters from the same intent, not through
a separate cloud- or tool-specific adapter. Azure CLI and Terraform support is
additional coverage inside the existing PowerShell/Bash adapters, not a new
adapter type; Docker, Kubernetes, and Helm are recognized only by the policy
engine today (risk-classified when typed directly), with no deterministic
plain-English intents yet.

Later adapters:

- native Linux distributions;
- remote Bash through SSH (beyond the SSH key-generation and connection
  commands the adapters already render);
- container shells;
- macOS with Zsh;
- deterministic plain-English intents for Docker, Kubernetes, and Helm.

Adapters own syntax differences. The AI must not guess whether a Linux,
PowerShell, or macOS command is appropriate.

### Policy Engine

Classifies the rendered operation and determines:

- whether it is supported;
- its risk level;
- whether it can run immediately;
- which approval control is required;
- whether extra context, such as the target Kubernetes cluster, must be shown;
- whether the operation is blocked.

The policy engine inspects the structured intent and rendered command. It does
not rely only on keyword matching. High Risk operations that are allowed to
execute require the user to type the literal word `RUN`, not just click an
approval control — a stronger confirmation than Caution's single-click
approval.

This is a separate mechanism from **Explain Only**, described below: Explain
Only is not a fifth risk level and does not go through the policy engine at
all. It is a hard-coded override keyed on the matched intent's action,
applied before classification, for a small set of actions whose risk is not
well captured by classifying the rendered command text alone (irreversible
infrastructure destruction, credential exposure). See Safety Model for the
current list.

### Execution Engine

Runs only commands that have:

1. a valid intent;
2. a supported adapter result;
3. a completed policy decision;
4. the required user approval — a single click for Caution, a typed `RUN`
   for High Risk.

It provides:

- persistent session input and output;
- cancellation and timeout controls;
- exit status;
- a bounded 1,000-line in-memory transcript for responsive workspace
  switching;
- complete per-run log files that can be opened or copied on demand;
- secret redaction;
- an immutable execution receipt.

On Windows, foreground child commands receive piped stdout and stderr rather
than a console handle or ConPTY. `spawn_line_pump` in
`src-tauri/src/commands/command_runs.rs` reads bytes through
`BufReader::read_until(b'\n')`; TerminalMate does not impose an internal
per-line size cap. Python children receive UTF-8, unbuffered output settings,
and decoded output falls back safely when a Windows-native tool emits non-UTF-8
bytes. Regression tests exercise the normal command path with a single Python
logging line longer than 50,000 characters.

Complete-log copying does not expand the in-memory transcript. The React UI
invokes the typed `read_log_file` Tauri command only after the user selects
**Copy log**. Rust canonicalizes and validates the requested path through
`terminal_mate_log_path`, accepts only TerminalMate-owned files under the app's
log directory, reads the complete file, and returns its text for an explicit
clipboard write. This preserves full long-running command output without
reintroducing the workspace-switching lag that the transcript bound prevents.

Explain Only intents never reach the execution engine. TerminalMate renders
the exact command and an explanation of why it is not offered to run, and
provides a copy-to-clipboard action instead of an approval control. This
applies only to intents matched from plain English; the same command typed
directly still goes through ordinary risk classification and approval
(so a literally-typed `az group delete` is High Risk with a typed-`RUN`
gate, not Explain Only — Explain Only is specifically about not trusting an
AI-interpreted plain-English request with the most destructive actions, not
about the command family in general).

`terraform apply`/`terraform destroy` are a deliberate exception worth
calling out: they are High Risk with a typed-`RUN` gate, not Explain Only,
even though they are just as destructive as the actions that are. The reason
is mechanical, not a relaxed risk judgment — Terraform's own interactive
confirmation prompt can never be satisfied by TerminalMate (no stdin channel
exists), so the adapter renders both with `-auto-approve` to avoid an
unanswerable hang, and TerminalMate's typed-`RUN` confirmation stands in as
the sole approval gate in place of Terraform's own.

The same reasoning applies when either command is typed directly rather than
matched from plain English: the policy engine blocks a bare `terraform
apply`/`destroy` with no `-auto-approve` and no pipe (which could be
deliberately supplying a "yes" answer), since it is not a risk judgment call
but a certainty that it would hang. Adding `-auto-approve` or piping a
confirmation makes it High Risk with the normal typed-`RUN` gate, same as
the intent-matched path.

Full-screen interactive programs (`nano`, `vim`, `emacs`, `top`, `htop`) are
blocked outright for the same "no PTY" reason, but with no rendering
workaround available — unlike Terraform's single confirmation prompt, these
need a real terminal for everything they do. Instead of trying to emulate
one, `commands/file_editor.rs` offers a narrower, purpose-built alternative
for the common case of wanting to edit a named file: a direct file
read/write, entirely outside the execution engine — no shell process, no
command classification, no PTY. It resolves the file's path against the
session's current working directory (converting a WSL-style working
directory back to its Windows equivalent first, via `wsl_path_to_windows`,
the reverse of the existing `windows_path_to_wsl` helper), enforces a 2 MB
size limit, and treats a missing file as an empty buffer — matching what
`nano <newfile>` itself would do. This is a safety improvement over trying
to support the original program, not a workaround with a weaker boundary:
there is no shell involved at all, only a bounded file write.

### Recovery Assistant

Receives sanitized failure output, execution profile, intent, and exit status.
It returns:

- a plain-English cause;
- checks that can confirm the diagnosis;
- one or more structured recovery intents;
- a warning when human investigation is safer.

Recovery actions go through the same validation, policy, and approval path as
the original action.

Implemented today as a deterministic (not AI-driven) heuristic classifier over
a failed command's captured output, recognizing: Azure sign-in required, an
Azure subscription unavailable to the active account/tenant, a missing path,
an unavailable command, a permission failure, a network failure, and a
generic nonzero-exit fallback. Each diagnosis carries a title, an
explanation, next steps, and sometimes a `suggestedRequest` — a plain-English
follow-up the UI can offer directly (for example, *"List Azure
subscriptions"* or *"Sign in to Azure with device code"*).

Recursive PowerShell discovery has a targeted permission recovery path. The
deterministic renderer uses `-ErrorAction SilentlyContinue` where the request
is known to be a recursive search, and an exact recursive command that fails on
an inaccessible folder can be retried with the same safe skip-inaccessible
behavior. Valid results remain visible while inaccessible paths are omitted.

The Azure-subscription case is the first fully guided recovery workflow: it
runs a safe read to list accessible subscriptions, lets the user pick one or
re-authenticate via device code, detects when re-authentication still returns
only the same rejected subscription (avoiding a retry loop and moving
straight to clearer account/tenant/administrator guidance instead), and
requires a fresh typed-`RUN` confirmation before retrying the original
failed command. Every step in that workflow — the subscription check, the
switch, the sign-in, the retry — goes through the normal intent,
policy, and approval path; the guidance only orchestrates which follow-up
request to offer next.

## Execution Profile

The host OS alone does not determine command syntax. Every command uses the
active session profile. This is the real, current shape (there is no
`capabilities` field — that was a Phase 0 sketch that was never built):

```json
{
  "hostOs": "windows",
  "runtime": "local",
  "runtimeName": "Windows",
  "targetOs": "windows",
  "shell": "powershell",
  "workingDirectory": "D:\\APE\\Portfolio-Project\\terminal-mate",
  "architecture": "x86_64",
  "privilege": "standard-user"
}
```

`runtime` is typed as `local | wsl | ssh | container | cloud` in the schema,
but only `local` (native PowerShell) and `wsl` (WSL Bash) are backed by a
real execution path today; the other three are modeled for future phases and
would currently fail with an explicit "unsupported runtime" error if reached.

Each workspace independently selects and persists its own runtime — native
Windows PowerShell or WSL Bash — through `configure_workspace_runtime`, which
rebuilds the execution profile (including remapping the working directory)
for the chosen runtime. If the session is WSL + Bash instead, the Bash
adapter is selected and every intent renders Bash syntax, even though the
desktop application itself is hosted by Windows.

## Workspace Path Resolution

A workspace stores one path: its Windows filesystem path. There is no
separate stored WSL/remote path mapping.

```text
Workspace alias: terminal-mate
Stored path:     D:\APE\Portfolio-Project\terminal-mate
```

When a workspace's runtime is set to WSL, the Windows path is converted to
its WSL equivalent on demand by a pure path-conversion helper
(`windows_path_to_wsl`) — `D:\APE\Portfolio-Project\terminal-mate` becomes
`/mnt/d/APE/Portfolio-Project/terminal-mate` — and that becomes the WSL
execution profile's working directory. Remote (SSH) path handling remains a
later-phase design, not yet implemented.

## Process Boundaries

```text
React UI
  | typed Tauri commands and events
Rust application core
  | local authenticated HTTP on a private port
Packaged Python AI sidecar
  | user-configured provider API or local model
AI provider
```

Rules:

- React cannot spawn arbitrary processes directly.
- Python cannot send commands directly to a terminal.
- The AI sidecar cannot bypass Rust validation or approval.
- Provider credentials never enter the command transcript.
- The sidecar listens only on loopback with a per-launch authentication token.

## Local Data

Implemented today, in the webview's `localStorage` (not SQLite — see
Technology Direction):

- workspace definitions, including each workspace's selected runtime;
- the active workspace selection;
- per-workspace command run history (the most recent runs, restored across
  restarts, selectable to restore the original request).

Complete command output is stored separately in TerminalMate-owned log files.
The centre transcript restores only its latest 1,000 lines; opening or copying
a log reads the complete file on demand.

Not yet implemented, still planned for SQLite once needed:

- non-secret execution profile history beyond the current run list;
- command intents and execution receipts as queryable records;
- risk decisions and approvals as an audit trail;
- saved workflows;
- privacy receipts.

Secrets are stored through operating-system credential protection and are
referenced by opaque identifiers — planned alongside AI-provider support, not
built yet.

## Test Strategy

The architecture requires:

- schema tests for every intent;
- unit tests for every adapter and risk rule;
- golden tests mapping the same intent across PowerShell, Linux, and macOS
  fixtures — implemented today for PowerShell and WSL Bash across the
  navigation, file, process, Azure CLI, and Terraform catalogue; native
  Linux and macOS fixtures remain later-phase work;
- path-conversion tests for Windows and WSL;
- terminal-session lifecycle and cancellation tests;
- adversarial tests for injection, obfuscation, traversal, and secret leakage;
- integration tests using temporary workspaces;
- end-to-end tests for the Phase 1 natural-language workflows.

No platform adapter is considered supported until its golden and integration
tests pass on that platform.
