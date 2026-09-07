# TerminalMate Roadmap

Status: Phase 2 in progress - WSL / Linux Subsystem Support and infrastructure
intents (see [Project status](TERMINAL-MATE-PROJECT_STATUS.md) for the current
slice)
Started: 25/07/2026

Version numbers align with product phases.

## Phase 0 - Product Blueprint - 25/07/2026

Goal: define the product boundary before application scaffolding.

Deliverables:

- product brief;
- architecture and process boundaries;
- structured intent and adapter approach;
- safety levels and approval contract;
- phased implementation roadmap.

Exit criteria:

- developers, students, and technical users are confirmed as the initial
  audience, with DevOps usability treated as an explicit advanced use case;
- TerminalMate is defined as an organized desktop terminal workspace;
- native Windows is the initial execution environment;
- AI is separated from deterministic command execution;
- Phase 1 scope and non-goals are explicit.

## Phase 1 - Windows Workspace MVP - v0.1.0

Goal: organize persistent native Windows PowerShell sessions and safely
handle basic plain-English navigation and inspection.

Planned capabilities:

- Tauri, React, Rust, and packaged Python sidecar foundation;
- one-window workspace layout;
- workspace registration and aliases;
- persistent Windows PowerShell terminal tabs;
- execution-profile detection;
- native Windows path resolution, including common aliases such as the home
  directory;
- structured intent schema;
- Windows PowerShell adapter;
- Safe, Caution, High Risk, and Blocked policy decisions;
- navigation, listing/finding files by name or extension, viewing a line
  range of a file, text search, process inspection, and port inspection;
- command preview, execution, streaming output, live elapsed time, Stop, and
  durable workspace-specific history with final command duration;
- AI-provider settings and privacy receipts;
- initial diagnostics and recovery guidance;
- Windows installer for the native Windows experience.

Scope boundary:

- common local developer commands are included in Phase 1;
- Git-specific guided workflows are deferred to avoid duplicating AI Git
  Assistant before TerminalMate's terminal and DevOps strengths are mature;
- SSH and remote execution remain Phase 4.

Exit criteria:

- `Take me to terminal-mate` changes the active persistent Windows session to
  the correct registered path;
- `Take me to the home directory` resolves the home-directory alias correctly;
- `Show me all the .txt files` runs a safe PowerShell-equivalent listing;
- `Show me lines 20-100 of sample.txt` runs a safe, bounded file-range view;
- unsafe or unsupported requests cannot bypass review;
- terminal state persists across multiple requests;
- the packaged application works without a separate Python or Node install.

## Phase 2 - WSL / Linux Subsystem Support - v0.2.0

Goal: add Ubuntu/WSL Bash operation alongside native Windows.

Planned capabilities:

- Ubuntu/WSL execution profile and Bash adapter;
- shell and tool capability detection;
- equivalent intent rendering across PowerShell and Bash;
- clear active-environment selector;
- path mapping between Windows and WSL (reusing the existing pure
  path-conversion helper);
- cross-platform adapter golden tests;
- environment-specific recovery guidance.

Completed on 01/08/2026:

- persisted Windows PowerShell / WSL Bash selection per workspace;
- real WSL Bash sessions and foreground command execution;
- Windows-to-WSL workspace path mapping;
- Bash rendering for the existing deterministic intent catalogue;
- shared safety classification across PowerShell and Bash.

Completed prioritized slice on 01/08/2026 (`v0.2.1`):

- deterministic Azure CLI and Terraform intents from the approved intent
  specification;
- typed command parameters and prerequisite checks instead of free-form shell
  generation;
- Safe, Caution, High Risk, Blocked, and Explain Only decisions appropriate to
  infrastructure operations;
- explain-only handling for Azure storage-key retrieval and resource-group
  deletion;
- typed-`RUN` High Risk execution for Terraform apply/destroy, rendered with
  `-auto-approve` because TerminalMate has no interactive stdin channel.

Completed Phase 2 usability slices through 26/08/2026:

- persistent per-workspace Windows/WSL runtime selection and path mapping;
- latest-10 reusable run history with elapsed time and restart persistence;
- responsive 1,000-line transcripts backed by complete on-disk logs, including
  one-click complete-log copying;
- script execution with arguments, sequential batch ordering, recursive
  discovery, live unbuffered Python output, and stop-on-failure behavior;
- exact multiline commands, cohesive PowerShell `foreach` and
  `ForEach-Object` blocks, and sequential pasted command queues with
  independent safety decisions;
- workspace-relative navigation and guarded file create, delete, copy, and cut
  workflows;
- deterministic recovery guidance, including guided Azure subscription
  diagnosis and retry;
- permission-tolerant recursive PowerShell searches and expanded process/port
  ownership requests;
- byte-safe piped Windows output with long-line regression coverage.

Completed on 31/08/2026:

- optional local-first generative AI intent fallback with a strict action
  allowlist and typed parameter validation;
- trusted Rust rendering and unchanged safety/approval enforcement for AI
  plans, with visible source provenance;
- session-only OpenAI-compatible provider configuration and a visible top-bar
  planner status control;
- persistent per-workspace, per-runtime directory Back/Forward history.

Remaining Phase 2 work:

- expanded PowerShell/Bash golden test matrix;
- environment-specific recovery guidance;
- broader WSL capability detection beyond the initial availability check.

## Phase 3 - Developer and DevOps Toolkit - v0.3.0

Goal: support common local development and DevOps inspection operations.

Discovery checkpoint:

- remind Allan before Phase 3 implementation begins;
- collect the common commands and recurring workflows used by his DevOps
  contact;
- record the environment, purpose, frequency, expected output, common failure,
  and operational risk for each workflow;
- use this evidence to prioritize intents, adapters, safety rules, and recovery
  guidance.

Planned capabilities:

- Docker process, image, container, network, and volume inspection;
- service and process diagnostics;
- port ownership and connectivity checks;
- structured log search and bounded output;
- package-manager detection and reviewed installation plans;
- Node, Python environment, and toolchain diagnostics;
- reusable safe inspection workflows.

Mutation remains approval-gated and destructive container operations remain
High Risk.

## Phase 4 - Remote Operations - v0.4.0

Goal: organize and operate reviewed SSH sessions.

Planned capabilities:

- SSH host profiles using OS credential storage;
- persistent remote terminal sessions;
- remote execution profiles and path aliases;
- host-key and identity visibility;
- clear local-versus-remote target indicators;
- minimum-context AI error analysis;
- remote policy rules and stronger approval defaults.

## Phase 5 - Infrastructure Context - v0.5.0

Goal: add safety-aware Docker, Kubernetes, Terraform, and cloud CLI workflows.

Planned capabilities:

- Kubernetes context, namespace, and resource visibility;
- read-only Kubernetes inspection;
- Terraform initialization, validation, and plan review;
- explicit plan-to-apply approval boundary;
- cloud account, project, subscription, and region visibility;
- production-environment protection policies;
- resource-impact summaries.

## Phase 6 - Workflows and Team Policy - v0.6.0

Goal: make safe operational practices reusable across a team.

Planned capabilities:

- versioned workflow recipes;
- repository-local TerminalMate context;
- organization policy files without secrets;
- environment classifications;
- team command conventions;
- exportable approval and execution receipts;
- workflow validation and dry runs.

## Phase 7 - Native Linux Release - v0.7.0

Goal: package and validate TerminalMate on native Linux while sharing the same
intent, adapter, and policy core.

Planned capabilities:

- native Linux execution profiles;
- Debian/Ubuntu package;
- native Linux session and path handling;
- installer and upgrade validation;
- Windows/WSL and Linux regression matrix.

## Phase 8 - Product Hardening - v0.8.0

Goal: prepare TerminalMate for broader public use.

Planned capabilities:

- accessibility and responsive workspace refinement;
- performance testing for many persistent sessions;
- crash and session restoration;
- expanded adversarial safety tests;
- signed release automation where available;
- contribution guide, security policy, and public issue templates.

## Parked: macOS Support

macOS remains an architectural requirement but is not assigned to a release
until a macOS build and signing environment is available.

Preparation retained in the design:

- OS-independent structured intents;
- dedicated Zsh/macOS adapter boundary;
- BSD-versus-GNU command tests;
- `launchctl` and Homebrew capability models;
- macOS path and credential-storage abstraction.
