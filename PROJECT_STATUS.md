# TerminalMate Project Status

Last updated: 25/07/2026

## Current Release

- Phase: 1 - Windows Workspace MVP
- Version: `0.1.0`
- Status: Persistent terminal session runtime added; command adapter and
  policy engine next

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
- Not yet manually click-tested end-to-end in the live desktop app in this
  session; recommend a manual pass (`npm run tauri:dev` -> add a workspace
  -> start a session -> try a Safe command, a Caution command, and a
  Blocked command) before relying on this slice.

Next:

1. Add the trusted adapter that renders the already NLP-matched Safe
   intents (change directory, list/find files by extension, view a line
   range of a file) into the same classify/execute path.
2. Connect the plain-English command bar to the sidecar intent endpoint,
   so a typed request first tries NLP matching and falls back to direct
   command classification.
3. Add approval receipts (what ran, when, its result) and cancellation.
4. Add basic recovery guidance for failed commands.
