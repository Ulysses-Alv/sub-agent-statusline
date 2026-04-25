# OpenCode Subagent Monitor

Desktop companion app that displays the status of running subagents in OpenCode.

## Requirements

- Rust (latest stable)
- Node.js 18+
- pnpm

## Development

```bash
cd companion
pnpm install
pnpm dev        # Start dev server + Tauri dev
pnpm build      # Build for production
pnpm test       # Run tests
```

## How It Works

1. Enable the runtime plugin in OpenCode: `"opencode-subagent-statusline/runtime"`
2. Launch the companion app
3. It will discover OpenCode processes and monitor their subagents

## Tech Stack

- Tauri v2 (Rust backend)
- Solid.js (frontend)
- Vitest (tests)
