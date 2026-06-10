# Cluaiz App Independent System Design v3 (Archer)

## 1) Product Intent
Cluaiz is a unified AI app system for Desktop, Mobile, and Web with:
- Best-in-class chat UX
- Rust-native power and OS control
- Flexible architecture for unlimited future features
- Lightweight and fast runtime behavior

This document is the source of truth for system structure, UI behavior, module boundaries, and scale strategy.

## 2) Core Principles (Non-Negotiable)
- DRY first: no duplicated business logic.
- Contract-first: schema/interface before implementation.
- Feature isolation: each feature in its own module folder.
- Stable core: app-core orchestrates; features plug in.
- Native isolation: UI never calls OS APIs directly.
- Backward-safe evolution: all commands/events versioned.
- Performance by default: startup, render, stream budgets enforced.

## 3) Final Technology Choice
- Native shell: Tauri v2 (desktop + mobile)
- UI app: React + Vite + TypeScript
- State: Zustand + TanStack Query
- UI system: Tailwind + tokenized primitives
- Protocol validation: zod
- Native bridge: Rust commands/events + adapter plugins
- Realtime: Tauri events + WebSocket/SSE
- Monorepo tools: npm/pnpm + Turbo + Cargo

Decision note:
- `npm` is for frontend toolchain.
- Rust remains the native execution/control layer.
- End-user app does not require Node runtime.

## 4) Macro Architecture
1. App Core (`control plane`)
- bootstrap, router, state shell, feature host, protocol gateway, error model

2. Feature Packs (`domain modules`)
- chat, coding, agents, workflows, projects, memory, settings, business

3. Native Capability Mesh (`adapter layer`)
- camera, filesystem, notifications, browser hooks, sensors, device profile, permissions

4. Runtime Bridge (`engine integration contract`)
- inference router, session gateway, tool gateway, fallback policy

## 5) Folder Stitcher (Scalable)
```text
Apps/app/
  APP-README.md
  package.json
  pnpm-workspace.yaml
  turbo.json

  apps/
    shell-desktop/
      src/
      src-tauri/
    shell-mobile/
      src/
      src-tauri/
    shell-web/
      src/

  packages/
    ui-core/
      src/tokens/
      src/primitives/
      src/patterns/
      src/motion/
      src/accessibility/

    app-core/
      src/bootstrap/
      src/router/
      src/layout-modes/
      src/state/
      src/contracts/
      src/events/
      src/feature-host/
      src/error-model/
      src/observability/

    protocol/
      src/commands/
      src/events/
      src/versioning/
      src/validation/

    platform-adapters/
      src/capability-registry/
      src/permissions/
      src/camera/
      src/filesystem/
      src/browser/
      src/notifications/
      src/device/
      src/sensors/

    features/
      chat/
      coding/
      agents/
      workflows/
      projects/
      business/
      memory/
      settings/

    runtime-bridge/
      src/inference-router/
      src/session-gateway/
      src/tool-gateway/
      src/fallback-policy/

    telemetry-ui/
      src/perf/
      src/ux/
      src/errors/

  docs/
    architecture/
    ux/
    protocol/
```

## 6) UX Architecture (Mode-Based Layout)
### A) Normal Chat Mode
- Header: optional global controls/tabs
- Left panel: chat history, workspace switch, settings entry
- Center: chat thread + composer
- Right panel: context/tools/inspector (toggle)

### B) Coding Mode (VS Code Style)
- Header: open file tabs + mode switch
- Left panel: file tree, project explorer, search
- Center: editor/pages area
- Right panel: AI chat + run/debug/context

### C) Mobile Focus Mode
- Default: only chat window
- Left drawer: history/settings/workspace
- Right drawer: context/tools
- Compact header and gesture-friendly toggles

User preference controls:
- Per-mode panel visibility
- Right panel default on/off
- Header density and tab behavior
- Mobile drawer behavior

## 7) Feature Evolution Matrix
### Now (MVP Foundation)
- Core chat UX with streaming
- History and thread controls
- Mode-based layout engine
- Settings shell and workspace shell
- Permission UX (mocked adapters)

### Next (3-6 months)
- Coding mode workflows
- Multi-agent run timeline
- Memory and knowledge panels
- Tool cards and execution tracing
- Business/org controls v1

### Later (6-12 months)
- Advanced automation studio
- Marketplace/integration ecosystem
- Enterprise audit/compliance surfaces
- Deeper native capability packs per OS
- Multi-model orchestration dashboards

## 8) Capability and Permission Model
Capability contract:
- `request(capability)`
- `status(capability)`
- `revoke(capability)`
- `subscribe(events)`

Rules:
- Sensitive native actions require explicit permission.
- Denied state must return typed fallback for UI.
- All privileged actions create audit events.
- Platform support differences are resolved in adapter layer, not UI.

## 9) AI-Agent Workflow Model
Required workflow abilities:
- Planner/Worker/Verifier role orchestration
- Step graph execution and re-run
- Human approval checkpoints
- Retry/fallback branches
- Traceable logs and telemetry metrics

System boundaries:
- `features/workflows`: UI, editors, run controls
- `runtime-bridge/tool-gateway`: tool execution contracts
- `protocol/events`: queued/running/blocked/failed/done streams

## 10) Performance and Reliability Targets
Performance budgets:
- Desktop cold start target: <= 1.8s
- First interactive target: <= 900ms
- Stream frame drop budget: < 2%
- Controlled idle memory baseline per shell

Reliability requirements:
- Typed error contracts across boundaries
- Reconnect-safe streaming sessions
- Feature failure isolation (no global shell crash)
- Graceful degraded modes on unsupported capabilities

## 11) Code Quality and Reusability Rules
- Primitive -> composite -> feature-view component layering
- Shared utilities only in dedicated shared package
- No cross-feature hidden dependency
- Protocol changes require version bump and compatibility test
- Comments explain intent/constraints, not obvious code

## 12) Testing and CI/CD
Test layers:
- Unit: contracts, stores, reducers, utils
- Integration: feature <-> app-core, adapters <-> contracts
- E2E: mode switches, chat flow, permission flow, mobile drawers
- Performance: startup, stream smoothness, memory drift
- Security: blocked native commands and permission bypass checks

CI/CD gates:
- Multi-platform matrix build (web/desktop/mobile)
- Contract compatibility checks
- UI regression snapshots
- Platform artifact outputs (exe/app/dmg/deb/appimage/apk/aab/ipa/web)

## 13) Delivery Phases (Execution-Ready)
Phase A: foundation
- workspace setup, ui-core tokens/primitives, app-core shell

Phase B: layout engine
- normal/coding/mobile mode framework + panel orchestration

Phase C: chat experience
- streaming UI, history, composer, tool cards, context panel

Phase D: workflow + agents
- role timeline, run console, retry and approval surfaces

Phase E: capability shell
- permission broker UX + adapter-backed capability matrix

Phase F: hardening
- performance tuning, reliability, a11y, security guardrails

Phase G: engine bridge
- connect runtime-bridge with existing Rust engine contracts

## 14) Definition of Done (Design Stage)
Design is complete when:
- Mode-based layout behavior is locked.
- Folder boundaries and module ownership are final.
- Feature evolution path (Now/Next/Later) is clear.
- Permission and capability contracts are fixed.
- Performance/reliability/security gates are measurable.

## 15) Strategic Summary
This design gives you what you asked for:
- Today: clean, flexible, premium chat-first app structure
- Tomorrow: coding, agents, business, automation modules without rewrites
- Always: Rust-native power preserved with lightweight cross-platform UX

## 16) Styling Policy (Locked)
- Primary styling system: Tailwind CSS + design tokens.
- Component styling: utility classes + reusable variants (`cva`, `clsx`, `tailwind-merge`).
- SCSS policy: not default. Allowed only for rare complex animation/layout cases that cannot stay maintainable in utility+token model.
- No mixed random styling systems per feature. Keep one consistent pipeline.
- Old frontend migration baseline: Tailwind + Radix + motion patterns are preferred references.
