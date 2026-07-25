# TerminalMate Roadmap

Status: Phase 0 - Approved direction  
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
- command preview, execution, streaming output, Stop, and local history;
- AI-provider settings and privacy receipts;
- initial diagnostics and recovery guidance;
- Windows installer for the native Windows experience.

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
