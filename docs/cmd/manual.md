# CLI Manual

The Cluaize CLI provides a set of direct terminal utilities to calibrate hardware, manage models, and run interactive generation.

---

## Core Command Set

The binary supports the following options:

| Command | Category | Description | Example |
| :--- | :--- | :--- | :--- |
| `cluaize` | Core | Launches the interactive TUI Dashboard. | `cluaize` |
| `cluaize help` | Core | Displays command-line help screen. | `cluaize help` |
| `cluaize run <model-id>` | Models | Pulls and executes the specified model. | `cluaize run bonsai:8b` |
| `cluaize --calibrate` | System | Re-scans hardware limits and updates config profiles. | `cluaize --calibrate` |
| `cluaize --benchmark` | System | Executes a full hardware speed benchmark. | `cluaize --benchmark` |

---

## Configuration JSON Assets

All CLI settings and commands are tracked in two local JSON profiles:

*   **`assets/commands.json`:** Mapped entries and command usage parameters, baked directly into the Rust binary at compile time.
*   **`Independent.json` / `system_control.json`:** Tracks active local configuration, hardware profiles, user identities, and loaded weight paths.
