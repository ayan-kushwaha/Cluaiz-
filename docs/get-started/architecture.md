# System Architecture & Design

Cluaize is built as a highly modular, decoupled system. By separating user-facing terminals from heavy mathematical processors, the platform achieves absolute structural resilience, thread safety, and zero-flicker UI updates.

This document details how the Client (Frontend Interface) and Backend (Compute Engine) interact, how hardware acceleration is dynamically resolved, and how processes coordinate execution under the hood.

---

## 🏛️ High-Level Architectural Flow

The entire system relies on a loopback network bridge. The Client process behaves as a lightweight consumer, while the Engine process executes as a persistent, high-performance background daemon.

```
┌────────────────────────┐                   ┌────────────────────────┐
│     CLIENT CONTAINER   │                   │    ORCHESTRATOR ENGINE │
│  (Apps/cli - Ratatui)  │                   │ (cluaize-engine - Axum) │
└───────────┬────────────┘                   └───────────▲────────────┘
            │                                            │
            │ 1. POST /chat JSON Payload                 │ 4. Dynamically Swaps Models
            ├────────────────────────────────────────────┘
            │
            │ 2. SSE Streams (Chunks)
            │◄───────────────────────────────────────────┐
            │                                            │
┌───────────▼────────────┐                   ┌───────────┴────────────┐
│      TUI MAIN LOOP     │                   │     SILICON DISPATCH   │
│  (Render Frame at 60Hz)│                   │ [CUDA / Metal / SIMD]  │
└────────────────────────┘                   └────────────────────────┘
```

---

## 💻 1. The Client System: `cluaize-cli` (Edge Interface)

The client is a pure terminal UI (TUI) running in the operator's shell. It is responsible only for rendering text, displaying telemetry charts, capturing keyboard input, and persisting local files.

### 👤 The User Perspective
When a user launches `cluaize`, they are greeted by an interactive dashboard that lists available models, displays active CPU/GPU temperatures, and opens a direct chat loop. The interface is optimized to remain completely active; even when a model is swamped with complex math, the cursor continues to blink, telemetry grids update, and scrolling is smooth.

### ⚙️ The Developer Perspective
The client is written in Rust, using `ratatui` for text-based graphics and `crossterm` for terminal event processing. To ensure fluid user experience, the CLI splits operation across two thread domains:

*   **The Main Rendering Thread:** Runs a deterministic loop that captures keyboard focus, handles cursor positions, and re-draws the grid cells at a stable frequency (60Hz target).
*   **The Async Worker Thread:** Spawns a background `tokio` runtime. When the user submits a prompt, this thread dispatches the HTTP payload to the server and captures incoming response chunks using non-blocking multi-producer, single-consumer (`mpsc`) message channels, forwarding the processed strings to the main thread's render buffer.

---

## 🧠 2. The Backend System: `cluaize-engine` (Orchestration Brain)

The engine runs as a background process, listening for local API commands. It operates as the gatekeeper for local hardware resources, swapping model weights in memory, managing inference pipelines, and communicating with hardware acceleration drivers.

### 👤 The User Perspective
The user never interacts with the engine directly. It boots automatically in the background when the TUI is ignited, runs hardware audits, swaps active weights when the user selects a new model from the roster, and automatically shuts down when the TUI console is closed.

### ⚙️ The Developer Perspective
The engine is compiled as a multithreaded Rust REST server built on the `axum` web framework, utilizing shared thread-safe state containers (`Arc<AppState>`).

```
┌────────────────────────────────────────────────────────┐
│                       AXUM ROUTER                      │
│     [/chat]        [/hardware]     [/models/load]      │
└──────────┬──────────────┬──────────────┬───────────────┘
           │              │              │
           ▼              ▼              ▼
┌────────────────────────────────────────────────────────┐
│                   TOKIO ASYNC STATE                    │
│           Arc<AppState> (Thread-Safe Memory)           │
└─────────────────────────┬──────────────────────────────┘
                          ▼
┌────────────────────────────────────────────────────────┐
│               DYNAMIC SILICON DISPATCH                 │
│              (LoadLibrary / dlopen Gates)              │
└──────────┬─────────────────────────────┬───────────────┘
           │                             │
           ▼                             ▼
┌────────────────────┐         ┌────────────────────┐
│  ACCELERATOR BRIDGE│         │ SIMD CPU KERNELS   │
│  cluaize-driver     │         │ cluaize-kernel      │
│  [CUDA / Metal]    │         │ [AVX512 / Neon]    │
└────────────────────┘         └────────────────────┘
```

---

## 🔌 3. Dynamic Silicon Dispatch & FFI Gates

To achieve native speed without Python compilation dependencies, Cluaize implements a dynamic dynamic-link library FFI mapping layer:

### Operating System & Instruction Set Probing
At boot, the engine executes high-fidelity platform discovery commands:
1.  **CPU Vector Detection:** The engine executes assembly-level checks (e.g., `cpuid` on Intel/AMD platforms, or `sysctl` on Apple Silicon) to query native instruction set extensions.
    *   If **AVX512** support is true: The engine enables compiled level-3 vector optimizations.
    *   If **AVX2** support is true: The engine falls back to standard SSE/AVX2 instruction blocks.
    *   If **Neon** support is true (ARM/Apple): The engine utilizes native register pipelines.
2.  **Silicon Driver Mapping (dlopen / LoadLibrary):**
    Rather than hard-linking compiler paths (which would cause a binary to crash if a user doesn't have an NVIDIA GPU or Apple Metal runtime), the engine resolves dynamic libraries at runtime:
    *   *Windows:* Scans `nvcuda.dll` or `cudart.dll` to bind CUDA dynamic runtime functions.
    *   *Linux:* Scans `/usr/lib/libcuda.so` or `/usr/local/cuda/lib64/libcudart.so`.
    *   *macOS:* Links directly to Xcode's native `Metal.framework` APIs.

If these system libraries are successfully mapped, the engine routes tensor operations directly through the FFI (Foreign Function Interface) gate to `cluaize-driver`. If they fail or are absent, it routes execution safely to the SIMD-optimized `cluaize-kernel`.

---

## 📡 4. Inter-Process Communication (IPC) Protocol

The client and server communicate strictly over local loopback sockets using standardized JSON API payloads.

### Streaming Pipeline (Server-Sent Events)
To display responses as they are generated rather than waiting for the entire context to complete, Cluaize employs **Server-Sent Events (SSE)**. 

When the user requests generation, the server keeps the HTTP connection open, pushing discrete data packets chunk-by-chunk using standard `text/event-stream` headers. The CLI background thread reads the streaming loop and immediately updates the active chat block.

### Example IPC Schema Mapping

#### A. Chat Generation Request (`POST /chat`)
Dispatched by the CLI to the Axum engine to queue a new user prompt:

```json
{
  "session_id": "session-88ac-991f",
  "model_id": "bonsai:8b",
  "prompt": "Explain the difference between AVX512 and AVX2.",
  "parameters": {
    "temperature": 0.7,
    "top_p": 0.9,
    "max_tokens": 512
  }
}
```

#### B. Streaming SSE Response Chunk (`text/event-stream`)
The engine pushes these data chunks sequentially over the active loopback thread:

```http
event: token
data: {"token": "AV", "session_id": "session-88ac-991f"}

event: token
data: {"token": "X", "session_id": "session-88ac-991f"}

event: token
data: {"token": "512", "session_id": "session-88ac-991f"}
```

#### C. Model Loading Request (`POST /models/load`)
Sent by the CLI when the operator switches the model within the Roster UI:

```json
{
  "model_id": "gemma:2b",
  "target_backend": "GPU",
  "vram_limit_gb": 8.0
}
```

---

## 🧪 5. Architecture Summary for Developers

If you are developing or maintaining the Cluaize ecosystem, keep these structural laws in mind:

1.  **Keep the FFI Boundaries Clean:** All dynamic library bindings (e.g., calling CUDA or Metal matrix functions) must execute safely within `unsafe` scopes in `cluaize-driver`. Ensure error codes are caught and translated into Rust `Result` variants before reaching `cluaize-engine`.
2.  **No Core Logic inside the TUI:** The `Apps/cli` directory must contain absolutely zero inference code, weight math, or network loading logic. It is a shell interface. If you need to fetch hardware telemetry, query the `/hardware` Axum endpoint; do not compile local hardware sensors inside the TUI app.
3.  **Strict State Synchronization:** When swapping models via `/models/load`, ensure the active weight buffers are completely dropped and garbage-collected before mounting new tensors to prevent memory spikes and OOM faults.
