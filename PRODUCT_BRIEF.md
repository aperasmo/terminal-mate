# TerminalMate Product Brief

Status: Phase 0 - Product definition  
Date: 25/07/2026

## Product Summary

TerminalMate is an organized, safety-first terminal workspace. It keeps
projects, terminal sessions, WSL environments, remote hosts, and containers in
one application, then turns plain-English requests into environment-aware
command plans.

The user keeps control: TerminalMate explains what it intends to do, applies
deterministic safety checks, and requires the appropriate level of approval
before any mutating or high-risk action runs.

Working tagline:

> Your safety-first command companion.

## Problem

Terminal work often spreads across many windows, shells, directories, and
environments. Users may understand what they want to accomplish without knowing
the exact command or the differences between operating systems and shells.

The complexity increases for DevOps work involving servers, containers, cloud
contexts, infrastructure tools, and shared environments.

Existing AI terminal products commonly introduce one or more of these
trade-offs:

- replace the user's preferred terminal;
- focus on coding agents instead of operational work;
- generate free-form commands without a strong safety boundary;
- require a hosted account or one AI provider;
- provide little guidance after a command fails;
- hide important environment and command-impact details.

TerminalMate addresses these gaps by organizing the work first, then adding AI
planning, deterministic command translation, approval controls, and guided
recovery.

## Initial Audience

The initial audience is developers, students, and technical users who know what
they want to accomplish but may not know the correct commands.

Initial user profiles:

- developers moving between projects, shells, and toolchains;
- students learning terminal concepts and command-line workflows;
- technical users who need clear commands without memorizing shell syntax;
- developers performing occasional operational or DevOps tasks.

TerminalMate must also remain useful as users advance. DevOps usability is an
explicit product consideration, including persistent environments, remote
systems, containers, CI/CD tools, cloud infrastructure, and stronger safety
controls for shared or production targets.

## Product Position

TerminalMate is:

- a desktop terminal workspace;
- a manager for persistent terminal sessions;
- a plain-English command planner;
- a deterministic environment-specific command translator;
- a safety and approval layer;
- an error explanation and recovery assistant;
- a local-first tool with user-selected AI providers.

TerminalMate is not:

- a general coding agent;
- an autonomous infrastructure operator;
- a replacement for every existing terminal;
- a system that sends raw AI output directly to a shell;
- a reason to hide the actual command from the user.

## Product Principles

1. **One organized workspace**
   Keep related projects, shells, sessions, and environments together.

2. **Intent before syntax**
   Understand the user's goal before choosing an operating-system-specific
   command.

3. **Environment-aware execution**
   Generate commands for the active runtime, shell, distribution, directory,
   privileges, and available tools.

4. **Deterministic safety boundary**
   AI proposes structured actions. Trusted application code validates,
   translates, and executes them.

5. **Proportional approval**
   Read-only navigation should feel fast. Mutations and destructive operations
   require increasingly explicit review.

6. **Visible operations**
   Show what ran, where it ran, its result, and what context was sent to an AI
   provider.

7. **Recovery is part of the workflow**
   When a command fails, explain the cause and offer reviewed next steps inside
   TerminalMate.

8. **Local-first and provider-flexible**
   Store workspace state locally and support both local and user-configured
   cloud AI providers.

## Primary Experience

The application uses four stable areas:

- **Left panel:** workspaces, WSL distributions, remote hosts, containers, and
  saved environments.
- **Centre:** persistent terminal sessions shown as tabs or split panes.
- **Right panel:** active environment context, command risk, AI explanation,
  running activity, and recovery guidance.
- **Bottom input:** plain-English requests and direct command entry.

Example:

1. The user selects the `terminal-mate` workspace.
2. The active execution profile is Windows host, native Windows runtime,
   PowerShell shell.
3. The user enters: `Take me to the terminal-mate directory.`
4. TerminalMate resolves the registered workspace, changes the persistent
   session directory, and confirms the location.
5. The user enters: `Show me all the .txt files.`
6. TerminalMate creates a read-only file-search intent and executes the
   PowerShell-equivalent listing command in the same session.

## Phase 1 MVP

The first usable release will include:

- one-window workspace and session organization;
- persistent native Windows PowerShell sessions on a Windows host;
- registered project directories with native Windows path resolution;
- terminal tabs with preserved current directory and environment;
- plain-English navigation and read-only file inspection;
- structured intent creation and validation;
- Safe, Caution, High Risk, and Blocked classifications;
- command preview, explanation, approval, execution, and cancellation;
- streamed output and local command history;
- basic failure explanation and recommended recovery;
- user-configured AI provider with a visible privacy receipt.

Initial supported intents:

- change directory, including common aliases such as the home directory;
- show current directory;
- list files and directories;
- find files by name or extension;
- view a line range of a file;
- search text in files;
- inspect file metadata;
- inspect processes and listening ports;
- show environment and installed-tool information.

## Phase 1 Non-Goals

The first release will not include:

- autonomous multi-step infrastructure changes;
- production Kubernetes or Terraform mutation;
- remote SSH session management;
- a plugin ecosystem;
- full shell-script generation and execution;
- macOS packaging;
- team-hosted accounts or shared cloud storage;
- unrestricted free-form command execution from AI output.

## Success Criteria

Phase 1 is successful when a user can:

- keep multiple project sessions organized in one window;
- use plain English to navigate and inspect a WSL workspace;
- see the exact environment and command before meaningful changes;
- distinguish safe inspection from mutating and destructive operations;
- recover from common path, tool, permission, and command failures;
- complete the workflow without installing Python or Node separately.
