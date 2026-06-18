# 🧿 Cluaize Neural Hub: Cluaize-DNA CLI (Architecture & Master Source of Truth)

This document serves as the **Single Source of Truth** and the **Architectural Brain** for the Cluaize CLI (The "Neural OS" Dashboard). It defines the deep logic, user experience sequences, CLI toolchain, and hardware-level governance that power the Cluaize ecosystem.

---

## 💻 1. The Command Arsenal (Complete CLI Reference)

The Cluaize CLI provides deep, bare-metal control over local AI orchestration.

### 🧩 Neural Skills & Sandboxed Agents
```bash
$ cluaize skill install <skill_name>   # Install a WASM Sandboxed Skill
$ cluaize skill list                   # List active OS-level skills
$ cluaize skill cache ls               # View generated SSD KV Caches
$ cluaize skill cache clear <model>    # Clear a specific corrupted cache
$ cluaize skill cache clear --all      # Nuke all orphaned caches globally
```

### 🧠 Core Engine & Evaluation
```bash
$ cluaize benchmark                    # Process-isolated full system VRAM stress test
$ cluaize benchmark bonsai1-8b --runs 3 # Iterative thermal-throttled evaluation
$ cluaize run <model_id>               # Launch CLI stream via C++ FFI Kernel
$ cluaize test-jit                     # Dynamic Pipeline Diagnostic & Memory Check
```

### 🗄️ Vault Management & Identity
```bash
$ cluaize pull <model_id>              # Fetch weights silently (Pre-flight Hardware Audit)
$ cluaize rm <model_id>                # Permanently free up SSD physical blocks
$ cluaize list                         # Active CoreRoster visualization
$ cluaize ps                           # Auto-heal stale PIDs and monitor memory
$ cluaize ingest <file_path>           # RAG: Semantic chunking via ONNX gatekeepers
$ cluaize setup profile                # LMDB Vectorization of Node Identity
```

### 🎛️ Bare-Metal Control
```bash
$ cluaize --calibrate                  # Real-time RDTSC hardware clock & SIMD profiling
$ cluaize booster                      # Interactive Hardware Tuning (Quant, FlashKDA)
$ cluaize brain <on/off/status>        # Manage Cluaizd FFI Daemon gateway
$ cluaize                              # Launch the Full TUI Interactive Dashboard
```

---

## 🏗️ 2. Core Philosophy: The "First-Boot" Handshake

Every top-tier neural engine must establish a "Neural Handshake" with the user. Cluaize's onboarding (`src/ui/apps/onboarding/`) is a sequential, animated journey that defines the **Structural DNA** of the local node.

### **Phase A: Neural Ignition (The Intro)**
- **Logic**: A 60-frame mathematical ASCII animation sequence (`ritual.rs`).
- **Implementation**: The `CLUAIZE` logo starts in `Color::DarkGray` and pulses into `Color::Cyan` over 1 second, providing a true active "boot-up" feel.

### **Phase B: Cluaize Identity & Native Extraction**
Cluaize must know who it is serving to optimize the neural weights:
1.  **Auth & Identity Entry**: Supported via Google, Email, or Guest (`native.rs`).
2.  **Utility Logic**: Determining the Purpose (Research, Production, Creative).
3.  **Hardware DNA Audit**: Live "Bare Metal" calibration (`HardwareGovernor::auto_calibrate()`) that checks CPU cores, VRAM, and RAM, asking the user to enable `TurboQuant` and `FlashAttention_v2`.

---

## 🧬 3. Deep Engine Patterns (The Core Orchestrator)

Cluaize adopts extreme performance hacks and deep architectural patterns to ensure maximum bare-metal optimization:

- **Surgical Silence (OS-level C++ Mute) 🔇**: During active inference, the system intercepts `stderr` via `libc::freopen("NUL\0")` to block noisy C++ backend logs from breaking the TUI, restoring `CONOUT$` immediately when done (`dashboard.rs`).
- **Live Keystroke Interception ⚡**: While streams are generating, `crossterm::event::poll` listens at the OS level. `Ctrl+T` instantly triggers `GLOBAL_SKIP_THINKING_SIGNAL` to hide reasoning chains dynamically, and `Ctrl+C` initiates an **Agentic Pause**, pausing inference safely in VRAM and asking the user for `[PIVOT_CONTINUE]` mid-stream.
- **Process-Isolated Benchmarking 🏎️**: `benchmark.rs` does not run in the same process. It uses `std::process::Command::new(current_exe)` to fork itself, ensuring absolute 100% VRAM release between evaluations.
- **FFI Vectorization Hooks 🧠**: Upon chat input, `EmbeddingGenerator::generate_vector` is invoked blockingly, storing prompt context dynamically into the local LMDB `storage_bridge`.

---

## 🎛️ 4. The System Booster (Hardware DNA Control)

The Engine allows dynamic, on-the-fly mutations to hardware behavior via `system_control.json`:

- **`mode_run`**: VRAM Allocation Strategies (`Edge`, `Balance`, `MaxBoost`, `UltraMaxBoost`).
- **`kv_cache_quantization`**: Dynamic adjustment of `Kv16` and `Kv8` based on layer depth.
- **`context_shifting`**: Strategies to handle token overflow (e.g., Aggressive 25% shift vs Standard).
- **`force_vram_reclaim`**: Hard stops generation if a 0.5% VRAM margin is breached.
- **Compute Offload Selection**: Interactive sliders to push exactly `-1` (Full GPU) or custom layers to CPU fallback.

---

## 🧠 5. TUI Architecture (State & Scalability)

### **State Machine Design (`core/state.rs`)**
Cluaize's state is divided into robust asynchronous dimensions:
1.  **OsState (Lifecycle)**: `Setup(OnboardingStep)` vs `Dashboard`.
2.  **Activity Stream (REPL)**: A dynamic vector of `ActivityBlock` elements. Blocks contain Live Sender Mapping (e.g., Cyan for `USER`, Yellow for `ARCHER`, Red for `ERROR`).

### **Scalability (The Async Governance)**
- **Thread Governance**: All Heavy ONNX compilation and Model downloading (`details.rs`) happens in separate `tokio::spawn` threads communicating with the UI via `mpsc` channels.
- **Dynamic Render Widths**: `ColumnWidths::compute` runs actual real-time terminal math via `crossterm::terminal::size()` to clamp UI elements responsively.
- **Visual Precision**: Hardware pulse bars (`app.live_pulse`) synchronously render RAM, CPU, and Power Draw at the bottom of the screen without interfering with chat streams.

---
**Cluaize Neural Hub: Built for the Cluaize-DNA Node. 🧿⚡🚀**
