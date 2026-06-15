# 🛠️ Interface Utilities (`interface-engines/utils/`)

<p align="center"><strong>Backend Agnostic Tooling</strong></p>

---

## 🎯 Deep Purpose

The `utils/` crate inside the `interface-engines` subsystem provides stateless helper functions specifically required for bridging and dispatching models. It handles low-level bitwise operations, memory offset calculations, and backend-agnostic string manipulations that would otherwise clutter the main execution routes.

## 🧬 Significant Details
- **The Core Logic:** Pure Rust functions for parsing byte streams, hashing native model binaries, and calculating optimal thread distributions before passing them to the active backend.
- **The "Why":** Keeps the `dispatcher` and `global_runtime` clean and focused entirely on state and routing, isolating all the noisy, stateless math helper code.
