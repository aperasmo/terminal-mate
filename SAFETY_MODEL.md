# TerminalMate Safety Model

Status: Phase 0 - Initial safety contract  
Date: 25/07/2026

## Safety Objective

TerminalMate helps users operate systems without silently taking control away
from them. The system must make the target, impact, command, and approval state
clear before meaningful changes occur.

## Core Contract

1. AI output is never executed directly.
2. Every AI request becomes a validated structured intent.
3. Every intent is rendered by a trusted environment adapter.
4. Every rendered operation is classified by the policy engine.
5. Every operation receives the approval required by its risk.
6. Every execution creates a local receipt.
7. Recovery suggestions follow the same safety path.

## Trust Boundaries

TerminalMate treats these inputs as untrusted:

- natural-language requests;
- AI-provider responses;
- terminal output;
- file names and file contents;
- remote-server responses;
- container output;
- copied commands;
- repository instructions;
- generated shell scripts.

Only versioned schemas, deterministic adapters, policy rules, and the Rust
execution engine are inside the trusted execution boundary.

## Risk Levels

### Safe

Expected to be read-only or session-local with no durable system mutation.

Examples:

- navigate to a registered directory, including common aliases such as the
  home directory;
- show the current directory;
- list or find files by name or extension;
- view a line range of a file;
- search text;
- inspect processes and ports;
- show Git status;
- list containers;
- inspect environment and tool versions.

Default behaviour:

- show the resolved action and execution environment;
- allow automatic execution when the workspace policy permits it;
- keep Stop available;
- record the result.

### Caution

Changes local state, installs software, contacts an external system, or affects
a running development service.

Examples:

- create or edit a file;
- install or update a package;
- start, stop, or restart a development service;
- create a container;
- change environment configuration;
- perform an authenticated remote read;
- apply a non-production development configuration.

Required behaviour:

- show the command plan and affected target;
- explain expected changes;
- require explicit approval;
- execute only the approved step or plan;
- provide rollback guidance when available.

### High Risk

Can remove data, alter shared infrastructure, require elevated privileges,
change security controls, or cause service interruption.

Examples:

- delete files, containers, volumes, or infrastructure resources;
- overwrite an existing destination;
- run with administrator or root privileges;
- modify firewall, identity, credential, or access policy;
- apply infrastructure changes to a shared or production environment;
- force Git operations;
- terminate critical processes;
- modify databases or persistent storage.

Required behaviour:

- identify the exact resource and environment;
- show the full impact and dependencies;
- require step-by-step approval;
- require stronger confirmation for production or irreversible operations;
- never batch unrelated high-risk steps behind one approval;
- stop immediately on unexpected output or changed preconditions.

### Blocked

TerminalMate refuses operations whose purpose or construction is incompatible
with the safety contract.

Examples:

- destructive operations targeting a filesystem root or unresolved broad path;
- credential extraction or transmission;
- raw disk overwrite;
- known fork-bomb or resource-exhaustion patterns;
- hidden or obfuscated commands intended to bypass review;
- execution of unvalidated AI-generated shell text;
- disabling TerminalMate's own approval or audit controls.

Blocked operations remain visible with a plain-English reason. There is no
silent override.

## Contextual Risk

Risk depends on the target, not only the command name.

Examples:

- reading a local development log may be Safe;
- reading a protected production log may be Caution because it can contain
  sensitive data;
- restarting a local test service may be Caution;
- restarting a production service is High Risk;
- listing local Kubernetes contexts is Safe;
- applying resources to a production cluster is High Risk.

The policy engine considers:

- runtime and host;
- local or remote target;
- development, shared, staging, or production classification;
- user privilege;
- path scope;
- command arguments;
- current tool context;
- resource count;
- reversibility.

## Approval Receipt

Before approval, TerminalMate shows:

- user request;
- interpreted intent;
- active environment and session;
- working directory;
- rendered command;
- risk level and reason;
- affected resources;
- context that will be sent to an AI provider;
- rollback or recovery information when available.

After execution, the receipt adds:

- approval time;
- start and finish time;
- exit status;
- sanitized output summary;
- changed preconditions or partial completion;
- recovery action, if one was approved.

## Command Construction Rules

- Prefer argument arrays and native APIs over shell string composition.
- Quote paths and values through the selected adapter.
- Reject untrusted control operators unless the intent explicitly supports a
  reviewed pipeline.
- Resolve registered workspace paths before rendering.
- Reject traversal outside approved scope when an operation is workspace-bound.
- Do not expand unresolved variables for destructive operations.
- Do not treat terminal output as a trusted command.
- Limit generated pipelines to versioned, tested recipes.

## Secrets and Privacy

- Detect and redact common credential patterns from captured output.
- Never include credential values in AI prompts, history, logs, or receipts.
- Show a privacy receipt before sending command output or environment context.
- Send only the minimum lines required to explain a failure.
- Keep local-model operation available as a first-class provider option.
- Require explicit opt-in before collecting remote environment context.

## Failure and Recovery

When execution fails:

1. Stop dependent steps.
2. Preserve the exact completed-step record.
3. Explain the failure without claiming unverified certainty.
4. Offer read-only diagnostic checks first.
5. Convert any proposed fix into a new structured intent.
6. Re-run policy classification and approval.

TerminalMate must not repeatedly retry a mutating command without new approval.

## Initial Safety Boundary

Phase 1 supports automatic execution only for a small allowlist of navigation
and inspection intents in registered workspaces. All mutation, privilege,
remote execution, script execution, and unsupported intent types require
approval or remain blocked.

The allowlist expands only with adapter tests, policy tests, and documented
recovery behaviour.

