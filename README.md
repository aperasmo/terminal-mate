# TerminalMate

TerminalMate is an organized, safety-first terminal workspace for developers,
students, and technical users who know what they want to accomplish but may not
know the correct commands.

The application keeps projects and persistent terminal sessions together, turns
plain-English requests into structured intents, selects commands for the active
environment, and applies proportional safety controls before execution.

## Current Status

Phase 1 foundation, version `0.1.0`.

The initial scaffold includes:

- Tauri 2 desktop shell;
- React and TypeScript workspace interface;
- native folder selection and execution-profile detection;
- locally persisted workspace registration;
- Python 3.12 FastAPI sidecar;
- authenticated health and structured-intent endpoints;
- initial deterministic intents for directory navigation and file inspection;
- sidecar and Rust workspace-path tests;
- generated application icons for Windows, Linux, and future macOS packaging;
- Windows NSIS preview installer.

Persistent terminal processes and command execution are intentionally not
enabled in this scaffold. They are the next Phase 1 implementation slice and
will be added with the Rust policy boundary.

## UI Preview Installer

Build the Windows installer from the project root:

```text
npm run installer:windows
```

Output:

```text
src-tauri/target/release/bundle/nsis/TerminalMate_0.1.0_x64-setup.exe
```

This installer is labeled **UI Preview** in the application header. It is
intended for reviewing:

- the three-column workspace layout;
- project registration and selection;
- environment and safety context;
- terminal-session placeholders;
- command-bar positioning, wording, and visual hierarchy;
- behavior at different window sizes.

Live shell sessions, AI planning, and command execution are not enabled in this
preview.

## Product Documents

- [Product brief](PRODUCT_BRIEF.md)
- [Architecture](ARCHITECTURE.md)
- [Safety model](SAFETY_MODEL.md)
- [Roadmap](ROADMAP.md)
- [Project status](PROJECT_STATUS.md)

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
