# Cluaiz App System Design (Archer)

## 1) Objective (UI-First, Engine-Isolated)
- Build a single `Cluaiz App` architecture for Desktop + Mobile + Web with Tauri 2 as native shell and shared UI system.
- First delivery target is `UI foundation only` (chat-grade experience), with strict isolation from engine runtime.
- Engine, inference, and hardware control integration will come only after UI contracts stabilize.

## 2) Design Principles (Rules-Aligned)
- DRY First: reusable shared modules, no duplicate feature logic across platforms.
- Modular Boundaries: each domain has isolated folder, explicit interfaces, and no cross-layer shortcut imports.
- Monolith Core + Modular Packs: one app core for navigation/state/contracts; feature folders plugged through registry.
- No Hardcoded Platform Logic in UI: platform behavior routed through adapters/contracts only.
- Reusable Comments Rule: only high-value comments at boundaries and non-obvious flows.

## 3) Proposed Folder Stitcher (Apps/app)
```text
Apps/app/
  APP-README.md
  package.json
  pnpm-workspace.yaml
  turbo.json
  .editorconfig
  .gitignore

  apps/
    shell-desktop/              # Tauri 2 desktop wrapper (Win/macOS/Linux)
      src-tauri/
      src/
    shell-mobile/               # Tauri 2 mobile wrapper (Android/iOS)
      src-tauri/
      src/
    shell-web/                  # Web runtime shell (PWA/Browser)
      src/

  packages/
    ui-core/                    # Pure design system + primitives (no business logic)
      src/tokens/
      src/primitives/
      src/patterns/
      src/accessibility/

    app-core/                   # App monolith core (state, routing, contracts, feature host)
      src/bootstrap/
      src/router/
      src/state/
      src/events/
      src/contracts/
      src/feature-host/

    features/
      chat/
        src/ui/
        src/state/
        src/contracts/
      composer/
        src/ui/
        src/state/
      history/
        src/ui/
        src/state/
      agents/
        src/ui/
        src/state/
      settings/
        src/ui/
        src/state/
      workspace/
        src/ui/
        src/state/

    platform-adapters/          # platform-specific bridges hidden behind interfaces
      src/camera/
      src/filesystem/
      src/notifications/
      src/browser/
      src/permissions/
      src/device/

    protocol/                   # versioned command/event schemas for UI <-> native/core
      src/commands/
      src/events/
      src/versioning/

    telemetry-ui/               # UX and perf telemetry (UI-level only)
      src/metrics/
      src/traces/

  tooling/
    lint/
    test/
    build/

  docs/
    architecture/
      app-system-overview.md
      ui-contracts.md
      adapter-contracts.md
    ux/
      chat-experience-spec.md
      interaction-motion-spec.md
```

## 4) Monolith + Module Strategy
- Monolith Part (`packages/app-core`):
  - App lifecycle, route control, global state shell, feature host, protocol binding.
  - No feature-specific rendering logic inside core.
- Modular Part (`packages/features/*`):
  - Each feature self-contained: `ui + local state + contracts`.
  - Registered into `feature-host` through manifest.
- Stitch Rule:
  - `ui-core` used by all features.
  - `platform-adapters` consumed only through `app-core/contracts`.
  - `features/*` never call native platform APIs directly.

## 5) UI System Blueprint (AI Chat Class UX)
- Visual System:
  - Tokenized typography, spacing, radius, elevation, motion durations.
  - Theming with light/dark + brand semantic palette.
- Core Surfaces:
  - Left rail (workspace/history/agents), primary chat canvas, right context panel.
  - Mobile variant: bottom nav + sheets + focus composer.
- Chat Essentials:
  - Streaming message blocks, composer with attachment slots, tool-call cards, retry/regenerate, pinned context.
  - Message actions: copy, edit, branch, summarize, reference.
- Performance UX:
  - skeleton states, optimistic transitions, incremental rendering, list virtualization.

## 6) Reusable Contract Design (Engine-Isolated for Now)
- `CapabilityProvider` contract in UI layer (mock + stub enabled for phase-1).
- `PermissionBroker` contract for request/check/revoke workflow and denial recovery UI.
- `InferenceRouter` contract represented as UI-side strategy only (no real engine binding in phase-1).
- `Protocol` package owns command/event schema versions to avoid tight coupling.

## 7) Build + Release Design (Single Codebase, Multi Artifact)
- One codebase, separate outputs:
  - Windows `.exe`
  - macOS `.app/.dmg`
  - Linux `.AppImage/.deb` (as configured)
  - Android `.apk/.aab`
  - iOS `.ipa`
  - Web static/PWA bundle
- GitHub Actions matrix can build all with platform runners and signing secrets.

## 8) Phase Plan (UI-Only First)
- Phase A: `ui-core` + app layout shell + navigation system.
- Phase B: chat feature UI (streaming mock, composer, thread states).
- Phase C: settings/workspace/history/agents UI modules.
- Phase D: platform adapter mocks + permission UX flows.
- Phase E: polish (motion, responsiveness, a11y, perf budgets).
- Phase F: only after UI lock, start engine connectivity phase.

## 9) Non-Goals (Current Stage)
- No direct engine wiring.
- No production inference runtime.
- No deep OS privileged actions implementation.

## 10) Definition of Done (UI Foundation)
- Shared design system in place and consumed by all app shells.
- Chat-class UI working across desktop/mobile/web shells with mock data.
- Adapter contracts and protocol versioning finalized.
- CI build matrix structure prepared for multi-platform artifacts.
