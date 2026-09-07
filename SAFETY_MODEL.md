# TerminalMate Safety Model

Status: Phase 2 in progress - the Phase 0 contract below still holds; this
revision adds two mechanisms implemented since: typed `RUN` confirmation for
executable High Risk operations, and Explain Only, a per-intent override that
sits outside the four risk levels
Date: 25/07/2026, updated 03/08/2026

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
- list containers, Kubernetes resources, or Helm releases (`docker ps`,
  `kubectl get`, `helm list`);
- preview or validate Terraform without changing infrastructure
  (`terraform plan`/`fmt`/`validate`/`output`, `terraform state list`/`show`);
- read cloud account or resource state (`az account show`, `az vm list`,
  `aws sts get-caller-identity`, `gcloud config list`);
- read GitHub repository, PR, or CI run state (`gh repo view`, `gh pr list`);
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
- create, build, or run a container, or run a Compose stack
  (`docker run`/`build`/`compose up`);
- change Kubernetes cluster state or open a local tunnel to it
  (`kubectl apply`, `kubectl port-forward`);
- install or upgrade a Helm release, or download Terraform providers/modules
  (`terraform init`);
- authenticate or configure a cloud provider or GitHub CLI (`az login`,
  `gh auth login`), or change an Azure VM's running state (`az vm start`/`stop`);
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
- apply infrastructure changes to a shared or production environment
  (`terraform apply`/`destroy`, creating/deleting an Azure resource group,
  deallocating an Azure VM);
- remove containers, images, volumes, Kubernetes resources, or Helm releases
  (`docker rm`/`rmi`, `docker volume rm`, `docker system prune`,
  `kubectl delete`, `helm uninstall`);
- force Git operations;
- terminate critical processes;
- modify databases or persistent storage.

Required behaviour:

- identify the exact resource and environment;
- show the full impact and dependencies;
- require step-by-step approval;
- require stronger confirmation for production or irreversible operations —
  implemented today as a typed `RUN` confirmation: a single click is not
  enough for an executable High Risk command, the user must type the literal
  word `RUN` before it executes;
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
- disabling TerminalMate's own approval or audit controls;
- a directly-typed `terraform apply`/`destroy` with no `-auto-approve` and no
  piped input — not a trust judgment like the others above, but a mechanical
  certainty: TerminalMate runs commands with no stdin channel, so Terraform's
  own interactive confirmation prompt could never be answered and the
  command would simply hang. A pipe (`echo yes | terraform apply`) or
  `-auto-approve` lifts the block, since either removes the unanswerable
  prompt;
- full-screen interactive editors and monitors (`nano`, `vim`/`vi`, `emacs`,
  `pico`, `top`, `htop`) — also a mechanical certainty rather than a trust
  judgment: TerminalMate has no PTY, so there is no cursor addressing, no
  screen redraw, and no way to send keystrokes. These programs cannot
  function at all through TerminalMate's execution model, not merely behave
  oddly. When a file path is confidently extracted from the blocked command
  (`nano main.tf`), TerminalMate offers its own built-in editor as a working
  alternative instead — a direct file read/write with no shell or terminal
  involved, safer than trying to emulate a terminal, not a relaxation of the
  block.

Blocked operations remain visible with a plain-English reason. There is no
silent override.

## Explain Only

Explain Only is not a fifth risk level, and it does not replace the four
levels above — every command still gets a Safe/Caution/High
Risk/Blocked classification regardless. It is a narrower, additional
safeguard that applies only to a small, explicitly named set of plain-English
intents whose consequence (irreversible infrastructure loss, or exposing
credential material) is severe enough that TerminalMate will not offer to run
them at all from an AI-interpreted request, no matter how the policy engine
would otherwise classify the rendered command.

Today's Explain Only intents:

- retrieving Azure storage-account keys (can reveal credentials);
- deleting an Azure resource group (removes every resource inside it).

For these, TerminalMate shows the exact rendered command and a plain-English
explanation of why it will not run it, with a copy-to-clipboard action in
place of an approval control — the user reviews and runs it manually in a
controlled terminal.

`terraform apply` and `terraform destroy` were Explain Only in an earlier
revision of this model, for the reason above (irreversible infrastructure
risk). They were moved to the ordinary High Risk / typed-`RUN` path once a
more specific problem was identified: Terraform's own interactive "type yes
to confirm" prompt cannot be answered by TerminalMate at all (commands run
with no stdin channel), so even an approved run would hang forever after
printing its plan. The adapter now renders both with `-auto-approve`, which
removes Terraform's own confirmation step entirely — TerminalMate's typed-`RUN`
confirmation is the only approval gate left standing in its place, not an
addition to Terraform's.

Explain Only is deliberately narrow in scope: it applies only when the
command came from interpreting a plain-English request. The same operation
typed directly by the user still goes through ordinary risk classification
and approval like any other command — a literally-typed `az group delete` is
High Risk with a typed-`RUN` gate, not Explain Only. The distinction Explain
Only protects against is trusting an AI interpretation of intent with the
most destructive infrastructure actions, not the command family in general;
a user who types the exact command themselves has already demonstrated they
know precisely what they intend to run.

A literally-typed `terraform destroy` (without `-auto-approve`) is a
different case, and Blocked rather than High Risk — not because of Explain
Only, but because it would hang unanswerably at Terraform's own prompt (see
the Blocked tier above). Add `-auto-approve` or pipe a confirmation to run
it directly; it is then High Risk with the usual typed-`RUN` gate.

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

Implemented today as a per-workspace run log (log file plus the visible
transcript and a durable "Recent runs" list, not yet the queryable structured
receipt described above — see Architecture's Local Data section). The
approval step itself is a single click for Caution, and a typed `RUN`
confirmation for High Risk. Explain Only intents are shown with the same
command-and-explanation preview but never reach an approval step at all —
their only available action is copying the command.

Keyboard focus follows the same proportional boundary. A Caution preview
focuses **Cancel**, so Tab then Enter deliberately reaches **Approve and run**.
A High Risk preview focuses the typed confirmation field and still requires
the literal word `RUN`; opening a preview never silently activates execution.

The visible transcript is bounded to the latest 1,000 lines for performance,
but the complete TerminalMate-owned log remains available through **Open log**
and the user-triggered **Copy log** action. Full-log reads are accepted only
after Rust canonicalizes the requested path and confirms that it belongs to
TerminalMate's application log directory; an arbitrary filesystem path cannot
be supplied through this command.

## Command Construction Rules

- Prefer argument arrays and native APIs over shell string composition.
- Quote paths and values through the selected adapter.
- Reject untrusted control operators unless the intent explicitly supports a
  reviewed pipeline.
- Resolve registered workspace paths before rendering.
- Reject traversal outside approved scope when an operation is workspace-bound.
- For guided file creation and deletion, require the target to remain inside
  the active workspace; deletion accepts one file only and refuses folders.
- For guided copy and cut, permit an explicit external source file but require
  the destination to be a folder inside the active workspace, verify both
  types, and refuse silent overwrite. A cut removes the source only after the
  move succeeds.
- Backup accepts an existing workspace file and a new workspace-relative file
  path. It refuses an existing backup path and remains a normal approval action.
- Replace accepts an existing workspace target and one explicit source file.
  It is classified High Risk and requires typed `RUN` confirmation because it
  deliberately overwrites the target.
- Backup-then-replace validates the target, replacement source, backup parent,
  and non-existing backup path before writing. It creates the backup first and
  replaces the original only if that backup copy succeeds.
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

For a read-only recursive PowerShell search that fails only because one folder
is inaccessible, recovery may offer a retry with
`-ErrorAction SilentlyContinue`. This changes error handling, not the requested
scope or risk level: readable matches are returned and inaccessible paths are
skipped. It is not applied as a generic way to suppress errors from mutating
commands.

TerminalMate must not repeatedly retry a mutating command without new approval.

## Initial Safety Boundary

Phase 1 supported automatic execution only for a small allowlist of
navigation and inspection intents in registered Windows workspaces. All
mutation, privilege, remote execution, script execution, and unsupported
intent types required approval or remained blocked.

Phase 2 extended this boundary without loosening it: WSL Bash workspaces get
the identical Safe/Caution/High Risk/Blocked classification and Explain Only
treatment as native Windows workspaces, and the newly added Azure CLI and
Terraform intents were classified into the existing four levels (plus Explain
Only where appropriate) rather than introducing a separate, weaker path for
infrastructure commands.

The allowlist expands only with adapter tests, policy tests, and documented
recovery behaviour.
