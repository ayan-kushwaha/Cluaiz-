# Introduction to Cluaiz

Cluaiz is a bare-metal neural engine orchestration framework engineered to run high-performance LLM and deep learning inference directly on local silicon. The framework eliminates the heavy, fragmented Python runtime stacks (like PyTorch, Hugging Face Transformers, and virtual environments) by replacing them with compiled, low-latency Rust binaries, native C++ compute kernels, and direct accelerator driver bindings.

This documentation serves as a comprehensive, deep-dive architectural manual for developers, operators, and systems architects to understand, build, and deploy Cluaiz nodes from scratch.

---

## 🏛️ Why Cluaiz? Solving Edge Inference Bottlenecks

Traditional local LLM deployments are plagued by massive resource overhead, thread blocking, and unstable dependencies. Cluaiz resolves these bottlenecks through three core design principles:

### 1. Zero-Copy Decoupled Shell
Most terminal-based LLM runners compile the user interface and the matrix-multiplication kernels in the same execution thread. When the CPU or GPU is saturated with tensor computations, the user interface locks up, keypresses are dropped, and telemetry screens freeze. 

Cluaiz strictly isolates the user interface shell (`cluaiz-cli`) from the core execution engine (`cluaiz-engine`). The two processes run independently, communicating via high-speed local loopbacks using asynchronous multi-producer single-consumer (`mpsc`) message channels. The UI thread remains fluid and responsive (stable 60 FPS) even when the compute engine is consuming 100% of GPU resources.

### 2. Multi-Accelerator Platform Agnosticism
Running AI at the edge requires compiling different backends for different silicon. Cluaiz compiles separate native libraries for every major platform. 
Instead of forcing a single CUDA or Metal build that fails on unsupported systems, Cluaiz implements a **Dynamic Silicon Dispatch** system. At startup, the engine audits the host machine's hardware and dynamically binds the optimal compiled compute bridge (CUDA, Metal, Vulkan, ROCm, HIP, or OpenVINO). If no accelerator is present, it seamlessly falls back to optimized CPU vector instruction sets (AVX512, AVX2, or ARM Neon).

### 3. Absolute Memory Safety & Isolation
By building the core runtime in Rust, Cluaiz prevents memory leaks, pointer overflows, and invalid memory access during long-context weight swaps. Models are dynamically mounted into and pruned from VRAM/RAM through a controlled transactional lifecycle, preventing typical Out-Of-Memory (OOM) segmentation faults.

---

## 🧬 The Four-Tier Stack Architecture

The Cluaiz codebase is cleanly divided into four decoupled architectural layers, ensuring that changes to the user interface never impact compute integrity, and new compiler flags in compute kernels never break the host engine.

```
┌─────────────────────────────────────────────────────────┐
│                    EDGE INTERFACE                       │
│      cluaiz-cli (Ratatui TUI / Terminal Terminal UI)    │
└────────────────────────────┬────────────────────────────┘
                             │ Local REST / SSE IPC Loop
┌────────────────────────────▼────────────────────────────┐
│                  ORCHESTRATION BRAIN                    │
│      cluaiz-engine (Axum HTTP REST & Tokio State)       │
└──────────────┬───────────────────────────┬──────────────┘
               │ Dynamic FFI               │ Dynamic FFI
┌──────────────▼─────────────┐┌────────────▼─────────────┐
│      INFERENCE KERNEL      ││     SILICON ACCELERATOR    │
│    cluaiz-kernel (SIMD)    ││   cluaiz-driver (GPUs)   │
│   [AVX512 / AVX2 / NEON]   ││   [CUDA / Metal / Vulkan]│
└────────────────────────────┘└──────────────────────────┘
```

### 💻 Tier 1: The Edge Client (`Apps/cli`)
The interactive control shell built in Rust using the `ratatui` UI drawing engine.
*   **Onboarding Engine:** Seeds the user's local workstation folder (`~/.cluaiz/workspace`) with configuration and state profiles: `IDENTITY.md`, `USER.md`, and `SOUL.md`.
*   **The Sentinel Protocol:** Creates a local `.ignition_lock` file during the onboarding sequence. If the shell is aborted, subsequent boots resume onboarding at the exact phase left off.
*   **UI Thread Separation:** Uses an event polling loop to capture keypresses and render widgets asynchronously. Telemetry and token chunks are received from the engine via local HTTP SSE loopback streams.

### 🧠 Tier 2: The Core Brain (`cluaiz-engine`)
The system orchestrator built on the `axum` HTTP web framework, running on a multi-threaded `tokio` asynchronous scheduler.
*   **Process Governance:** Dynamically monitors thread priorities, CPU/GPU temperatures, memory allocations, and schedules parallel generation queues.
*   **System Booster Core:** Contains the core execution acceleration libraries:
    *   `manager/conflict_resolver.rs`: Ensures incompatible compiler profiles (e.g., speculative decoding vs. low VRAM systems) bypass safely to avoid crashing the GPU.
    *   `manager/auto_tuner.rs`: Evaluates system parameters and profiles host hardware limits to configure compute priorities.
    *   `dflash/`: Handles Block-Diffusion speculative verification via the `DDTree` token routing algorithm.
    *   `turbo_quant/`: Precision quantizer performing Givens/Hadamard matrix rotations, Mean-Squared-Error quantization, and polar weight corrections for 3-bit/4-bit compression.

### ⚙️ Tier 3: The SIMD Inference Kernel (`cluaiz-kernel`)
Base CPU interpreters compiled for target processors without GPU or NPU access.
*   **Instruction Sets:** Hard-optimized vector operations using Intel/AMD AVX512/AVX2 instruction sets and Apple/ARM Neon extensions.
*   **Compilation Matrix:** Compiled across exactly 9 distinct platform profiles (Windows x64, Linux x64/Aarch64/ARMv7, macOS ARM64/x64, Android, iOS).

### 🔌 Tier 4: The Accelerator Bridge (`cluaiz-driver`)
Direct binary drivers providing absolute native performance on graphics and tensor processors.
*   **Hardware Wrappers:** Bindings for NVIDIA CUDA (v11/v12/v13), Apple Metal, Vulkan (Universal), AMD ROCm/HIP, and Intel OpenVINO.
*   **Dynamic Linking:** The engine scans and binds these dynamic libraries (`.dll` / `.so` / `.dylib`) at runtime based on the hardware audit.

---

## 🌊 The Lifecycle of a Prompt (Technical Execution Flow)

To understand how data flows through Cluaiz, here is the chronological step-by-step path of a prompt request from the keyboard to the GPU matrix multiplication and back:

```
[User Types Prompt] ──> [cli: captures event] ──> [POST /chat Request]
                                                         │
[TUI Stream Display] <── [SSE Response Chunks] <── [engine: Axum Route]
                                                         │
[Dynamic Driver Swap] ──> [Matrix Mult (GPU/SIMD)] <── [System Booster Align]
```

### Phase A: Capture & IPC Dispatch
1.  The user types a prompt inside the interactive TUI chat console and presses `Enter`.
2.  `cluaiz-cli` locks the input container, wraps the input text into a standardized JSON packet, and dispatches a `POST` request to `http://127.0.0.1:3000/chat`.
3.  The main thread of the CLI immediately continues polling for UI events (such as resizing the window, checking status indicators, or moving the cursor) without waiting for the engine's response.

### Phase B: Orchestration & Tuning
4.  `cluaiz-engine` receives the API request. The Axum handler delegates the transaction to the async state scheduler.
5.  The **Auto Tuner** evaluates the prompt request against the currently loaded model.
6.  The **Conflict Resolver** verifies hardware telemetry:
    *   *If prompt context exceeds standard bounds:* It executes KV-Cache pruning and forces block-diffusion off.
    *   *If VRAM constraints are optimal:* It routes the model context through `dflash` speculative decoding threads.
7.  The scheduler dispatches a tensor compute request to the loaded execution backend.

### Phase C: Native Execution & Streaming
8.  The dynamically loaded `cluaiz-driver` performs matrix multiplications on the GPU (via CUDA/Metal compute shaders) or CPU SIMD cores.
9.  As individual tokens are resolved, the engine formats them into Server-Sent Events (SSE) packets.
10. The Axum HTTP stream pushes the packets back to the local network loopback.
11. The background thread of `cluaiz-cli` catches the SSE chunks, pipes them through an asynchronous channel directly to the UI rendering loop, and the characters appear on the terminal with absolute fluid precision.

---

## ⚙️ Workstation Node Configuration

Cluaiz nodes manage state and preferences through local configuration profiles. Developers can inspect and customize these settings to adjust computing weights or interface behavior:

### System Controller Profile (`sovereign.json`)
Located in the primary directory of the node, this file governs hardware limits and environment parameters:

```json
{
  "node_id": "cluaiz-node-x1",
  "active_model": "bonsai:8b",
  "user_identity": {
    "name": "Operator",
    "purpose": "PRODUCTION"
  },
  "hardware_governance": {
    "vram_limit_gb": 12.0,
    "cpu_thread_limit": 8,
    "allow_speculative_decoding": true,
    "fallback_to_cpu": true
  },
  "network": {
    "api_host": "127.0.0.1",
    "api_port": 3000,
    "enable_cors": true
  }
}
```

*   **`purpose` Modes:**
    *   `RESEARCH`: Optimizes for high-precision quantization layers (slower generation, higher accuracy).
    *   `PRODUCTION`: Optimizes for maximum token throughput and parallel execution threads.
    *   `CREATIVE`: Optimizes for extended context windows and relaxed pruning constraints.
*   **`hardware_governance`:** Operators can lock maximum VRAM allocations to ensure that background tasks do not starve the primary operating system graphics loops.

---

## 🛠️ Cross-Compilation & Deployment Pipeline

Cluaiz employs a highly automated, cross-platform CI/CD pipeline to compile, validate, and index compute binaries for all supported hardware profiles.

### The Pipeline Architecture
The `.github/workflows/` directory contains isolated workflow runners compiling parallel dependencies:

1.  **SIMD Baseline Compilation (`inference-kernel.yml`):**
    *   Uses Docker-based `cross` compiler containers to target ARM (Aarch64, ARMv7) and x86 architectures without virtualized hardware.
    *   Validates instruction compilation flags (`-C target-feature=+avx512f` or `+neon`) to prevent binary execution faults on target platforms.
    *   Pushes compiled binaries directly to GitHub `kernel-v*` release indices.
2.  **Silicon Accelerator Compilation (`inference-driver.yml`):**
    *   Compiles dynamic driver binaries for CUDA backends (using official NVIDIA CUDA developer environments), macOS Metal (compiled on Apple Silicon cloud hosts), and Vulkan runners.
    *   On a successful compile, the workflow runs a synchronization script to update driver tags and version mappings inside the master `registry.json` database.

---

## 🧬 Summary of Developer Guidelines

For developers looking to extend Cluaiz, always follow these core engineering principles:

1.  **Keep Layers Decoupled:** Never write terminal UI drawing logic inside `cluaiz-engine` or compute libraries. The CLI must remain a pure interface client.
2.  **No Blocking on Main Thread:** Ensure all disk reads, REST queries, and tensor computations are spawned inside asynchronous `tokio` worker threads. The Ratatui draw loop must execute instantly to prevent terminal flickering.
3.  **Strict Quantization Integrity:** When developing new model pipelines under `turbo_quant`, always cross-reference weight corrections with Mean-Squared-Error matrices to guarantee precision retention.
