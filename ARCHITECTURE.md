# TerminalMate Architecture

Status: Phase 0 - Proposed architecture  
Date: 25/07/2026

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

TerminalMate will reuse the proven architecture of AI Git Assistant, with a
larger role for Rust:

- **Tauri:** desktop application shell and secure native bridge;
- **React and TypeScript:** workspace, terminal, approval, history, and
  settings interfaces;
- **Rust:** persistent terminal processes, environment detection, policy
  enforcement, command rendering, execution, cancellation, and output
  streaming;
- **Python and FastAPI sidecar:** natural-language interpretation, AI-provider
  adapters, explanations, and recovery suggestions;
- **SQLite:** local workspaces, session metadata, history, policies, and
  reusable workflows;
- **OS credential storage:** encrypted API keys, SSH references, and tokens;
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
understand the request.

### Intent Validator

Rejects:

- unknown action types;
- missing or invalid parameters;
- paths outside the allowed scope;
- unsupported platform and action combinations;
- malformed shell arguments;
- requests that attempt to bypass the policy engine.

### Adapter Registry

Translates validated intents into commands for the active execution profile.

Initial adapters:

- Windows with PowerShell.

Later adapters:

- Ubuntu/WSL with Bash, after the initial Windows-native slice;
- native Linux distributions;
- remote Bash through SSH;
- container shells;
- macOS with Zsh;
- tool-specific adapters for Docker, Kubernetes, Terraform, and cloud CLIs.

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
not rely only on keyword matching.

### Execution Engine

Runs only commands that have:

1. a valid intent;
2. a supported adapter result;
3. a completed policy decision;
4. the required user approval.

It provides:

- persistent session input and output;
- cancellation and timeout controls;
- exit status;
- bounded output capture;
- secret redaction;
- an immutable execution receipt.

### Recovery Assistant

Receives sanitized failure output, execution profile, intent, and exit status.
It returns:

- a plain-English cause;
- checks that can confirm the diagnosis;
- one or more structured recovery intents;
- a warning when human investigation is safer.

Recovery actions go through the same validation, policy, and approval path as
the original action.

## Execution Profile

The host OS alone does not determine command syntax. Every command uses the
active session profile:

```json
{
  "hostOs": "windows",
  "runtime": "local",
  "runtimeName": "Windows",
  "targetOs": "windows",
  "shell": "powershell",
  "workingDirectory": "D:\\APE\\Portfolio-Project\\terminal-mate",
  "architecture": "x86_64",
  "privilege": "standard-user",
  "capabilities": ["Get-ChildItem", "Select-String", "git", "python3"]
}
```

If the session is Windows + WSL + Bash instead, the Linux adapter is selected
even though the desktop application is hosted by Windows.

## Workspace Path Resolution

Workspace paths are stored with environment-specific mappings:

```text
Workspace alias: terminal-mate
Windows path: D:\APE\Portfolio-Project\terminal-mate
WSL path: /mnt/d/APE/Portfolio-Project/terminal-mate (populated once the WSL adapter ships)
Remote path: optional and host-specific
```

The workspace manager resolves aliases first. The selected adapter then uses
the path appropriate to the active runtime. The Windows-to-WSL mapping is
already implemented as a pure path-conversion helper and is reused as soon as
the WSL adapter is added; it is not on the active path for the Windows-native
adapter.

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

SQLite stores:

- workspace definitions;
- non-secret execution profiles;
- terminal-session restoration metadata;
- command intents and execution receipts;
- risk decisions and approvals;
- saved workflows;
- privacy receipts.

Secrets are stored through operating-system credential protection and are
referenced by opaque identifiers.

## Test Strategy

The architecture requires:

- schema tests for every intent;
- unit tests for every adapter and risk rule;
- golden tests mapping the same intent across PowerShell, Linux, and macOS
  fixtures;
- path-conversion tests for Windows and WSL;
- terminal-session lifecycle and cancellation tests;
- adversarial tests for injection, obfuscation, traversal, and secret leakage;
- integration tests using temporary workspaces;
- end-to-end tests for the Phase 1 natural-language workflows.

No platform adapter is considered supported until its golden and integration
tests pass on that platform.

