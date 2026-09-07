# TerminalMate Project Status

Last updated: 31/08/2026

## Current Release

- Phase: 2 - WSL / Linux Subsystem Support
- Version: `0.2.1`
- Status: Native PowerShell and real WSL Bash sessions are selectable and
  persisted independently per workspace. Structured intents render for the
  selected shell, Windows paths map to WSL paths, and execution remains behind
  the same policy and approval boundary. Deterministic Azure CLI and Terraform
  intents now add typed parameters, cross-shell rendering, prerequisite
  guidance, and an Explain Only boundary for sensitive infrastructure work.
  Azure resource-provider registration checks and a stale-login recovery
  command close out the last known gaps against the original intent
  specification. Streamed and logged command output is now stripped of ANSI
  color escape codes, since TerminalMate displays plain text rather than
  emulating a real terminal. Terraform apply/destroy now execute through the
  same typed-RUN High Risk path as other destructive operations, rendered
  with -auto-approve since Terraform's own interactive confirmation cannot
  be answered without a stdin channel. A directly-typed apply/destroy
  lacking that flag or a piped confirmation is now blocked with guidance
  instead of hanging. Full-screen interactive editors and monitors (nano,
  vim, emacs, top, htop) are refused outright with an explanation, since
  TerminalMate has no terminal attached to a running command. When a file
  path can be read off the blocked command, a built-in editor (direct file
  read/write, no shell or PTY) offers a working alternative. A bare
  interactive `ssh user@host` login is blocked the same way (it needs a
  real terminal exactly like a full-screen editor does), and both that case
  and monitors like `top`/`htop` now offer "Open in a new terminal window"
  instead — the same command, handed off to a real external console
  TerminalMate does not supervise, rather than emulating one internally.
  The deterministic `ssh_vm` intent also renders with `BatchMode=yes` and
  `StrictHostKeyChecking=accept-new`, so it fails fast instead of hanging on
  a first-connection host-key or password prompt.

## Phase 2 fix - Stateful PowerShell paste and semantic Git outcomes - 28/08/2026

Completed:

1. Kept ordinary dependent PowerShell statements in one reviewed child process
   when later lines reference variables assigned earlier in the paste.
2. Added regressions for Docker inspection, REST request bodies, and random
   password generation while retaining the stricter `foreach` boundary.
3. Interpreted an empty `git check-ignore` exit code 1 as "Path is not ignored
   by Git" instead of a generic failed command.
4. Added Windows PowerShell 5.1 guidance for the unavailable static
   `RandomNumberGenerator.Fill` API and a compatible approval-gated
   `Create().GetBytes(...)` retry.
5. Updated in-app Help, README, and architecture documentation.

Validation:

- Sequential command parser: 15 scenarios passed.
- Rust intent tests: 31 passed.
- Rust command-run tests: 35 passed.

## Phase 2 Slice - Editable Workspace Aliases - 27/08/2026

Completed:

1. Added inline alias editing from the workspace sidebar with save, cancel,
   keyboard escape, validation, and persisted names.
2. Allowed the same canonical folder to be registered as multiple independent
   workspace entries.
3. Kept runtime selection, terminal session, warning state, and Recent runs
   isolated by workspace ID even when entries share one path.
4. Added automatic unique aliases such as `backend (2)` when a folder is
   registered repeatedly, ready for the user to rename.
5. Documented the alias model in README and in-app Help.

## Phase 0 - Product Blueprint - 25/07/2026

Completed:

1. Defined developers, students, and technical users as the initial audience.
2. Added DevOps usability as an explicit advanced product consideration.
3. Defined the one-window workspace and persistent-session product experience.
4. Separated AI intent planning from deterministic command translation.
5. Defined execution profiles for Windows, WSL, Linux, macOS, remote, and
   container sessions.
6. Defined Safe, Caution, High Risk, and Blocked operations.
7. Parked macOS packaging until a build and signing environment is available.

## Phase 1 - Windows Workspace MVP - 25/07/2026

Completed in the foundation scaffold:

1. Added the Tauri, React, TypeScript, Rust, and FastAPI project structure.
2. Added native folder selection and typed execution profiles.
3. Added local workspace registration and selection in the interface.
4. Added the initial one-window workspace layout.
5. Added authenticated sidecar health and structured-intent endpoints.
6. Added deterministic intent matching for initial navigation and inspection
   requests.
7. Added sidecar tests.
8. Added TerminalMate application icons for Windows, Linux, and future macOS
   packaging.
9. Validated the frontend production build, Python sidecar tests, and Rust
   desktop core.
10. Added a Windows NSIS UI preview installer and one-command packaging script.
11. Added a visible UI Preview marker so the review build cannot be confused
    with the functional terminal MVP.

Validation:

- `npm run build`: passed
- `npm run sidecar:test`: 5 passed
- `cargo check`: passed
- `cargo test`: 2 passed
- `npm run installer:windows`: passed
- Installer: `TerminalMate_0.1.0_x64-setup.exe` (1.87 MB)
- Installer SHA-256:
  `BEF4646681330A5E93446D1ED435D5090494E87D91387A142035D7B8C34CFB52`
- Browser screenshot inspection: unavailable because no in-app browser was
  connected in this session

See the dated slice below for what shipped next in this phase.

## Phase 1 slice - Persistent terminal sessions - 25/07/2026

Completed:

1. Added a Rust `SessionManager` that spawns one persistent
   `powershell.exe` process per terminal session, keyed by session ID, so
   working directory and shell state survive between commands.
2. Added a command-completion protocol: each command is followed by a
   unique boundary sentinel written to stdout so Rust can detect when a
   command finished and capture its exit code, without needing a full PTY.
3. Added live streaming of stdout/stderr lines to the frontend via a
   `session-output` Tauri event, plus `create_session` / `close_session`
   Tauri commands and cleanup on session close or workspace removal.
4. Wired the existing "+" / close (X) controls in the terminal workspace
   tab to actually create and close a session, and added a scrolling
   output log view that replaces the placeholder empty state once a
   session is live.
5. On creation, each session runs one Safe, read-only command
   (`Get-Location`) to prove the session is live end-to-end before it is
   handed back to the UI — no mutating or free-text command execution is
   wired up yet; the command bar still only previews.

Validation:

- `cargo test` (src-tauri): 5 passed, including a real integration test
  that spawns an actual `powershell.exe`, runs a command through the
  boundary protocol, asserts the streamed output arrives, and confirms
  session close removes it from the manager.
- `npm run build`: passed.
- Manual desktop verification was partial: the packaged dev app was
  confirmed to launch cleanly via a window screenshot (workspace sidebar,
  empty terminal state, command bar all rendered correctly), and the
  "Add workspace" button was confirmed reachable via accessibility
  automation. Clicking through the native folder picker to a live session
  was not completed in this session — the automation attempt was stopped
  partway after a full-desktop screenshot incidentally captured unrelated
  windows, which was flagged and abandoned rather than continued. A quick
  manual click-through (`npm run tauri:dev` -> Add a workspace -> click +)
  is still recommended before relying on this slice.

Next:

1. Add the first trusted Windows PowerShell command adapter that renders
   the already-matched Safe intents (change directory, list/find files by
   extension, view a line range of a file) into real PowerShell commands
   run through the session.
2. Add policy classification and approval receipts.
3. Connect the plain-English command bar to the sidecar intent endpoint.
4. Execute only validated Safe intents through the command bar.
5. Add cancellation, history, and basic recovery guidance.

## Phase 1 slice - Direct command entry with policy classification - 25/07/2026

Deliberately reordered from the "Next" list above: policy classification and
direct (literal, not plain-English) command execution now come before the
NLP intent adapter and sidecar wiring. Rationale: a working, safety-respecting
terminal (type an exact PowerShell command, it gets classified and runs) is
more valuable to prove out first than plain-English translation, and every
later feature (the intent adapter, the sidecar command bar) needs the same
classification + execution boundary this slice adds. The command bar still
does not call the sidecar's NLP intent endpoint — that remains future work.

Completed:

1. Added a Rust command classifier (`commands/policy.rs`) that assigns every
   typed command a Safe / Caution / High Risk / Blocked decision using
   pattern rules grounded in `SAFETY_MODEL.md`'s documented examples (e.g.
   recursive+forced deletion of a bare drive root is Blocked, a scoped
   delete is High Risk, an ordinary Git write is Caution, navigation and
   listing are Safe). Unrecognized commands default to Caution rather than
   Safe or Blocked, so the terminal stays usable for commands outside the
   initial rule set while still requiring a look before they run.
2. Classification applies to every direct command typed into the command
   bar, not only AI-generated ones — this was an explicit product decision
   (not just an implementation detail) to keep `Remove-Item -Recurse -Force
C:\`-style commands risk-classified even when the user types them
   directly, matching the safety contract instead of behaving like a bare
   passthrough terminal.
3. Added a Rust `execute_command` command that re-classifies before running
   and refuses Blocked commands unconditionally at the execution boundary,
   regardless of what the caller believes was already decided — the one
   tier that cannot be bypassed by a compromised or buggy UI layer.
4. Wired the command bar end-to-end: Safe commands run immediately; Caution
   and High Risk commands show an inline approval card (risk badge, reason,
   the exact command, Approve/Cancel) before running; Blocked commands are
   refused with the reason shown and never offered an approve path.
5. This is direct command entry only (you type the literal PowerShell
   command) — plain-English requests still are not translated or executed;
   that is the sidecar-wiring work still ahead.

Validation:

- `cargo test` (src-tauri): 17 passed, including 11 classifier tests
  covering every documented Safe/Caution/High Risk/Blocked example plus
  edge cases (root-vs-scoped deletion, redirected reads, unrecognized
  commands, empty input).
- `npm run build`: passed.
- Manually verified live in the desktop app: a session started, its
  `Get-Location` startup output streamed correctly, and a directly typed
  command (`dir`) ran and streamed its output through the same command bar.

Next:

1. Add the trusted adapter that renders the already NLP-matched Safe
   intents (change directory, list/find files by extension, view a line
   range of a file) into the same classify/execute path.
2. Connect the plain-English command bar to the sidecar intent endpoint,
   so a typed request first tries NLP matching and falls back to direct
   command classification.
3. Add approval receipts (what ran, when, its result) and cancellation.
4. Add basic recovery guidance for failed commands.

## Phase 1 slice - Session activation and layout polish - 26/07/2026

Completed, based on live manual testing feedback:

1. Selecting or adding a workspace now starts its terminal session
   automatically — no separate manual step. A workspace can still only have
   one active session at a time, and switching between workspaces just
   switches which one's already-running session is shown.
2. Removed the separate "+" new-session button, since auto-start made it
   redundant in normal use (it was only ever enabled in the instant after an
   explicit close). The terminal tab itself is now the affordance: clicking
   it restarts a session that was explicitly closed via its "x".
3. Tightened the terminal output panel's layout (explicit full width,
   reduced padding) so streamed command output uses the available panel
   width instead of leaving an unused margin.

Validation:

- `npm run build`: passed.
- Confirmed by the user directly in the running desktop app.

## Phase 1 slice - Policy classifier audit against real command references - 26/07/2026

The user supplied four community reference sheets covering common and DevOps
terminal commands for both Bash and PowerShell (Docker, Docker Compose,
Kubernetes, Helm, Terraform, Ansible, SSH, GitHub CLI, AWS/Azure/gcloud CLIs).
Ran every PowerShell-relevant command from those sheets through
`classify_command` via a temporary audit test to check real-world coverage
rather than only the small set of examples in `SAFETY_MODEL.md`.

Found and fixed one real bug and a large coverage gap:

1. **Bug:** `docker run --rm ...` was misclassified High Risk — the delete
   rule's `rm` alias matched inside Docker's extremely common `--rm` flag.
   The `regex` crate has no lookbehind, so `rm` as a delete command is now
   only matched at the start of input or after `;`/`&`/`|`, never as part of
   a `-`-prefixed flag. Fixed identically in both the general delete rule
   and the broad-root-delete Blocked check.
2. **Gap:** Kubernetes, Helm, Terraform, Docker Compose, `taskkill`,
   `Set-ExecutionPolicy`, GitHub CLI, and basic cloud CLI commands had no
   dedicated rules at all, so `terraform destroy` and `gh pr list` received
   the identical generic "unrecognized" Caution — no differentiation between
   trivial and catastrophic actions in those tool families. Added targeted
   rules so read-only operations (`kubectl get`, `terraform plan`, `helm
list`, `gh pr list`, `docker compose ps`, cloud account/version checks)
   are Safe, ordinary mutations (`kubectl apply`, `helm install`, `docker
compose up`, `gh pr create`, cloud login/configure) are Caution, and
   irreversible or infrastructure-destroying actions (`terraform apply` /
   `destroy`, `kubectl delete`, `helm uninstall`, `docker compose down -v`,
   `taskkill /F`, `Set-ExecutionPolicy ... Bypass`) are High Risk. Also added
   `kubectl get secret(s)` as a specific Caution override ahead of the
   general `kubectl get` Safe rule, matching `SAFETY_MODEL.md`'s point that
   risk depends on content, not just the command family. Full tool coverage
   for these DevOps commands is still Phase 3/5 work per `ROADMAP.md` — this
   change only improves how today's classifier scores that vocabulary if
   someone happens to type it.

Validation:

- `cargo test` (src-tauri): 24 passed, including a regression test for the
  `--rm` fix and one test per newly covered tool family.
- `npm run build`: passed.

## Phase 1 slice - Re-audit focused on developer-common commands - 26/07/2026

The DevOps reference sheets covered infrastructure tooling (Docker, K8s,
Terraform, Helm, cloud CLIs) that is explicitly Phase 3/5 scope, not today's
priority. The user clarified the actual near-term focus is Windows-native
developer workflows, and asked for a re-check specifically against
`common-terminal-commands-windows-powershell.md` (navigation, files, Git,
Node/npm, Python, Rust/Cargo, processes, networking) — the vocabulary a
working developer actually types day to day.

Ran every command from that sheet through the classifier again. No command
came back mis-classified in the unsafe direction, but a batch of everyday,
harmless commands fell to generic "unrecognized" Caution and needed explicit
rules:

1. Added Safe rules for developer tool version checks (`git`, `node`, `npm`,
   `python`, `rustc`, `cargo` with `--version`/`-v`), `git rev-parse`,
   `cargo check` (distinguished from `cargo build`/`test`, which compile and
   run code and stay Caution), `deactivate`, and `npm cache verify`.
2. Added `git add` to the existing Git-write Caution rule (previously
   unmatched).
3. Added explicit Caution reasons for `cargo build`/`test`/`run` and
   `python -m venv`/`pytest`, replacing the generic "unrecognized" message
   with an accurate one, even though the risk level was already correct.

Validation:

- `cargo test` (src-tauri): 29 passed, including five new tests covering
  version checks, `git add`/`rev-parse`, `cargo check` vs. `build`/`test`,
  the Python venv lifecycle, and `npm cache verify`.

## Phase 1 slice - Long-running commands: log files, live tail, Stop - 26/07/2026

The user tried using the app to run terminal-mate's own dev commands
(`npm run tauri:dev`, `npm run installer:windows`) and hit the obvious limit:
`execute_command` waited synchronously for a completion sentinel with a
30-second timeout, so it could not handle commands that either never exit on
their own (dev servers) or simply take longer than 30 seconds (the installer
build). Agreed approach with the user: every command — not just guessed-long
ones — now goes through one unified execution path with a log file, live
output, and a completion banner; short commands just finish almost
instantly and barely show it.

Completed:

1. Added `commands/command_runs.rs` (`RunManager`) — a `run_command` Tauri
   command that spawns each command as its own dedicated one-shot
   `powershell.exe` process (separate from the persistent interactive
   session, which is only used for its working directory), returns
   immediately with a run ID and log file path, and streams output the same
   way session commands already do.
2. Every run's full stdout/stderr is written to a per-run log file under the
   app's log directory (`app_log_dir()/command-runs/<sessionId>/<runId>.log`)
   as well as streamed live — so the log is always there to review
   afterward, whether the command took 10ms or 10 minutes.
3. Added a `stop_command` Tauri command. Real Ctrl+C is not reliably
   possible cross-process on Windows, so Stop force-kills the entire process
   tree via `taskkill /T /F /PID`, matching the app's existing "trusted
   application code enforces the boundary" model rather than trusting the
   child process to exit cleanly.
4. Added `open_log_file`, which opens the log with its default (e.g.
   Notepad) via `explorer.exe`.
5. Only one foreground command can run per session at a time — a second
   `run_command` call while one is active is refused, same idea as not being
   able to type into a terminal while something is running in it.
6. CommandBar now shows a live "Running... [Stop]" bar while a command is
   active, and a "Succeeded / Failed (exit N) / Stopped" bar with an
   "Open log" button once it finishes. The old `execute_command` path is
   removed — this is now the only way commands run, per the "every command,
   always" decision.
7. Classification/approval is unchanged: Blocked commands are still refused
   at the execution boundary (re-checked inside `run_command`, not just
   trusted from the caller), Caution/High Risk still require the approval
   card first.

A real concurrency bug was caught by the tests, not eyeballing: the waiter
thread that detects a command's exit was calling the blocking `Child::wait()`
while still holding the shared session-run map's mutex lock, so `stop_command`
and the "already running" guard could not acquire that lock until the command
had already exited on its own — meaning Stop silently did nothing for the
entire lifetime of whatever it was supposed to stop. Fixed by polling
`try_wait()` and only holding the lock briefly on each poll, not for the
entire run.

Validation:

- `cargo test` (src-tauri): 33 passed, including 4 new integration tests that
  spawn real processes — a quick command that succeeds, a Blocked command
  refused before it ever starts, a 120-second command actually killed by
  Stop within a moment (not after running to completion), and two
  overlapping runs in the same session correctly rejected.
- `npm run build`: passed.
- Not yet manually verified against the real target case (`npm run
tauri:dev` / `npm run installer:windows` through the app itself) in this
  session — recommend trying both once you're back in the running app.

## Phase 1 slice - Fixed misleading stderr coloring - 26/07/2026

The user ran `npm run installer:windows` through the app itself (the real
target case the previous slice's log-file/Stop support was built for) and it
worked end-to-end, but every line of output rendered red, making a fully
successful build look like a wall of errors.

1. **Bug:** `.terminal-line-stderr` was colored with `var(--red)` regardless
   of outcome. Cargo, npm, and Vite all write routine build/progress output
   to stderr as a matter of course, not only on failure, so coloring by
   stream origin rather than by actual result was misleading by design, not
   just in this one case.
2. **Fix:** stderr lines now render in `var(--text-muted)`, matching stdout.
   The authoritative success/fail signal remains the exit-code-based
   completion banner (Succeeded/Failed + Open log), not which stream a line
   came from.

Validation:

- `npm run build`: passed.
- Manually confirmed by the user: `npm run installer:windows` run through
  the app against a representative workspace completed successfully
  (`Finished release profile`, installer `.exe` produced) — this is the
  first real manual verification of the long-running command path against
  its actual target use case.

Next:

1. Connect the plain-English command bar to the sidecar's NLP intent
   endpoint (still entirely unwired — the sidecar builds and has its own
   tests, but Rust never spawns it and the command bar never calls it), so a
   typed request first tries intent matching and falls back to direct
   command classification.
2. Add the trusted adapter that renders the sidecar's already-matched Safe
   intents (change directory, list/find files by extension, view a line
   range of a file) through the same classify/execute path direct commands
   already use.
3. Add approval/run history (what ran, when, its result) beyond the current
   single most-recent-run-per-session state.

## Phase 1 slice - Sidecar wiring: plain-English intents to PowerShell - 26/07/2026

Closed out the two "Next" items above in one slice, since the adapter (item 2) has nothing to render until the sidecar is actually reachable (item 1).

Completed:

1. Added `app_state.rs` / `sidecar_proxy.rs` / `sidecar.rs`: Rust spawns
   `terminal-mate-sidecar` as
   a Tauri shell-plugin sidecar on app startup (`tauri_plugin_shell`), reads
   a one-line `TERMINAL_MATE_READY:{"port":N,"protocol_version":"1"}`
   readiness message off its stdout, then polls its authenticated
   `/v1/health` endpoint before marking it ready. A random per-launch
   session token is generated in Rust, passed to the child process only via
   an env var, and never exposed to the webview — every sidecar request is
   sent with it as a bearer token. On app exit, the sidecar process tree is
   force-stopped the same way long-running commands already are
   (`taskkill /T /F` on Windows).
2. Added `commands/intent_adapter.rs`: a pure function that renders each of
   the sidecar's three matched Phase 1 intents into the literal PowerShell
   command that will actually run — `show_current_directory` →
   `Get-Location`, `change_directory` → `Set-Location` (with `$HOME` for
   "home", single-quote-escaped `-LiteralPath` otherwise), `find_files` →
   `Get-ChildItem -Filter ... -File` (+ `-Recurse` when asked), and
   `view_file_lines` → `Get-Content | Select-Object -Skip -First`, rejecting
   an inverted or non-positive line range before it ever reaches a shell.
3. Added the `resolve_command` Tauri command: given what the user typed, it
   asks the sidecar to interpret it, and only when the sidecar is reachable,
   ready, and returns a matched intent this adapter knows how to render does
   it substitute the rendered PowerShell command; every other case (sidecar
   still starting, unreachable, no match, or an intent action this build
   doesn't render yet) falls straight back to treating the typed text as a
   literal command — the exact behavior that already existed. Either way,
   the result still goes through the same `classify_command_text` →
   approval → `run_command` boundary as before; intent matching only decides
   what runs, never whether it is allowed to.
4. Command bar now shows a small "Interpreted ... as a command" note when a
   plain-English request was matched, so the user sees the literal command
   before it is classified or approved, not just after.

Added five more tests (`commands/intents.rs`) that spawn the actual
PyInstaller-built `terminal-mate-sidecar` binary directly (the same one
Tauri's shell plugin launches in production) and drive it through the real
HTTP + bearer-token wire protocol — not a mock. These caught what a GUI
click-through alone would not have proven: "where am i", "show me all the
.md files", and "show me lines 20-100 of sample.txt" each round-trip through
the real Python matcher and come back rendered as the exact PowerShell
command (`Get-Location`; `Get-ChildItem -LiteralPath '.' -Filter '*.md'
-File -Recurse`; `Get-Content -LiteralPath 'sample.txt' | Select-Object
-Skip 19 -First 81`), and an unrecognized request or a never-configured
sidecar both correctly fall back to `None` (direct command handling).

Validation:

- `cargo test` (src-tauri): 49 passed, including 11 intent-adapter tests and
  5 real-sidecar integration tests (3 successful-match cases across all
  three Phase 1 intents, 2 fallback cases).
- `npm run build`: passed.
- Confirmed no orphaned sidecar processes after the test run.
- Still not manually clicked through in the running desktop app in this
  session (the GUI itself, as opposed to the sidecar wiring underneath it,
  is unverified) — recommend a quick `npm run tauri:dev` pass to confirm the
  "Interpreted ... as a command" note actually renders correctly in the
  command bar.

Next:

1. Manually click through the running app to confirm the frontend
   experience (the interpreted-note UI, not just the backend wiring, which
   is now verified end-to-end against the real sidecar binary).
2. Expand the sidecar's local matcher beyond the three Phase 1 intents
   (currently the near-term ceiling of what `intent_adapter.rs` renders).
3. Add approval/run history (what ran, when, its result) beyond the current
   single most-recent-run-per-session state.

## Phase 1 slice - Fixed the sidecar never actually going ready - 26/07/2026

The user ran the real app (`npm run tauri:dev`) and typed "show me all the
.md files" — it never resolved through the sidecar. It was classified
CAUTION as an "unrecognized command" and ran literally, failing in
PowerShell (`show` is not a cmdlet). The intent path was silently never
engaging at all.

Root cause: `sidecar.rs`'s `HealthResponse` struct included
`#[serde(rename_all =
"camelCase")]` and a comment claiming "FastAPI serialises response fields as
camelCase JSON." That assumption applies to schemas using a `CamelModel` base
class with `alias_generator=to_camel`. It is not true
for terminal-mate: `sidecar/app/schemas/system.py` and `schemas/intents.py`
are plain `pydantic.BaseModel`s with no aliasing, so the real JSON is
snake_case (`protocol_version`, not `protocolVersion`). Every one of the 20
authenticated health checks `launch()` sends during startup silently failed
to deserialize, the sidecar was marked `"failed"` once the retry budget was
exhausted, and every `resolve_command` call fell back to direct handling
exactly as designed for an unreachable sidecar — just with no user-visible
error, because that fallback is deliberately silent.

The previous slice's five real-sidecar tests didn't catch this because they
manually called `configure_sidecar` + `ready()` on a test `AppState` instead
of going through `launch()`, so they exercised `try_interpret` correctly but
never exercised the health-check step where the bug actually lived.

Completed:

1. Removed the incorrect `rename_all = "camelCase"` from `HealthResponse`
   and the comment that justified it; both were true of a sibling project's
   sidecar schemas, not this one's.
2. Added `sidecar::tests::health_response_deserializes_from_the_real_sidecar`
   — spawns the real sidecar binary and calls its actual `/v1/health`
   endpoint the same way `launch()`'s readiness loop does, deserializing the
   response with the real struct. This is the test that should have existed
   before the previous slice was called verified; it fails against the old
   camelCase struct and passes against the fix.

Validation:

- `cargo test` (src-tauri): 50 passed (the 49 from the previous slice plus
  this one new health-check test).
- `npm run build`: passed.
- Manually confirmed by the user in the running app: the sidecar now
  reaches ready and a plain-English request resolves and runs through it
  correctly — the first real end-to-end confirmation of the intent path,
  frontend included.

## Phase 1 slice - Expanded the local intent matcher - 26/07/2026

Grew coverage beyond the original three Phase 1 intents, staying inside the
same developer-common Windows navigation/file scope rather than reaching
into DevOps territory (still Phase 3/5, per `ROADMAP.md`).

Added four new intents to `sidecar/app/intent/local_matcher.py`, each with a
matching Rust render branch in `commands/intent_adapter.rs`:

1. **Go up a directory** ("go up one directory" / "go back") — folds into
   the existing `change_directory` action with `target: ".."`, so no new
   adapter branch was needed for it specifically; `Set-Location
-LiteralPath '..'` already worked correctly through the existing
   non-"home" branch.
2. **List directory, no extension filter** ("list the files here" / "what's
   in this folder?") — new `list_directory` action → `Get-ChildItem
-LiteralPath '<root>'`. Complements the existing extension-filtered
   `find_files` action.
3. **Make a directory** ("create a folder called drafts") — new
   `make_directory` action → `New-Item -ItemType Directory -Path '<path>'`.
4. **Search file contents** ("search for TODO in this folder" / "find the
   text TODO in files") — new `search_file_contents` action →
   `Get-ChildItem -LiteralPath '<root>' -Recurse -File | Select-String
-Pattern '<term>'`. This is the other half of Phase 1's original
   "grep-like" scope alongside `view_file_lines` — search vs. read a
   specific range.

All four render to commands the existing policy classifier already has
dedicated rules for (`Get-ChildItem`/`Select-String` → Safe, `Set-Location`
→ Safe, `New-Item` → Caution), so no classifier changes were needed — they
picked up sensible risk levels automatically.

Validation:

- `npm run sidecar:test`: 16 passed (8 new Python-level matcher tests).
- `npm run sidecar:build`: rebuilt the bundled binary so both the app and
  the Rust integration tests exercise the new Python logic, not a stale
  build — a real risk after the previous slice's lesson about testing
  against the actual binary.
- `cargo test` (src-tauri): 59 passed — 5 new `intent_adapter` unit tests,
  4 new real-sidecar integration tests (one per new intent, following the
  same real-binary pattern established after the health-check bug), plus
  everything from before, unaffected.
- `npm run build`: passed.
- Not yet manually verified in the running app in this session.

## Phase 1 slice - Persistent command working directory - 31/07/2026

Corrected an important terminal-semantics gap: foreground commands were
launched as separate PowerShell processes, so a successful `Set-Location`
changed only that temporary process and the following command started back in
the workspace root.

Completed:

1. Captured each command process's final PowerShell working directory in a
   private run metadata file.
2. Updated the shared session state when a run completes, so the next command
   starts from the directory established by the previous command.
3. Updated the command prompt and Environment context panel to show the live
   directory.
4. Made `SessionManager` safely cloneable for completion workers and limited
   session cleanup to the final manager owner.
5. Added a focused two-command regression test proving that a directory change
   persists into the next command.

Validation:

- Focused Rust working-directory regression test: passed.
- `cargo check`: passed.
- `npm run build`: passed.
- `npm run sidecar:test`: 16 passed, with one upstream deprecation warning.
- The full Rust test target was not used as a release gate for this slice
  because it stalled in this Windows environment; that runner issue remains to
  be isolated.

## Phase 1 slice - Session approval and run history - 31/07/2026

Added a compact Recent runs section to the Environment context panel so a user
can see what TerminalMate executed without leaving the active workspace.

Completed:

1. Retained the ten newest commands for each open terminal session.
2. Recorded the policy risk level and whether execution was automatic or
   explicitly approved.
3. Updated running entries with success, failure, stop state, exit code, and
   completion time.
4. Added direct access to each completed command's existing log file.
5. Kept history isolated per session and cleared it when that terminal session
   closes.

Validation:

- `npm run build`: passed.
- Manual desktop verification remains pending.

Next:

1. Manually verify working-directory persistence and Recent runs in the
   installed desktop app.
2. Show a live elapsed time for every running command and retain the final
   duration in the completed run and Recent runs history.
3. Persist command history across app restarts after the session-scoped UX is
   confirmed.
4. Add broader diagnostics and guided recovery.

## Phase 1 slice - Common developer command intents - 31/07/2026

Expanded the plain-English path around common local development work while
keeping Git, SSH, cloud, container, and infrastructure operations outside this
slice.

Completed:

1. Added full-file reading and bounded first/last-line previews, with previews
   limited to 1-500 lines.
2. Added recursive filename search, folder-only listing, file metadata
   inspection, and file counting.
3. Added read-only running-process, listening-port, and specific-port
   inspection.
4. Rendered every new action into deterministic PowerShell and kept it behind
   the same policy classification and execution boundary as direct commands.
5. Added focused matcher and adapter tests, including invalid preview-size and
   invalid-port rejection.
6. Documented the current plain-English command examples in `README.md`.
7. Added a parent-process guard to the packaged sidecar after validation found
   orphaned sidecars left by interrupted Rust test/dev processes. Production
   and real-sidecar tests now identify the owning Rust PID so the Python
   process exits if that owner disappears.

Scope decisions:

- DevOps discovery remains scheduled before Phase 3 implementation, including
  collecting real commands and workflows from Allan's DevOps contact.
- SSH remains Phase 4.
- Git-specific guided workflows are intentionally deferred because AI Git
  Assistant already covers that product area.

Validation:

- `npm run sidecar:test`: 27 passed, with one upstream Starlette deprecation
  warning.
- Focused Rust intent-adapter tests: 27 passed.
- `cargo check`: passed.
- `npm run build`: passed.
- `npm run sidecar:build`: passed and refreshed the bundled Windows sidecar.

Next:

1. Verify the new phrases in the desktop app.
2. After the current unit-testing pass, add a live command elapsed timer and
   retain the final duration in run history.
3. Persist command history across app restarts.
4. Add broader diagnostics and guided recovery.

## Phase 1 slice - Command feedback and Recent runs polish - 31/07/2026

Followed up on manual command testing that exposed three failed natural-language
requests and several command-bar usability gaps.

Completed:

1. Fixed `list folders only` so it resolves to the folder-only PowerShell
   listing instead of running as an invalid literal command.
2. Added `Preview first/last lines of <file>` as a combined, bounded file
   preview. Requests that omit the filename now ask for it instead of creating
   a failed run.
3. Added direct `Inspect port <number>` phrasing. Port inspection without a
   number now asks which port to inspect instead of creating a failed run.
4. Increased Recent runs from 8 to 10 entries and retained the user's original
   natural-language request as the primary label, with the resolved PowerShell
   command available beneath it.
5. Restored keyboard focus to the command input after completion, approval,
   cancellation, blocked requests, and clarification prompts.
6. Added an explicit `No output returned.` terminal message for successful
   commands that produce no stdout or stderr. Numeric zero remains visible as
   the command's normal output.

Validation:

- `npm run sidecar:test`: 32 passed, with one upstream Starlette deprecation
  warning.
- `npm run build`: passed.
- `cargo check`: passed.
- Focused Rust intent-adapter tests: 28 passed.
- Focused Rust empty-output command test: passed.
- Focused real-sidecar clarification test: passed.
- `npm run sidecar:build`: passed and refreshed the bundled Windows sidecar.

Next:

1. Manually verify these interactions in the desktop app.
2. Add a live elapsed timer and retain final command duration in Recent runs.
3. Persist command history across app restarts.
4. Add broader diagnostics and guided recovery.

## Phase 1 slice - Reusable Recent runs and command timing - 31/07/2026

Followed up on desktop testing of the expanded developer-command intents and
completed the timing work planned in the previous slices.

Completed:

1. Added matcher coverage for the exact variants `list folder only`,
   `Preview first/last lines .env`, and `Inspect a port 8000`.
2. Made every Recent runs entry reusable. Selecting an entry restores its
   original natural-language request to the command line and returns keyboard
   focus there.
3. Added backend start, finish, and duration timestamps to each command run so
   fast and long-running commands use the same timing source.
4. Added a live elapsed timer while a command is running and retained the final
   duration in the completion strip and Recent runs.
5. Hardened Windows Stop behavior with a direct child-process fallback when
   `taskkill` cannot terminate the process tree.

Validation:

- `npm run sidecar:test`: 35 passed, with one upstream Starlette deprecation
  warning.
- `npm run build`: passed.
- `cargo check`: passed using an isolated target while the development app held
  the normal Tauri target directory.
- Rust timing and Windows Stop command-run tests: passed.
- Prettier check for the changed frontend files: passed.

Next:

1. Manually verify click-to-reuse and live/final timing in the desktop app.
2. Persist command history across app restarts.
3. Add broader diagnostics and guided recovery.

## Phase 1 slice - Durable workspace history - 31/07/2026

Completed the persistence follow-up after desktop testing confirmed Recent runs
were cleared whenever TerminalMate restarted.

Completed:

1. Persisted the latest 10 command runs in local application storage.
2. Keyed saved history by workspace instead of temporary terminal session ID,
   so a newly created session restores the correct project's history.
3. Preserved the original natural-language request, resolved command, approval,
   risk, result, exit code, duration, and log path.
4. Kept click-to-reuse behavior available for restored history entries.
5. Marked commands that were still running when the app closed as
   `Interrupted` on the next launch.
6. Added validation and failure isolation for saved history so malformed or
   unavailable local storage cannot block command execution.
7. Cleared a workspace's saved history only when that workspace is removed
   from TerminalMate.

Validation:

- `npm run build`: passed.

Next:

1. Manually verify history restoration by running commands, closing the app,
   and reopening the same workspace.
2. Add broader diagnostics and guided recovery.
3. Complete the Phase 1 regression pass and Windows installer validation.

## Phase 1 slice - Accurate PowerShell outcomes - 31/07/2026

Desktop testing found a command that printed a PowerShell error but appeared as
Succeeded with exit code 0. The same request also exposed an ambiguous local
intent match.

Completed:

1. Updated the PowerShell execution wrapper so cmdlet errors become terminating
   errors and produce a nonzero process exit code.
2. Preserved normal success behavior for valid commands, successful commands
   with no output, directory changes, and explicitly stopped runs.
3. Fixed `show all .py` so it resolves to a recursive `*.py` file listing
   instead of trying to read a file literally named `all .py`.
4. Added source-level matcher coverage, a real packaged-sidecar request test,
   and a Rust command-run regression test for missing-file errors.

Validation:

- `npm run sidecar:test`: 36 passed, with one upstream Starlette deprecation
  warning.
- Full Rust library suite: 75 passed.
- Real packaged-sidecar shorthand extension request: passed.
- `npm run sidecar:build`: passed and refreshed the bundled Windows sidecar.

Next:

1. Manually verify `show all .py` displays matching files and records a
   Succeeded result.
2. Manually verify a genuinely missing file records Failed with a nonzero exit
   code.
3. Continue with broader diagnostics and guided recovery.

## Phase 1 slice - Responsive workspace switching - 31/07/2026

Desktop testing found that switching to a workspace with a large persisted
terminal transcript could stall while React recreated every historical output
line.

Completed:

1. Batched streamed terminal events into 50 ms UI updates instead of updating
   React state once for every output line.
2. Limited each open terminal's visible in-memory transcript to its latest
   1,000 lines.
3. Preserved complete foreground-command output in the existing per-run log
   files.
4. Added a visible notice when older terminal lines are hidden, including the
   hidden-line count and guidance to use Open log.
5. Memoized the terminal workspace so output arriving in an inactive workspace
   does not rebuild the active terminal transcript.
6. Preserved stop-and-close behavior while a command is running.

Validation:

- `npm run build`: passed.
- Actual buffer helper verified with 5,001 lines: 1,000 visible, 4,001 hidden,
  and the correct newest line range retained.

Next:

1. Manually run a high-output command and switch repeatedly between workspaces.
2. Confirm the terminal stays responsive and Open log retains the full command
   output.
3. Continue with broader diagnostics and guided recovery.

Manual validation completed:

- Switching between workspaces after a 25,000-line command remained responsive.
- Reopening the workspace restored only the newest 1,000 visible lines as
  intended.

## Phase 1 slice - Initial diagnostics and guided recovery - 31/07/2026

Added a read-only recovery layer for failed commands so a nonzero exit provides
more than a red status and raw shell output.

Completed:

1. Added deterministic diagnosis for missing files or folders, unavailable
   commands, permission failures, network resolution/connectivity failures, and
   unknown nonzero exits.
2. Restricted diagnosis to the final 64 KiB of TerminalMate-owned command logs,
   avoiding full reads of large build logs and rejecting paths outside the app's
   command-log directory.
3. Added a compact recovery panel with a plain-English cause and ordered next
   steps immediately after a failed run.
4. Added a safe reusable suggestion for missing-path failures: `List files in
this folder`. Selecting it returns the request to the focused command input;
   it is still classified normally before execution.
5. Kept permission, networking, unavailable-tool, and generic failures
   advisory-only because TerminalMate cannot safely infer a universal repair
   command.
6. Added Rust regression coverage for missing-path, unavailable-command, and
   generic nonzero-exit diagnoses.

Validation:

- `npm run build`: passed.
- Full Rust library suite: 78 passed.
- `cargo fmt --check`: unavailable because `rustfmt` is not installed in the
  active Rust toolchain.

Next:

1. Manually verify the recovery panel using a missing file and an unavailable
   command in the desktop app.
2. Complete the Phase 1 sidecar regression and Windows installer validation.
3. Close Phase 1 and begin Phase 2 WSL/Linux subsystem support after the
   regression checklist passes.

## Phase 1 slice - Named subfolder listing recovery - 31/07/2026

Fixed a matcher gap discovered during desktop testing. Requests such as
`List files .claude in this folder` previously fell through as literal
PowerShell, causing PowerShell to look for a command named `List`.

Completed:

1. Added deterministic plain-English matching for inspecting a named path in
   the current workspace.
2. Preserved the existing current-folder listing behavior while mapping the
   named target to the structured `inspect_path` intent.
3. Added runtime path validation. The result labels an existing target as
   `Folder` or `File`, lists children only for folders, and displays metadata
   for files.
4. Added sidecar matcher, Rust adapter, and packaged-sidecar integration
   regressions for the exact phrases reported during desktop testing.
5. Narrowed the blocked disk-format rule after desktop testing found that it
   incorrectly treated PowerShell's read-only `Format-List` cmdlet as the
   destructive `format` command.

Validation:

- `npm run sidecar:test`: 38 passed.
- `npm run sidecar:build`: passed and refreshed the Windows sidecar binary.
- Full Rust library suite: 81 passed.

Next:

1. Restart the desktop development app and manually verify both documented
   `.claude` listing phrases.
2. Run the frontend production build and Windows installer validation.
3. Close Phase 1 after the regression checklist passes.

## Phase 1 slice - Saved workspace runtime migration - 01/08/2026

- Migrated saved Windows-path workspaces from the retired WSL placeholder
  profile to the actual Windows / PowerShell executor.
- Added a session boundary check so an unsupported runtime profile can no
  longer be executed by a different shell than the UI reports.

## Phase 2 - Per-workspace Windows / WSL runtime - 01/08/2026

Completed:

1. Added an Environment Context selector that switches the active workspace
   between native Windows PowerShell and WSL Bash.
2. Persisted the runtime choice on each workspace, allowing different open
   workspaces to use different shells at the same time.
3. Added real WSL session and foreground-command execution through `wsl.exe`,
   including Windows-to-WSL working-directory mapping.
4. Added Bash rendering for the existing structured navigation, file,
   directory, search, process, and port-inspection intents.
5. Extended the policy classifier for Bash read and mutation commands while
   keeping the same Safe, Caution, High Risk, and Blocked boundary.
6. Migrated stale placeholder WSL profiles safely and validated WSL before a
   workspace can switch modes.

Validation:

- `npm run build`: passed.
- `npm run sidecar:test`: 38 passed.
- `cargo check`: passed.
- `cargo test --lib`: 88 passed.
- Real WSL2 Bash path and read-only file discovery: passed.
- `npm run installer:windows`: passed.
- Installer: `TerminalMate_0.2.0_x64-setup.exe` (18.84 MB).
- Installer SHA-256:
  `C5EFAF693CCA70E092259AABA74C42676224CB4AB4E868B8619225858016F5A6`.

Next:

1. Implement the prioritized deterministic Azure CLI and Terraform intent
   specification with typed parameters, prerequisite checks, and explicit
   safety levels.
2. Keep `terraform apply`, `terraform destroy`, storage-key retrieval, and
   resource-group deletion explain-only until their stronger approval flows
   are designed and tested.
3. Add environment-specific recovery guidance and golden PowerShell/Bash
   adapter tests as the intent catalogue expands.

## Phase 2 slice - Azure CLI and Terraform intents - 01/08/2026

Completed:

1. Added deterministic Azure CLI and Terraform matching for account,
   subscription, SSH key, resource-group, storage, VM, network, state,
   validation, planning, output, and import requests.
2. Extracted required names, locations, subscriptions, resource groups, and
   Terraform addresses as typed parameters; incomplete requests return a
   clarification instead of generating a guessed shell command.
3. Rendered supported intents independently for native PowerShell and WSL
   Bash while preserving the selected runtime of each workspace.
4. Added an Azure account prerequisite check and deterministic sign-in
   recovery guidance for operations that require an authenticated session.
5. Added an Explain Only result for Terraform apply/destroy, Azure storage-key
   retrieval, and resource-group deletion. These cards expose an exact command
   for manual use but cannot execute it from TerminalMate.
6. Added typed `RUN` confirmation for executable High Risk commands and kept
   Safe and Caution operations behind their proportional policy controls.
7. Added sidecar matcher tests, shell-rendering tests, policy tests, and Azure
   recovery tests.

Validation:

- `npm run build`: passed.
- `npm run sidecar:test`: 56 passed.
- `cargo check`: passed.
- `cargo test --lib`: 93 passed.
- `npm run sidecar:build`: passed.
- `npm run installer:windows`: passed.
- Installer: `TerminalMate_0.2.1_x64-setup.exe` (18.85 MB).
- Installer SHA-256:
  `BC18D6C91CEC7D0B0967DB91E793414ADAFE9E9FA73DF419EB8D89A33CD6D012`.

Next:

1. Validate representative Azure and Terraform requests manually in both
   PowerShell and WSL workspaces.
2. Expand the PowerShell/Bash golden matrix and capability diagnostics.

## Phase 2 slice - In-app command reference - 01/08/2026

Completed:

1. Connected the top-bar Help button to a searchable in-app command reference.
2. Grouped supported requests into Everyday, Azure, Terraform, and SSH tabs.
3. Added Safe, Approval, High Risk, and Explain Only labels so examples do not
   overstate what TerminalMate can execute.
4. Added one-click command reuse that closes Help and returns the selected
   plain-English request to the active workspace command line with focus.
5. Added responsive scrolling and keyboard/backdrop dismissal for the Help
   dialog.
6. Added command-line Help routing: `Help` opens the full catalogue, while
   `Help Azure`, `Help Terraform`, and `Help SSH` open focused references.
7. Added an explicit unsupported-topic state for requests such as `Help AWS`,
   preventing unsupported help phrases from reaching command execution.

Verification:

- `npm run build`: passed.
- `npm run sidecar:test`: 56 passed, with one existing Starlette deprecation
  warning.

Next:

1. Manually verify the top-bar Help button and the `Help`, `Help Azure`,
   `Help Terraform`, `Help SSH`, and `Help AWS` command-line routes.
2. Complete the Phase 2 manual regression before beginning Phase 3 DevOps
   discovery.

## Phase 2 slice - Cross-shell infrastructure regression - 01/08/2026

Completed:

1. Added a table-driven Azure and Terraform golden matrix for native
   PowerShell and WSL Bash rendering.
2. Locked shell-specific quoting for names containing spaces and apostrophes.
3. Verified exact Azure authentication preflight commands before supported
   resource-group, storage-account, and VM operations.
4. Expanded sidecar intent coverage to every Azure and Terraform request
   currently advertised in Help, including storage containers, storage keys,
   Terraform lifecycle/state/import, Azure resources, NSGs, and VM lifecycle.
5. Kept Explain Only commands in the rendering contract without weakening
   their execution boundary.

Validation:

- `npm run build`: passed.
- `npm run sidecar:test`: 70 passed, with one existing Starlette deprecation
  warning.
- `cargo test --lib intent_adapter`: 37 passed.
- `cargo test --lib`: 94 passed.
- `cargo fmt --check`: not run because `rustfmt` is not installed in the
  current Windows Rust toolchain.

Next:

1. Execute a short manual Azure/Terraform smoke matrix in one PowerShell and
   one WSL workspace, including a missing-login recovery case.
2. Begin Phase 3 DevOps command discovery after the manual cross-shell gate
   passes.

## Phase 2 slice - Exact command input - 01/08/2026

Completed:

1. Retained direct exact-command input alongside deterministic plain-English
   intent matching.
2. Routed exact commands through the same shell context, policy decision,
   approval, execution, logging, and failure-recovery path.
3. Added multiline paste normalization for Bash backslash and PowerShell
   backtick continuations.
4. Rejected unrelated commands separated by bare newlines so each command
   receives its own safety review.
5. Verified that exact Azure resource-group creation remains High Risk and
   requires typed `RUN` approval before execution.
6. Documented exact-command entry in the in-app Help and README.

Validation:

- `npm run build`: passed.
- `cargo test --lib`: 96 passed.
- Focused direct-command normalization and multiline-rejection tests: passed.
- Focused Azure policy classification test: passed.

Next:

1. Paste one continued Azure command into PowerShell and WSL workspaces and
   verify its normalized preview and typed `RUN` gate.
2. Confirm that two unrelated pasted commands are rejected with one-command
   guidance.

## Phase 2 slice - Azure subscription recovery - 02/08/2026

Completed:

1. Recognized Azure CLI `SubscriptionNotFound` failures instead of presenting
   generic nonzero-exit guidance.
2. Explained that the selected subscription is unavailable to the active
   account or tenant.
3. Added `List Azure subscriptions` as the safe recovery action, followed by
   guidance to select an accessible subscription or sign into the correct
   tenant before retrying the original command.
4. Added a focused regression test using the real Azure error shape returned
   by an exact `az storage account create` command.
5. Replaced the one-line suggestion with a guided recovery workflow that runs
   a safe structured subscription check and presents accessible subscriptions
   in the app.
6. Preserved the normal safety boundary: switching subscriptions and Azure
   device-code sign-in require approval, and retrying the original
   infrastructure command requires a fresh typed `RUN` confirmation.
7. Detects when Azure lists only the same subscription it rejected, avoids a
   retry loop, and moves to reauthentication. A repeated rejection after
   refreshed authentication ends with clear account, tenant, and administrator
   guidance.
8. Added secure command-log parsing for Azure subscription JSON and focused
   regression coverage for subscription ID extraction and structured results.

Validation:

- `npm run build`: passed.
- Focused Azure subscription recovery tests: 2 passed.
- `cargo test --lib`: 98 passed.
- Automatic Rust formatting was unavailable because the local stable
  toolchain does not currently include the optional `rustfmt` component.

## Phase 2 slice - Closed gaps against the Azure/Terraform intent spec - 03/08/2026

A documentation-accuracy pass against the original Azure/Terraform intent
specification turned up two intents the spec called for that were never
implemented, plus confirmed one suspected gap was not actually a bug.

Completed:

1. Added `azure_provider_show` and `azure_provider_register` — the spec's
   Phase D resource-provider registration check ("Check if the Storage
   provider is registered." / "Register the Storage provider.") was flagged
   by the spec as an important first-class recovery path, since an
   unregistered provider on a brand-new subscription returns a misleading
   `SubscriptionNotFound` error that is easy to mistake for a broken login.
   Common short names (storage, compute, network, keyvault, sql, web,
   insights) resolve to their full `Microsoft.*` namespace; anything already
   shaped like `Microsoft.X` passes through unchanged. Both intents require
   the same Azure login prerequisite as the rest of Phase D-I, per the
   spec's own requirement.
2. Added `azure_account_clear` ("Clear my Azure login.") — the spec's Phase B
   fix for stale/corrupted cached tokens that a normal re-login alone does
   not resolve. Deliberately excluded from the login-prerequisite wrapper,
   since requiring a working login before a command whose purpose is fixing
   a broken login would be self-defeating.
3. Classified all three in `policy.rs` for direct command entry: provider
   show is Safe (read-only), provider register and account clear are
   Caution (matching the spec's risk table).
4. Re-verified the previously reported "`az vm stop` missing from the Azure
   login-preflight list" concern by reading `intent_adapter.rs` directly
   rather than relying on an earlier research pass — `azure_vm_stop` was
   already present in `azure_login_required` alongside start/deallocate/
   show/create. No fix was needed; the earlier report was inaccurate on
   this specific point.

Validation:

- `npm run sidecar:test`: 76 passed (6 new parametrized matcher cases).
- `npm run sidecar:build`: rebuilt the bundled binary so the real-sidecar
  integration tests below exercise the new Python logic.
- `cargo test --lib`: 103 passed, including 3 new `intent_adapter` render
  tests, 2 new `policy` classification tests, and 2 new real-sidecar
  integration tests (`azure_provider_show`, `azure_account_clear`) that
  spawn the actual rebuilt binary.
- `npm run build`: passed.
- Not yet manually verified in the running app in this session.

Next:

1. Manually try "Check if the Storage provider is registered.", "Register
   the Storage provider.", and "Clear my Azure login." in the running app.
2. Continue auditing the original intent specification against the
   implementation for any other gaps before treating Phase 2's Azure/
   Terraform coverage as complete.

## Phase 2 slice - Stripped ANSI escape codes from command output - 03/08/2026

The user ran `terraform init` through the app and its output showed raw
escape sequences (`[0m[1m`, etc.) instead of the plain text those codes are
meant to format. TerminalMate streams plain text, not a real terminal — no
PTY is allocated — so color-aware tools (Terraform, git, npm, cargo, and
effectively anything else run through it) that emit ANSI codes for bold/color
formatting had no way to have those codes interpreted; they just appeared as
literal garbage in both the live transcript and the log file.

Completed:

1. Added `strip_ansi_escapes` in `command_runs.rs`, matching standard CSI
   escape sequences (`ESC [ ... <final byte>`), and applied it to every line
   in `spawn_line_pump` before it reaches the log file or the streamed
   `session-output` event. This is tool-agnostic — it does not depend on any
   specific program respecting `NO_COLOR` or a `-no-color` flag, so it
   covers Terraform, git, npm, cargo, and anything else a user runs.
2. Added a regression test using the user's own pasted `terraform init`
   output as the exact input.

Validation:

- `cargo test --lib`: 104 passed, including the new ANSI-stripping test.
- `npm run build`: passed.
- Not yet manually re-verified against a real `terraform init` run in this
  session — recommend re-running the same command that surfaced the bug.

Next:

1. Re-run `terraform init` (or any other Terraform/git/npm command) in the
   app and confirm the output reads as clean plain text.

## Phase 2 slice - Terraform apply/destroy execute via typed RUN, not Explain Only - 03/08/2026

The user questioned why `terraform apply`/`terraform destroy` required
leaving the app to run manually when TerminalMate already has a typed-`RUN`
approval gate for High Risk operations. Investigating surfaced the real
reason Explain Only existed for these two specifically: a directly-typed
`terraform apply` run earlier in the session had hung indefinitely at
Terraform's own interactive `Enter a value:` confirmation prompt, because
TerminalMate runs commands with no stdin channel (`Stdio::null()`) and has
no way to answer it. Explain Only was masking that mechanical limitation
behind what looked like a pure risk-tolerance decision.

Completed:

1. `intent_adapter.rs` now renders both `terraform_apply` and
   `terraform_destroy` with `-auto-approve` appended, removing Terraform's
   own confirmation prompt from the equation entirely.
2. Removed both actions from the Explain Only hard-coded list in
   `intents.rs`'s `execution_guidance`. They now fall through to ordinary
   classification — already High Risk in `policy.rs` — and require
   TerminalMate's typed-`RUN` confirmation as the sole approval gate, in
   place of Terraform's own rather than in addition to it.
3. Storage-key retrieval and resource-group deletion remain Explain Only,
   unaffected — their block is a deliberate credential/irreversibility
   judgment, not a stdin limitation, so they were not in scope for this
   change.
4. Direct (literally-typed) `terraform apply`/`destroy` without
   `-auto-approve` still hangs at the interactive prompt exactly as it did
   before — this change only affects the plain-English intent path, which
   is the only place TerminalMate controls the rendered command text.

Validation:

- `cargo test --lib`: 106 passed, including 2 new `intents` tests
  (`execution_guidance` now returns Execute for both actions, Explain Only
  is unchanged for the remaining two), and updated `intent_adapter`
  golden-matrix/exact-render assertions for the new `-auto-approve` text.
- `npm run build`: passed.
- Not yet manually re-verified against a real `terraform apply` run in this
  session.

Next:

1. Run `terraform apply` via a plain-English request in the app, confirm it
   completes (no hang at a confirmation prompt) after typing `RUN`.
2. Consider whether directly-typed `terraform apply`/`destroy` (without
   `-auto-approve`) deserves its own guardrail, since it can still hang the
   same way the original bug report showed — out of scope for this slice,
   which only touched the intent-rendering path.

## Phase 2 slice - Blocked unanswerable direct terraform apply/destroy - 03/08/2026

Closed the residual gap noted above: a directly-typed `terraform apply` or
`terraform destroy` with no `-auto-approve` and no piped input still hung at
Terraform's own confirmation prompt exactly like the original bug report,
since direct commands run verbatim and TerminalMate only controls rendered
text on the intent-matched path.

Completed:

1. Added `is_unanswerable_terraform_confirmation` in `policy.rs`, checked
   alongside the existing broad-root-delete and credential-exfiltration
   pre-checks, ahead of the ordered rule table. A bare `terraform
apply`/`destroy` (no `-auto-approve`, no `|` pipe) is now Blocked with an
   explanation and two concrete ways to unblock it: add `-auto-approve`, or
   pipe a confirmation (`echo yes | terraform apply`).
2. This is a certainty, not a risk judgment, so Blocked was the right tier —
   the command would hang unconditionally, not just riskily.
3. A `|` pipe is treated as evidence of a deliberate answer already being
   supplied and is not blocked; `-auto-approve` also lifts the block,
   matching what the intent-rendering path already does automatically.

Validation:

- `cargo test --lib`: 107 passed, including a new dedicated test and an
  updated `classifies_terraform_by_infrastructure_impact` (bare apply/destroy
  now assert Blocked instead of HighRisk; the `-auto-approve` variants stay
  HighRisk).
- `npm run build`: passed.
- Not yet manually re-verified against a real direct `terraform apply` in
  this session.

Next:

1. Type `terraform apply` directly in the app and confirm it is refused with
   the explanation rather than left to hang.
2. Confirm `terraform apply -auto-approve` typed directly still runs through
   the normal High Risk / typed-`RUN` path.

## Phase 2 slice - Blocked full-screen interactive editors and monitors - 03/08/2026

The user ran `nano infra/azure/main.tf` in a WSL Bash workspace. The output
was garbled terminal control sequences (`Standard input is not a terminal`,
then raw charset-designation escapes) rather than an editor, and the run
failed with exit 1. This is a more fundamental version of the Terraform
hang from the previous two slices: `nano` isn't blocked by one unanswerable
prompt, it needs a real terminal for everything it does — cursor
positioning, full-screen redraw, live keystrokes — none of which
TerminalMate's piped-text execution model can ever provide. No amount of
escape-sequence stripping fixes this; the program simply cannot function
here.

Completed:

1. Added `is_interactive_editor` in `policy.rs`, alongside the other Blocked
   pre-checks. Matches `nano`, `vim`/`vi`, `emacs`, `pico`, `top`, and `htop`
   as a command token (start of input, or after `;`/`&`/`|`), so it does not
   fire on words that merely contain those letters (`*.view`, `top level`).
2. The refusal message explains why (no PTY, not a policy restriction) and
   suggests the working alternative already supported: `cat <file>` or a
   plain-English "show me the contents of `<file>`" request to view a file;
   a code editor outside TerminalMate to edit one.

Validation:

- `cargo test --lib`: 108 passed, including a new dedicated test covering
  each blocked program plus two words that must not false-positive.
- `npm run build`: passed.
- Not yet manually re-verified against a real `nano` run in this session.

Next:

1. Type `nano <file>` in the app and confirm it is refused with the
   explanation instead of producing garbled output.
2. Consider whether other interactive-only programs (`less`, `more`, `man`,
   `ssh` without a command argument) need the same treatment — out of scope
   for this slice, which focused on the concrete case reported.

## Phase 2 slice - Built-in file editor as a nano/vim workaround - 03/08/2026

The user pushed back on the nano block with evidence that Warp (a real
terminal emulator) runs `nano` fine, asking whether TerminalMate could too.
The honest answer: Warp works because it allocates a real PTY; TerminalMate
doesn't, by design, and adding one would break the safety model's premise of
classifying one command at a time (a live PTY session is a stream of
keystrokes, not a single classifiable command). Asked what to do instead,
the user proposed a workaround: load the file into a textbox, edit it there,
save it back — sidestepping the terminal-emulation question entirely rather
than solving it.

Completed:

1. Added `commands/file_editor.rs` with `read_editable_file` and
   `write_editable_file` Tauri commands — direct file I/O, no shell process
   and no PTY involved at all, which is a safety improvement over the
   original ask, not a compromise of the existing block.
2. Added `wsl_path_to_windows` in `workspaces.rs`, the reverse of the
   existing `windows_path_to_wsl`, since a WSL workspace's working directory
   is WSL-style (`/mnt/d/...`) but Tauri commands run natively on the
   Windows host with no direct filesystem access to a WSL path.
3. A missing filename reads as an empty buffer rather than an error,
   matching what `nano <newfile>` itself does. Files over 2 MB are refused
   rather than loaded into a plain `<textarea>`.
4. Extended `RiskDecision` with an `editable_path` field, populated only for
   the Blocked interactive-editor case when a file path can be confidently
   read off the command (`nano main.tf` -> `main.tf`; `top`/`htop` never
   populate it, since they take no file argument).
5. Added `FileEditorModal.tsx` (modeled on the existing Help dialog) and an
   "Open in built-in editor" button on the blocked notice whenever
   `editablePath` is present. The modal reads the file, edits in a plain
   textarea, and Save calls `write_editable_file` — no approval-card
   ceremony beyond the Save button itself, since this is a direct file
   write rather than a shell command running through the policy engine.

Validation:

- `cargo test --lib`: 120 passed, including 7 new `file_editor` tests (path
  resolution across Windows/WSL working directories and absolute-path
  forms, missing-file-as-empty-buffer, round-trip write/read, size-limit
  rejection), 4 new `workspaces` tests for `wsl_path_to_windows` (including
  a round-trip test against the existing `windows_path_to_wsl`), and 1 new
  `policy` test for `editable_path` extraction.
- `npm run build`: passed.
- Not yet manually verified against a real `nano <file>` run in this
  session.

Next:

1. Type `nano infra/azure/main.tf` in a WSL workspace, confirm the "Open in
   built-in editor" button appears, opens the correct file, and Save writes
   it back correctly.
2. Consider whether the same button should appear for `vim`/`emacs`, or
   whether their users are more likely to want the real editor's features —
   currently offered identically for all three.

## Phase 2 slice - Workspace tab warning badge - 04/08/2026

The user shared a Warp terminal screenshot (its "+" new-tab menu, alongside
a session list showing a warning icon on a "nano" tab) and asked what, if
anything, was worth taking from that UI. Two ideas came out of it: letting
one workspace hold multiple sessions on different shells (a real session-
model change, intentionally not started without further scoping), and a
cheap, no-architecture-cost one implemented now — a warning badge directly
on the workspace list so a failing session doesn't go unnoticed just
because it isn't the one currently open.

Completed:

1. `WorkspaceSidebar.tsx` now accepts a `warnings: Record<string, boolean>`
   prop and renders a small red `TriangleAlert` icon next to a workspace's
   name when set.
2. `App.tsx` computes it with a `useMemo`: a workspace is flagged only when
   its session has a finished run that neither succeeded nor was stopped by
   the user, and only while no run is currently active in that session (an
   in-progress run supersedes whatever the previous outcome was). This reuses
   state (`sessionsByWorkspace`, `activeRunBySession`,
   `lastFinishedBySession`) that already existed for other purposes — no new
   Rust code or events were needed.

Validation:

- `npm run build`: passed.
- No Rust changes in this slice; existing 120 Rust tests unaffected.
- Not yet manually verified against a real failing background workspace in
  this session (e.g. switch away from a workspace mid-command, let it fail,
  confirm the badge appears on the unfocused workspace row).

Next:

1. Manually verify the badge appears/clears correctly by causing a failure
   in a non-active workspace and switching back to confirm it cleared after
   a subsequent successful run.
2. Decide whether to scope the bigger idea (multiple concurrent
   sessions per workspace across different shells, Warp-style) as a future
   roadmap item, or leave the current one-runtime-per-workspace model as is.

## Phase 2 slice - SSH login hang and external-terminal handoff - 04/08/2026

A real capstone deployment surfaced the bug directly: after `terraform
apply -auto-approve` created an Azure VM, an NLP-resolved `ssh_vm` request
connected to it and then sat at "Running" for 26+ minutes with no further
output past `Pseudo-terminal will not be allocated because stdin is not a
terminal.` — the SSH equivalent of the earlier Terraform confirmation hang,
one layer further from the local machine (a new VM's SSH host key has never
been seen before, so OpenSSH's own "are you sure?" prompt, or a password
prompt, has nothing to answer it on a `Stdio::null()` stdin).

This reopened the standing "should TerminalMate build a real PTY" question,
raised again given three separate no-PTY bugs this session (ANSI leakage,
Terraform, now SSH). Recommended against it: a live PTY session is a stream
of keystrokes, not one classifiable command, so it would break the
classify-then-approve safety model that is the actual product, not just add
an engineering cost — worse, an SSH PTY session would execute arbitrary
commands on a remote host TerminalMate has no policy visibility into at
all, with no approval gate whatsoever. The user then proposed the actual
fix: redirect commands that generically need a real terminal to an external
terminal window rather than emulating one in TerminalMate itself.

Completed:

1. Added `policy::is_unactionable_ssh_login`: blocks a bare `ssh user@host`
   (or `ssh` plus flags and a destination) that has no trailing remote
   command, since that opens a full interactive login shell needing a real
   terminal — the same problem as `nano`, just one hop further away. `ssh
user@host <command>` is unaffected (Caution, like any other remote-host
   command): it runs one command over the connection and returns.
   Implemented as manual tokenizing rather than a single regex, since the
   `regex` crate has no lookahead and there is no cheap way to distinguish
   `ssh` as a command token from `ssh-keygen`, or "a destination with
   nothing after it," as one pattern.
2. Extended `RiskDecision` with an `external_terminal: bool` field —
   distinct from `editable_path` — set when a Blocked command has no
   in-app workaround but would work fine in a real terminal: the new bare
   SSH login case, and `top`/`htop` (which previously had no escape hatch
   at all). `nano`/`vim`/`emacs`/`pico` with a resolvable file path still
   prefer the built-in editor over this.
3. Split the interactive-editor Blocked reason in two: the existing
   editor-specific text (mentions `cat <file>` and the built-in editor) and
   a new, separate monitor-specific reason for `top`/`htop`, which have no
   file to view or edit.
4. Fixed the `ssh_vm` intent's render in `intent_adapter.rs` for both
   shells: added `-o BatchMode=yes -o StrictHostKeyChecking=accept-new`
   (auto-trusts a new host key the same way `git clone` over SSH already
   does via TOFU, and fails fast instead of hanging if a password would
   have been needed). This still renders a bare login — by design, since it
   now hits the new Blocked check and offers the terminal handoff below
   rather than ever attempting to run inside TerminalMate's own panel.
5. Added `open_external_terminal` (new `external_terminal.rs`): spawns
   `powershell.exe` or `wsl.exe` in a brand-new, real console window
   (`CREATE_NEW_CONSOLE`, not `cmd /C start`, avoiding a second layer of
   shell-metacharacter escaping) at the session's working directory,
   running the exact blocked command and then dropping into an interactive
   shell (`exec bash` / `-NoExit`) so the user can keep working in it. This
   is a real, TerminalMate-unsupervised terminal — intentionally not
   logged or classified further, since the whole point is handing off a
   command TerminalMate cannot run.
6. Wired a second escape-hatch button, "Open in a new terminal window," on
   the Blocked notice in `CommandBar.tsx` alongside "Open in built-in
   editor," conditioned on `externalTerminal`/`editablePath` respectively.

Validation:

- `cargo test`: 123 passed, including new tests for the bare-login block
  (allows a trailing remote command, `ssh-keygen`/`scp` unaffected, `ssh -V`
  with no destination unaffected), the editor-vs-monitor reason/flag split,
  and the `ssh_vm` render (both shells contain the new flags).
- `npm run build`: passed.
- Not yet manually verified against a real hung SSH session in this
  session — the original bug was captured via screenshot, not reproduced
  live afterward.

Next:

1. Manually verify: let the guided Azure VM flow reach `ssh_vm` again,
   confirm it is now Blocked immediately (not left hanging), and that "Open
   in a new terminal window" opens a real, working interactive SSH session
   at the VM.
2. Confirm the currently-running hung session from the bug report has been
   stopped by the user (via the Stop button) rather than left consuming a
   session slot.

## Phase 2 slice - Guided VM lifecycle decision - 04/08/2026

A separate conversation about the real glaucoma-detection deployment
surfaced a "what should happen to this VM now" decision (leave it running /
`az vm deallocate` / `terraform destroy`), presented with a recommendation
and reasoning. Asked whether TerminalMate could produce that same kind of
decision itself, rather than it only existing as free-form chat text — the
answer was yes, using the same guided-card pattern already established by
Azure Recovery, triggered automatically at the one moment it actually
matters: right after `terraform apply` finishes creating a VM.

Completed:

1. Added `find_created_virtual_machine` / `detect_created_virtual_machine`
   in `command_runs.rs`: parses a successful `terraform apply` log tail for
   an Azure VM resource ID
   (`resourceGroups/<rg>/providers/Microsoft.Compute/virtualMachines/<name>`)
   and returns the resource group and VM name if one was created. Reuses
   the existing `terminal_mate_log_path`/`read_log_tail` helpers already
   used by `diagnose_command_failure`.
2. `CommandBar.tsx` watches `lastFinished`/`lastHistoryEntry` for a
   successful command matching `terraform apply`, calls the new command
   once per run (guarded by a ref, same pattern as the Azure-recovery
   effect), and shows a new guided card when a VM was detected.
3. The card offers three choices, mirroring the original conversation:
   leave it running (dismiss, no command), deallocate (`az vm deallocate`,
   built directly from the detected name/resource group), or destroy with
   Terraform (`terraform destroy -auto-approve`, marked recommended for a
   learning project with no live traffic). Deallocate and destroy both go
   through the normal `classifyAndQueue` path — the same HighRisk
   typed-RUN approval as running either command by hand, since the guided
   card is a shortcut to the command, not a bypass of its review.
4. Added a `.recommended` modifier on `.command-recovery-action` (green
   accent, matching the existing "Default" subscription badge color) so
   the suggested option is visually distinct without a separate component.

Validation:

- `cargo test`: 125 passed, including 2 new tests for
  `find_created_virtual_machine` (detects a VM from the user's own
  `terraform apply` log excerpt; does not fire when apply only touched
  unrelated resources like a bare resource group).
- `npm run build`: passed.
- Not yet manually verified against a real `terraform apply` run that
  creates a VM in this session.

Next:

1. Manually verify: run a `terraform apply` that creates a VM, confirm the
   guided card appears with the correct name/resource group, and that each
   of the three choices behaves as expected (dismiss, deallocate through
   approval, destroy through approval).
2. Consider whether other Terraform-created resource types (storage
   accounts, AKS clusters) warrant the same guided-decision treatment, or
   whether VMs are the common case worth covering for now.

## Phase 2 slice - scp remote-directory-missing diagnosis - 05/08/2026

The user hit this directly deploying to the same VM:
`scp -i ~/glaucoma-ai-azure-key -r backend/models/ azureuser@...:~/glaucoma-detection/backend/`
failed with `scp: realpath ...: No such file` / `path canonicalization
failed`. Not a TerminalMate architecture problem this time — a real scp
behavior: its (newer, SFTP-based) upload protocol requires the destination
directory to already exist on the remote host; `-r` does not create missing
parent folders. The existing generic `missingPath` diagnosis did not catch
this wording (`no such file`, not `does not exist`), and would have offered
"List files in this folder" regardless — useless here, since the missing
path is on the _remote_ host, not the local one.

Completed:

1. Added a `scpRemoteDirectoryMissing` diagnosis case in
   `diagnose_failure` (`command_runs.rs`), gated on the command starting
   with `scp` and the output containing `realpath` or
   `path canonicalization failed` — checked ahead of the generic
   `missingPath` case so it does not fall through to unhelpful local-path
   guidance.
2. Added `suggest_remote_mkdir_command`: reconstructs a working
   `ssh -i <key> -o BatchMode=yes -o StrictHostKeyChecking=accept-new
<user@host> "mkdir -p <remote-dir>"` from the failed scp command's own
   `-i` flag and `user@host:path` destination, populated as the
   diagnosis's `suggested_request`. Reuses the same BatchMode/
   StrictHostKeyChecking flags as the `ssh_vm` intent render, since this is
   the same one-shot non-interactive ssh usage that fix was written for.
3. No frontend changes needed — `CommandBar.tsx`'s existing
   `Try: <suggestedRequest>` button already handles any diagnosis kind
   generically (only `azureSubscriptionNotFound` has bespoke UI); a new
   `kind` value with a `suggestedRequest` populated the message box for
   free.

Validation:

- `cargo test`: 127 passed, including a test using the user's exact
  command/output pair (confirms both the diagnosis kind and the exact
  reconstructed `ssh ... mkdir -p ...` suggestion), and a test confirming
  an ordinary local `Cannot find path ... does not exist` failure still
  resolves to the existing `missingPath` diagnosis, not the new scp one.
- `npm run build`: passed.
- Not yet manually verified against a real failed scp upload in this
  session — the failure was reported via pasted output, not reproduced
  live afterward.

Next:

1. Manually verify: trigger the same scp failure again, confirm the new
   diagnosis card appears with the reconstructed `ssh ... mkdir -p ...`
   suggestion, and that running it (then retrying the original scp) works
   end-to-end.

## Phase 2 slice - Deallocate a VM by name only (no resource group) - 05/08/2026

`azure_vm_start`, `azure_vm_stop`, and `azure_vm_deallocate` all share one
sidecar phrase template requiring both `vm_name` and `resource_group`
("deallocate vm X in Y"). Asked to add a friendlier path specifically for
deallocate (the one actually used for cost control): typing just the VM
name, with the resource group auto-resolved via `az vm list` — using the
one record directly if the subscription only has one VM, otherwise letting
the user pick. Scoped explicitly to deallocate only; start and stop are
unchanged, per the user's own choice when asked.

Completed:

1. Added `parseVmDeallocateRequest` (new `vmDeallocateRequest.ts`): a
   frontend-only regex for `deallocate (vm|virtual machine) <name>` with
   nothing after the name — mirrors `parseHelpRequest`'s existing precedent
   of a local match that never reaches the sidecar, used here because this
   phrase resolves via a multi-step lookup, not a single deterministic
   render. Anchored to end-of-string, so "deallocate vm X in Y" (both
   params given) still falls through unmatched to the existing sidecar
   intent, unchanged.
2. Added `read_azure_virtual_machines` / `parse_azure_virtual_machines` in
   `command_runs.rs`, parsing `az vm list --output json` — deliberately
   JSON rather than the `--output table` initially described, since the
   goal is programmatic disambiguation, not a rendered table; matches the
   same choice already made for the existing subscription-check lookup.
   Refactored the JSON-array-extraction logic out of
   `parse_azure_subscriptions` into a shared generic
   `extract_json_array<T>` helper used by both.
3. Wired a guided resolution flow in `CommandBar.tsx`, structurally
   parallel to Azure Recovery and the VM-lifecycle-decision card: typing
   the new phrase runs `az vm list` (Safe, auto-runs) via the normal
   `classifyAndQueue` path, then an effect reads the result and either (a)
   deallocates directly if there is exactly one VM in the subscription, or
   the typed name uniquely matches one VM among several, or (b) shows a
   picker card when the name is ambiguous or absent among multiple VMs.
   Picking one, or the direct-match path, both still route the actual
   `az vm deallocate` through `classifyAndQueue` — same HighRisk typed-RUN
   approval as any other route to that command.

Validation:

- `cargo test`: 129 passed, including 2 new tests for
  `parse_azure_virtual_machines` (a real `az vm list --output json`
  excerpt, and an empty list).
- `npm run build`: passed.
- Not yet manually verified against a real "Deallocate vm <name>" phrase
  in this session.

Next:

1. Manually verify: type `Deallocate vm glaucoma-ai-azure-vm` with only one
   VM in the subscription, confirm it resolves and deallocates directly
   with no picker; then (if practical) test the multiple-VM picker path.
2. Decide whether `azure_vm_start`/`azure_vm_stop` should eventually get
   the same name-only + auto-resolve treatment, or stay as they are — left
   open per the user's own scoping choice this round.

**Correction (same day):** the first version of `parseVmDeallocateRequest`
still required a VM name ("deallocate vm `<name>`"), so a bare "Deallocate
vm" with no name at all fell through to the old sidecar intent and was
rejected ("Provide both the VM name and resource group."). The user's
actual ask was for the name to be optional too — clarified as: if _either_
piece is missing, run the list-and-resolve flow; only fall back to asking
the user is the "let them pick" step, never a hard requirement to name the
VM. Fixed by making the regex's name group optional
(`deallocate (?:the |my )?vm(?:\s+(\S+))?`) and updating the resolution
effect to skip name-based filtering entirely when no name was given
(picker shows every VM found, not just name matches).

## Phase 2 slice - Extended name-only resolution to az vm start - 05/08/2026

Asked to bring `az vm start --resource-group X --name Y` under the exact
same name-only + auto-resolve treatment just built for deallocate, since
both take the identical two parameters. Rather than duplicate the whole
resolution flow a second time for one differing verb, generalized the
deallocate-only version into one shared `start`/`deallocate` flow.

Completed:

1. Renamed `vmDeallocateRequest.ts` to `vmActionRequest.ts`;
   `parseVmActionRequest` now recognizes `(start|deallocate) [the|my] vm
[name]`, returning which action matched alongside the optional name.
   `azure_vm_start`/`azure_vm_stop` remain untouched at the sidecar level —
   this is still a frontend-only local match, same as deallocate's.
2. Generalized `CommandBar.tsx`'s single-purpose deallocate state machine
   (`VmDeallocateResolution`, `beginVmDeallocateResolution`,
   `chooseVmToDeallocate`, one `useEffect`) into an action-parameterized
   one (`VmActionResolution` carries an `action: "start" | "deallocate"`
   field; a single `runVmAction(action, vm)` builds
   `az vm <action> --name X --resource-group Y` for whichever action was
   requested). One resolution effect, one picker render block, and one
   `VM_ACTION_LABEL` map now serve both verbs instead of two near-identical
   copies.
3. Switched the underlying lookup from `az vm list --output json` to
   `az vm list --show-details --output json` and added an optional
   `power_state` field to `AzureVirtualMachine` (Rust and TS). The picker
   now shows each candidate's power state (e.g. "VM running" / "VM
   deallocated") next to its resource group — directly useful context when
   the decision is _which_ VM to start or stop, not just disambiguation.

Validation:

- `cargo test`: 130 passed, including 2 new tests for the `power_state`
  field (present when `--show-details`-style JSON includes it, absent —
  not an error — when it does not).
- `npm run build`: passed.
- Not yet manually verified against a real "Start vm `<name>`" phrase in
  this session.

Next:

1. Manually verify: type `Start vm glaucoma-ai-azure-vm` (or bare `Start
vm` with the VM already deallocated), confirm it resolves the same way
   deallocate did, and that the picker (if multiple VMs) shows power state
   correctly.

## Phase 2 slice - Automatic status check after start/deallocate - 06/08/2026

Asked for a follow-up step: after any `az vm start` or `az vm deallocate`
actually runs, automatically run `az vm list --show-details --output
table` right after, so the resulting state is visible without having to
ask separately.

Completed:

1. Added a `useEffect` in `CommandBar.tsx` watching
   `lastHistoryEntry.command` (the literal rendered/typed command, not the
   plain-English request) for `az vm (start|deallocate)` — this catches
   every path that can produce that command: the new guided
   resolve-by-name flow, the older sidecar `azure_vm_start`/
   `azure_vm_deallocate` intent (both params given), and a directly typed
   `az vm start/deallocate ...` command alike.
2. On a match, automatically runs `az vm list --show-details --output
table` via the normal `classifyAndQueue` path (Safe, auto-runs, appears
   as its own "Check Azure VM status" entry in Recent Runs). Unlike the
   `--output json` lookups added for the resolve-by-name flow, this one is
   deliberately `--output table` — there is nothing left to parse
   afterward, it exists purely for the user to read.
3. Runs regardless of whether the start/deallocate itself succeeded or
   failed, since the resulting state is informative either way (e.g. a
   failure because the VM was already in that state is exactly what the
   status check would reveal).

Validation:

- `npm run build`: passed. No Rust changes in this slice.
- Not yet manually verified against a real start/deallocate run in this
  session.

Next:

1. Manually verify: start or deallocate a VM (any path — guided flow,
   typed command, or plain-English "start vm X in Y"), confirm the status
   table run appears automatically right after and shows the correct
   resulting power state.

## Phase 2 slice - azure_vm_list now includes power state - 06/08/2026

Asked what the short plain-English phrase was for
`az vm list --show-details --output table` — there wasn't one; the
existing `azure_vm_list` intent ("list azure vms" / "show azure vms")
rendered the plainer `az vm list --output table`, with no power state.
Closed the gap by making `--show-details` the default for that intent
rather than adding a second, separate phrase, since power state is what
you actually want whenever listing VMs, not a rare special case.

Completed:

1. `azure_vm_list`'s render in `intent_adapter.rs` changed from
   `az vm list --output table` to `az vm list --show-details --output
table`. Still classifies Safe (the policy pattern matches on the
   `az vm list` prefix, unaffected by trailing flags) and still gets the
   `az account show` login prerequisite wrap like before.
2. Added a render test asserting both the login prerequisite and the
   `--show-details` flag, since this intent previously had no dedicated
   test of its own (only exercised indirectly via the `azure_login_required`
   match-arm test).

Validation:

- `cargo test`: 131 passed (1 new).
- No sidecar or frontend changes needed — the phrase and matching were
  already correct, only the deterministic render changed.

## Phase 2 slice - Required az/azure marker on every Azure pattern - 07/08/2026

Asked to make the "az"/"azure" marker mandatory (never optional) across
every Azure plain-English pattern, anticipating a future AWS adapter: a
phrase like "start vm X" should not be silently assumed to mean Azure once
"start aws vm X" could plausibly mean something else. Clarified: both "az"
and "azure" count as valid markers; the SSH-key intents (`generate_ssh_key`,
`show_ssh_public_key`, `ssh_vm`) stay unchanged, since SSH itself isn't
cloud-specific.

Completed:

1. `local_matcher.py`: converted 13 regex patterns from an optional
   `(?:\s+azure)?` marker to a required `(?:az|azure)` — subscription set,
   resource group create/delete, storage account/container create, storage
   keys list, VM show/start/stop/deallocate, resource list. Also added the
   marker to `azure_provider_show`/`azure_provider_register`, which
   previously had no marker in their pattern at all.
2. Removed two alias phrases that had no marker either — "list virtual
   machines" (`azure_vm_list`) and "list network security groups"
   (`azure_nsg_list`) — since equivalent azure-marked aliases already exist
   for both, and an unmarked alias is exactly the ambiguity being closed.
3. Updated the three "near-miss" clarification patterns (used to give
   helpful "you're missing a parameter" guidance for an almost-matching
   phrase) to require the same marker, so a marker-less phrase is no longer
   treated as an Azure request at any stage, not just the successful-match
   one.
4. Updated the frontend-only `vmActionRequest.ts` shortcut ("start/deallocate
   vm [name]", no resource group) the same way — marker now required there
   too, word order adjusted so "start the azure vm" / "start my az vm" both
   read naturally.
5. Fixed the 4 sidecar test phrases that had relied on the marker being
   optional (the provider show/register cases had no marker in the test
   message at all) and added a new parametrized test asserting that
   marker-less phrases for VM start/deallocate, resource group create, and
   provider show now correctly fail to match.

Validation:

- `pytest`: 80 passed (4 new).
- `cargo test`: 131 passed, including both real-sidecar integration tests
  (`azure_provider_register`, `azure_account_clear`), confirming their
  already-marked phrases still resolve correctly end-to-end.
- `npm run build`: passed.
- README's example phrases were already checked — every one already said
  "Azure" in the right place, so no doc changes were needed.
- Known, deliberate behavior change: a bare phrase like "deallocate vm X"
  (no "az"/"azure" at all) no longer matches anything Azure-specific — not
  the sidecar intent, not the near-miss clarification, and not the
  frontend shortcut. It now falls through to being classified as an
  ordinary (unrecognized) direct command instead. This is the intended
  effect of requiring disambiguation, not a regression, but it does mean
  the old "Provide both the VM name and resource group" guidance no longer
  appears for a marker-less phrase — the guidance still fires for e.g.
  "deallocate az vm" (marker present, name missing).

Next:

1. Manually verify a few phrases end-to-end: "start az vm" (no name, no
   resource group — should trigger the guided resolve-by-name flow),
   "deallocate azure vm X in Y" (should still resolve directly, unchanged),
   and a bare "start vm X" (should no longer be recognized as anything
   Azure-specific).

## Phase 2 slice - Two flagged gaps closed - 08/08/2026

Two small gaps noticed while auditing the full Azure command list, closed
now rather than left as unaddressed asides.

Completed:

1. **`az storage account keys list` policy gap**: this command retrieves
   Azure storage account access keys (credential-equivalent material) but
   previously matched no explicit rule, so it silently fell through to the
   generic "Unrecognized command; review before running" Caution
   catch-all. Added a dedicated Caution rule with the reason "Reveals
   Azure storage account access keys," same precedent as the existing
   `kubectl get secrets` rule — a read-only command is not automatically
   Safe just because it changes nothing, if what it reveals is sensitive.
2. **`show_ssh_public_key` test coverage gap**: this intent was
   implemented (sidecar pattern + Rust render) but had zero test coverage
   at either layer. Added two sidecar phrase-match cases ("Show the public
   key named X", "Show SSH public key for X") and a Rust render test
   covering both shells plus the same key-name path-sanitization
   (`/`/`\` → `_`) that `generate_ssh_key` already has tested.

Validation:

- `pytest`: 82 passed (2 new).
- `cargo test`: 132 passed (1 new, plus 2 new assertions on the existing
  `classifies_azure_commands_by_cloud_impact` test).

## Phase 2 slice - Diagnostic-gap logger - 08/08/2026

The user hit `docker exec -it glaucoma_backend pip show slowapi` failing
with "cannot attach stdin to a TTY-enabled container because stdin is not
a terminal" — the exact same no-PTY root cause already fixed for ssh,
nano, and bare terraform apply, just not yet recognized for `docker exec
-it`. Asked for a logger that collects every such failure so patterns like
this can be spotted and turned into real fixes over time, instead of each
one only being visible by chance inside its own run's log.

Completed:

1. `diagnose_command_failure` now also appends a record to a new
   `diagnostic-gaps.log` (JSON-lines, one object per line) in the app log
   directory, alongside the existing per-run command logs, whenever
   `diagnose_failure` falls back to the generic "nonzeroExit" diagnosis —
   i.e. exactly the failures TerminalMate does not yet have specific
   guidance for. Each entry: timestamp, the failing command, exit code,
   and a short trailing snippet of its output (last 500 characters) so the
   file is scannable at a glance without reopening every individual run
   log.
2. Deliberately best-effort and silent on failure (missing directory, IO
   error): logging is a convenience for spotting patterns later, not
   something any actual command's success should ever depend on.
3. Does not touch `CommandFailureDiagnosis`'s shape or the frontend at
   all — this is a side effect of diagnosis, invisible to the running app.

Validation:

- `cargo test`: 136 passed (4 new: two for `record_diagnostic_gap`'s file
  output using the user's own docker/pip example, two for the
  `tail_snippet` helper).

Next:

1. The `docker exec -it` case itself is still unfixed — it will now show
   up in `diagnostic-gaps.log` rather than silently vanishing, but it has
   not been given its own diagnosis or policy guardrail yet, unlike the
   ssh/nano/terraform cases that share the exact same root cause. Worth
   doing as its own follow-up: `docker exec -it`/`docker run -it` reliably
   fail or hang under TerminalMate's no-PTY execution model, so `-it`
   could reasonably join the same Blocked-with-guidance treatment.
2. Periodically review `diagnostic-gaps.log` for repeat entries — that is
   the actual mechanism this slice built for "based on that we can fix and
   support that command."

## Phase 2 slice - Recent runs: dedupe display, keep full history - 08/08/2026

Asked to stop duplicate entries cluttering "Recent runs" while still
keeping the historical record — left the actual design choice open ("best
practice in the real world"). Auditing the existing code surfaced that
there effectively was no historical record beyond what the panel shows:
`storage.ts` capped persisted history at 10 entries per workspace (on both
load and save), and `App.tsx` separately hardcoded the same `.slice(0, 10)`
when appending a new run — once a workspace passed 10 runs, older ones
were gone for good, panel and storage alike.

Design (mirrors how shell history dedup actually works — e.g. bash's
`HISTCONTROL=ignoredups` — rather than a global merge):

- **Two separate caps, not one.** A generous _storage_ cap (how much is
  actually persisted, "the historical") and a small _display_ cap (how
  many rows the compact panel shows at once) are no longer the same
  number.
- **Dedup is display-only, consecutive-only.** Collapsing happens purely
  in the render layer, over the full stored history; the stored data
  itself is never merged or mutated. Only _consecutive_ identical commands
  collapse into one row with a repeat badge — a command repeated again
  later, after something else ran in between, stays its own separate row,
  since that sequence is meaningful, not noise.

Completed:

1. New `runHistory.ts`: `HISTORY_STORAGE_LIMIT` (10 → 200 per workspace —
   generous enough that normal use essentially never truncates, small
   enough to stay well within localStorage's practical limits) and
   `HISTORY_DISPLAY_LIMIT` (unchanged at 10 — the panel's visible size).
   `collapseConsecutiveDuplicateRuns(history)` returns display rows, each
   carrying the newest occurrence's data plus a `repeatCount`.
2. `storage.ts` and `App.tsx` both now import `HISTORY_STORAGE_LIMIT` from
   this single shared module instead of each hardcoding their own `10` (a
   real small inconsistency found while reading the code — two
   independent magic numbers that happened to agree, not one source of
   truth).
3. `ContextPanel.tsx` renders `collapseConsecutiveDuplicateRuns(history)`
   (capped to `HISTORY_DISPLAY_LIMIT`) instead of `history` directly; a row
   with `repeatCount > 1` shows a small "×N" badge next to its status.
   "Open log" / "run again" on a collapsed row act on the newest run in
   that streak, matching what is actually displayed.
4. Noted for the record, not fixed retroactively: existing users' already-
   saved history was already capped at 10 under the old code, so there is
   nothing beyond that to recover — this only stops _future_ history from
   being discarded so aggressively.

Validation:

- `npm run build`: passed (no frontend test runner exists in this repo, so
  `collapseConsecutiveDuplicateRuns` was verified by careful tracing and
  `tsc`'s type-checking rather than a unit test, consistent with how the
  rest of the frontend-only code in this project is verified).
- No Rust or sidecar changes in this slice.

Next:

1. Manually verify: run the same command several times in a row, confirm
   the panel shows one row with a "×N" badge instead of N separate rows;
   run something else in between two repeats of the same command and
   confirm both stay as separate rows.

## Phase 2 slice - Runtime-aware script execution - 09/08/2026

Added a reviewed workflow for running individual scripts or an ordered set of
scripts without making users translate the task into shell-specific syntax.

Completed:

1. Added `run <filename>` support with runtime-specific interpreter dispatch
   for PowerShell and WSL Bash workspaces.
2. Added batch execution for supported scripts in the current folder. The
   default order is newest modified first, with explicit filename and
   oldest-first ordering options.
3. Added optional recursive discovery and language filtering, including
   `Run all Python scripts recursively.`
4. Added a safe, read-only execution-order preview so users can inspect the
   discovered scripts before approving a batch run.
5. Batch scripts run sequentially and stop on the first nonzero exit code so a
   failed prerequisite cannot be silently followed by dependent scripts.
6. Single scripts and filters that are incompatible with the active runtime
   are rejected during planning with guidance to switch runtime.
7. Classified every script execution as Caution, preserving the standard
   plan, review, and approval boundary.
8. Added the workflow and its ordering variants to the in-app Everyday Help
   reference and README.

Validation:

- Sidecar intent tests cover single, ordered, filtered, recursive, and preview
  requests.
- Rust renderer tests cover both runtimes, interpreter dispatch, ordering,
  incompatible types, and read-only previews.
- Policy tests verify local script execution requires approval.

## Phase 2 slice - Live Python script output - 10/08/2026

Removed the pipe-buffering delay that could make an active Python script look
silent in TerminalMate even while the same script printed progress in a normal
interactive PowerShell window.

Completed:

1. Set `PYTHONUNBUFFERED=1` on every foreground PowerShell and WSL command
   process so exact commands such as `python check_for_updates.py` stream their
   printed progress through TerminalMate's existing line pump.
2. Applied the same environment to persistent terminal sessions for consistent
   behavior across direct terminal input and reviewed command runs. WSL starts
   Bash through `env PYTHONUNBUFFERED=1`, and one-shot WSL command wrappers
   export it inside Linux rather than relying on Windows environment import.
3. Updated generated Python script commands to use `python -u` on PowerShell
   and `python3 -u` on WSL Bash, making the unbuffered behavior explicit in the
   reviewed plan.
4. Added a compact running indicator inside the terminal surface so a silent
   command remains visibly active while TerminalMate waits for its next line.
5. Added Rust regression coverage for the inherited execution environment and
   updated the existing cross-runtime script-renderer assertions.

Expected result:

- A Python script that prints progress such as `[11/113] Checking ...` now
  displays each newline in TerminalMate while the process is running.
- Programs that do not emit output still show an active running state; they are
  not misreported as frozen or complete.

Validation:

- `npm run sidecar:test`: 89 passed, with one upstream Starlette deprecation
  warning.
- `cargo test --lib`: 143 passed.
- `npm run build`: TypeScript and the Vite production build passed.

## Phase 2 slice - Script arguments in natural-language runs - 10/08/2026

Extended single-script intent handling so users can describe an interpreter,
script, and arguments in one request without TerminalMate treating `run` as a
literal shell command.

Completed:

1. Added parsing for requests such as
   `Run python collect_manual.py --prefixes R --out ../generic-residence`.
2. Stored script arguments as a structured string list in the `run_script`
   intent instead of concatenating untrusted text into a shell fragment.
3. Rendered each argument as an individually escaped PowerShell or Bash
   literal while retaining runtime-aware `python -u` / `python3 -u` execution.
4. Preserved quoted multi-word values as one argument and returned
   clarification guidance for unmatched quotes.
5. Added the argument form to in-app Help and the README.

Expected result:

- The word `run` is interpreted as plain-English intent and is never passed to
  PowerShell as an executable.
- The reviewed command runs `collect_manual.py` with the four requested
  argument values after the standard approval step.

Validation:

- `npm run sidecar:test`: 91 passed, with one upstream Starlette deprecation
  warning.

## Phase 2 slice - Workspace-scoped file actions - 21/08/2026

Added guided file creation and deletion that resolves paths from the active
workspace instead of allowing arbitrary filesystem access.

Completed:

1. Added deterministic `create_file` and `delete_file` intents for plain-English
   requests such as `create file example.py` and `delete file scripts/example.py`.
2. Treated a leading slash or backslash as relative to the active workspace,
   while rejecting drive-absolute paths, UNC paths, and `..` traversal.
3. Made file creation refuse to overwrite an existing path and require Caution
   approval.
4. Made file deletion require High Risk confirmation and explicitly refuse to
   delete folders.
5. Added PowerShell and WSL command rendering, Help and README examples, and
   regression tests across the sidecar and Rust command boundary.

Validation:

- `npm run sidecar:test`: 98 passed, with one upstream Starlette deprecation
  warning.
- `npm run sidecar:build`: passed and rebuilt the packaged Windows sidecar.
- `cargo test --lib --quiet`: 163 passed.

## Phase 2 slice - Forgiving workspace navigation syntax - 21/08/2026

Improved guided folder navigation for Windows-style paths commonly entered as
`\backend\tests\`.

Completed:

1. Normalized one leading backslash and trailing path separators before
   rendering a workspace navigation command.
2. Preserved drive-absolute paths, UNC paths, and Linux absolute paths.
3. Kept missing-directory failures visible after normalization instead of
   masking an invalid target.
4. Added Help and README guidance plus sidecar and packaged-sidecar regression
   coverage.

Validation:

- `npm run sidecar:test`: 100 passed, with one upstream Starlette deprecation
  warning.
- `npm run sidecar:build`: passed and rebuilt the packaged Windows sidecar.
- Packaged-sidecar navigation regression: passed.
- `cargo test --lib --quiet`: 164 passed.
- Rust renderer tests verify literal argument escaping for PowerShell and WSL.
- A packaged-sidecar integration test verifies that
  `run trim_generic_residence.py` resolves through the same HTTP and adapter
  path used by the desktop app and never falls back to a literal `run`
  command.
- The Windows sidecar binary was rebuilt after the matcher change.

## Phase 2 slice - Grouped PowerShell multiline input - 11/08/2026

Extended exact-command paste handling for valid PowerShell expressions that
span multiple lines without explicit backtick continuations.

Completed:

1. Added balanced parentheses, brackets, and braces detection to the frontend
   paste flow and Rust execution boundary.
2. Accepted one grouped PowerShell expression such as
   `[Environment]::SetEnvironmentVariable(...)` while retaining its standard
   safety classification and approval flow.
3. Continued rejecting separate commands divided by bare newlines after the
   grouped expression closes.
4. Added clear errors for incomplete quotes and mismatched or unfinished
   delimiters.
5. Updated in-app Help and README guidance.

Validation:

- Rust tests cover the environment-variable example, separate-command
  rejection, non-PowerShell rejection, and malformed delimiter handling.
- `cargo test --lib`: 149 passed against the rebuilt packaged sidecar.
- `npm run build`: TypeScript and the Vite production build passed.

## Phase 2 slice - Cohesive PowerShell foreach scripts - 19/08/2026

Extended exact-command input for practical PowerShell validation scripts that
prepare data in variables and then inspect it in one `foreach` block.

Completed:

1. Recognized a top-level variable-assignment prelude followed by one complete
   `foreach` block as a single reviewed PowerShell script.
2. Wrapped the script in one invocation and preserved source newlines so
   pipelines and `[PSCustomObject]@{ ... }` hashtables remain valid.
3. Applied identical recognition in the frontend paste flow and Rust command
   boundary.
4. Kept unrelated commands after the loop blocked by the existing one-command
   rule.
5. Added regression coverage based on a manifest hash-comparison workflow.

Validation:

- Focused cohesive-script regression: passed.
- Commands-after-loop rejection regression: passed.
- `npm run build`: TypeScript and the Vite production build passed.

## Phase 2 slice - Sequential multiline command queue - 19/08/2026

Extended multiline paste handling for independent commands that should run in
order without weakening TerminalMate's command-by-command safety boundary.

Completed:

1. Split independent nonblank command lines into an ordered frontend queue.
2. Kept Bash and PowerShell continuation syntax, grouped expressions, and
   cohesive PowerShell `foreach` scripts as single logical commands.
3. Applied command resolution, environment checks, risk classification, and
   approval independently to every queued entry.
4. Advanced only after the matching command run succeeds and stopped the queue
   after a block, cancellation, user stop, or failed exit code.
5. Added a compact queue preview with progress, command ordering, and a clear
   action for pending entries.
6. Updated Help, README, and architecture documentation.

Validation:

- `npm run build`: TypeScript and the Vite production build passed.
- `npm run sidecar:test`: 91 passed, with one upstream Starlette deprecation
  warning.

## Phase 2 slice - Guided external file copy and cut - 21/08/2026

1. Added plain-English copy/cut flows supporting either `file <name> from
<folder>` or a complete source file path.
2. Destination paths are normalized relative to the active workspace, while
   explicit external source paths remain available for importing a file.
3. Generated PowerShell and WSL commands verify source/destination types and
   refuse silent overwrites before copying or moving.
4. Added sidecar, adapter, policy, packaged-sidecar, Help, and README coverage.

## Phase 2 slice - Copy complete current log - 21/08/2026

1. Added a Copy log action beside Open log in the completed-run footer.
2. Read the complete TerminalMate-owned command log only when requested, so
   the terminal can retain its 1,000-line display limit without copying a
   truncated transcript.
3. Added loading, copied, and readable failure feedback for clipboard actions.
4. Added Rust regression coverage for a complete log containing a
   50,000-character single output line.

Validation:

- `npm run sidecar:test`: 103 passed, with one upstream Starlette deprecation
  warning.
- `cargo test --lib --quiet`: 168 passed.
- `npm run build`: TypeScript and the Vite production build passed.

## Phase 2 slice - Help and documentation alignment - 21/08/2026

1. Added in-app guidance for persistent Recent runs, request reuse, the
   1,000-line visible transcript, and complete-output Open log / Copy log
   actions.
2. Confirmed Help examples cover workspace navigation, named file/folder
   inspection, guarded file create/delete/copy/cut, script arguments and batch
   ordering, Azure, Terraform, and SSH.
3. Documented the complete-log Tauri trust boundary and the workspace path and
   overwrite rules for file operations.
4. Aligned the README, architecture, safety model, product brief, roadmap, and
   current automated-test totals with the implemented `v0.2.1` behavior.

## Phase 2 slice - Guarded backup and replacement workflows - 21/08/2026

1. Added standalone natural-language intents for backing up a workspace file
   and explicitly replacing an existing workspace file.
2. Added one combined backup-then-replace workflow that validates all paths,
   creates the backup first, and replaces the original only after the backup
   succeeds.
3. Made `as` and `with` optional when path order is unambiguous, including
   support for quoted paths containing spaces.
4. Kept ordinary copy non-overwriting, kept backup at normal approval, and
   classified explicit replacement as High Risk with typed `RUN` confirmation.
5. Added focused sidecar parser tests plus PowerShell, WSL, and policy tests for
   the new workflow.

Validation:

- `npm run sidecar:test`: 111 passed, with one upstream Starlette deprecation
  warning.
- `cargo test --lib --quiet`: 171 passed.
- `npm run build`: TypeScript and the Vite production build passed.

## Phase 2 slice - Explicit sequential command batches - 21/08/2026

1. Confirmed that independent pasted lines remain separate queue entries and
   execute strictly one at a time.
2. Added compile-then-run coverage for two-command `uv` workflows and a
   four-command batch regression.
3. Preserved Bash and PowerShell continuation syntax plus grouped PowerShell
   scripts as single logical commands.
4. Clarified queue progress, Help, and README wording around one-at-a-time
   execution and stop-on-failure behavior.

Validation:

- `npm run command-parser:test`: 5 sequential parsing scenarios passed.
- `npm run build`: TypeScript and the Vite production build passed.

## Phase 2 slice - UI-only output timestamps - 21/08/2026

1. Added a local timestamp to each output line displayed in the terminal UI.
2. Standardized the display as `dd/mm/yyyy HH:mm:ss` using 24-hour time.
3. Kept timestamps out of persisted command log files and copied raw logs.
4. Added focused formatter coverage for morning and afternoon timestamps.

Validation:

- `npm run terminal-timestamp:test`
- `npm run build`

## Phase 2 slice - Readable command transcript boundaries - 25/08/2026

1. Added a restrained separator between the visible output of consecutive
   command runs.
2. Removed whitespace-only rows from the terminal UI so timestamps no longer
   appear on otherwise blank lines.
3. Carried each command run ID with streamed output so dividers follow actual
   command boundaries rather than timing guesses.
4. Kept persisted command logs and complete-output Open log / Copy log actions
   unchanged, including their original blank lines.
5. Added focused transcript preparation coverage for blank rows, same-run
   output, changed-run separators, and session startup output.

Validation:

- `npm run terminal-transcript:test`
- `npm run terminal-timestamp:test`
- `npm run build`
- `cargo test --lib --quiet`

## Phase 2 slice - Batch-level sequential approvals - 25/08/2026

1. Paused newly pasted multi-command queues before the first command and added
   an explicit approval-mode choice.
2. Kept `Review each` as the keyboard-focused default for the existing
   per-command approval behavior.
3. Added an opt-in that automatically approves only Caution commands in the
   current pasted batch while still classifying every command independently.
4. Preserved typed `RUN` confirmation for High Risk commands, blocked-command
   refusal, strict one-at-a-time execution, and stop-on-failure behavior.
5. Reset the batch approval scope when the queue is cleared or a new batch is
   pasted, preventing approval from leaking into later commands.

Validation:

- `npm run command-parser:test`: 8 parser and batch-approval policy scenarios
  passed.
- `npm run build`: TypeScript and the Vite production build passed.

## Phase 2 slice - Hidden-item detail listing - 25/08/2026

1. Added a dedicated plain-English intent for listing hidden items with their
   names and filesystem modes.
2. Prevented `Show hidden files with name and mode` from falling through to
   the generic file-content reader.
3. Rendered the request as `Get-ChildItem -Force | Select-Object Name, Mode`
   for PowerShell and `ls -la` for WSL Bash.
4. Added both supported phrasings to Everyday Help and README examples.

Validation:

- Focused sidecar intent tests: 3 passed.
- Focused Rust shell-adapter test: 1 passed.

## Phase 2 slice - Comma-delimited file and folder creation - 25/08/2026

1. Added plain-English plural creation for commands such as `Create folders
components, api, services` and `Create files src/index.ts, src/app.ts,
README.md`.
2. Normalized every target as workspace-relative and rejected absolute paths
   or parent traversal before rendering a command.
3. Removed duplicate paths while preserving the user's first-seen order.
4. Added PowerShell and WSL Bash renderers that preflight the complete target
   list before creating anything, preventing partially completed batches when
   a path already exists.
5. Kept the complete batch behind one Caution approval and documented the new
   syntax in Help and README examples.

Validation:

- Full sidecar suite: 115 passed, with one upstream Starlette deprecation
  warning.
- Full Rust library suite: 174 passed.
- TypeScript and Vite production build passed.

## Phase 2 slice - Visible command transcript headers - 25/08/2026

1. Added a timestamped command-start row before each command's visible output.
2. Styled command rows in the terminal accent color so commands are distinct
   from stdout, stderr, and informational output.
3. Emitted the command-start marker before either output pipe begins reading,
   preserving the correct order for fast commands and sequential queues.
4. Kept command headers UI-only so persisted logs, Open log, and Copy log
   continue to expose the original raw process output.
5. Preserved separators between runs and whitespace-only line suppression.

Validation:

- `npm run terminal-transcript:test`: 7 scenarios passed.
- `npm run build`: TypeScript and the Vite production build passed.
- `cargo test --lib`: 174 passed.

## Phase 2 fix - PowerShell pipeline continuation - 25/08/2026

1. Recognized a trailing PowerShell `|` as an explicit multiline continuation.
2. Joined formatted pipelines into one command before planning and execution.
3. Added frontend and Rust regression coverage for a three-stage
   `Get-ChildItem | Select-String | Select-Object` pipeline.

Validation: command parser tests, focused Rust normalization test, and the
production frontend build passed.

## Phase 2 reliability and documentation alignment - 26/08/2026

1. Expanded deterministic port inspection language to include `Find port`,
   `Locate port`, `Show port`, `Find process using port`, and `Which process
owns port`, with native PowerShell and WSL Bash rendering.
2. Added permission-tolerant recursive PowerShell discovery and guided retry
   behavior so inaccessible cache/system folders do not erase valid search
   results.
3. Confirmed the Windows foreground execution transport uses piped stdout and
   stderr rather than a console handle or ConPTY, with byte-line reading and no
   TerminalMate per-line cap.
4. Added UTF-8/unbuffered Python child output, Windows-byte fallback decoding,
   and normal-path regression coverage for a single logging line longer than
   50,000 characters.
5. Extended grouped PowerShell paste detection to keep variable assignments
   and range pipelines such as `100..115 | ForEach-Object { ... }` in one
   process, preserving variable scope. The Rust direct-command boundary applies
   the same rule, including scripts such as `$lines = Get-Content ...` followed
   by `80..180 | ForEach-Object { ... }`.
6. Updated in-app Help, README, Architecture, Safety Model, Product Brief, and
   Roadmap with the supported behavior and current validation counts.

Validation:

- Sequential command parser and approval policy: 10 scenarios passed.
- Frontend TypeScript and Vite production build passed.
- Full sidecar suite: 122 passed, with one upstream Starlette deprecation
  warning.
- Full Rust library suite: 176 passed.
## Phase 2 slice - Hybrid AI intent planning and durable directory history - 31/08/2026

1. Added an optional local-first generative planner: deterministic matching
   remains first, and only unmatched plain-English requests reach the configured
   OpenAI-compatible provider.
2. Restricted AI output to the existing structured action catalogue and typed
   parameters. Models cannot provide executable shell text; Rust still renders,
   classifies, previews, approves, and executes every command.
3. Added explicit `LOCAL MATCH` and `AI PLAN` provenance plus a visible top-bar
   **AI planner: On/Off** control and Settings form.
4. Kept the API key in session-scoped Rust memory only. It is not returned to
   React, written to localStorage, logged, or saved in project files.
5. Persisted directory Back/Forward stacks independently per workspace and
   runtime, preserving forward history when returning to a known root and
   replacing it only after branching to a new directory.
6. Tightened provider endpoint validation to HTTPS or exact HTTP loopback hosts.

Validation:

- `npm run build`: passed.
- `npm run sidecar:test`: 130 passed, with one upstream FastAPI/httpx
  deprecation warning.
- `cargo check`: passed.
- `cargo test --lib --quiet`: 191 passed.
