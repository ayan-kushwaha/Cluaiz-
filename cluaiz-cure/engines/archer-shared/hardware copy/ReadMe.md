# 🏛️ Sovereign Silicon Kernel: Hardware Abstraction & Telemetry Engine

Welcome to the **Sovereign Silicon Kernel** architecture. This module represents the innermost core of the Cluaiz AI CURE Engine, responsible for performing precise, dynamic, and vendor-agnostic hardware interrogation across Windows, Linux, Darwin, and Edge devices.

**Golden Rule of the Architecture:** 
> *The entire system must remain universally agnostic. No hardcoded logic, no blindly assuming CUDA exists, and no Windows-specific logic leaking into mathematical algorithms. Hardware is probed securely at the OS-level, and mathematical pipelines consume pure numbers.*

---

## 📑 Core Architecture Map

This module is rigidly structured into four major operational spheres:
1. **The Factory Router:** Platform identification and Conditional Compilation (`cfg`).
2. **The Provider Implementation (OS-Level):** The concrete bindings extracting actual data (Sensors).
3. **The Agnostic Wrappers (API Layer):** The universal bridges for the Engine to call (GPU, CPU, NPU).
4. **The Telemetry & Mathematical Profiles:** Benchmarks and strict physics equations evaluating capability.

---

### Phase 1: The Factory Router & Contract

#### `mod.rs` & `platform.rs` (The Central Coordinator)
This is the single source of truth for OS-routing. 
- **Role:** Rather than placing OS checks (`if windows`) haphazardly across the engine, `platform.rs` securely detects the active environment.
- **`mod.rs` (`get_provider()`):** A public factory function that invokes the correct native sensor module using conditional compilation (`#[cfg(target_os = ...)]`). This ensures Linux builds don't compile Windows WMI dependencies.

#### `provider.rs` (The Universal Blueprint)
- **Role:** Exposes the `SiliconProvider` Rust Trait. 
- **Logic:** Every OS-specific sensor MUST implement this trait to return a clean `SovereignProfile`. This is what guarantees that wrappers (like `gpu.rs`) can blindly call `detect_specs()` without caring if they are running on a Raspberry Pi or an Nvidia DGX server.

#### `mod_types.rs` (The Data Standard)
- **Role:** Contains `SovereignProfile`, `MemoryProfile`, `StorageProfile`, and `ComputeProfile`.
- **Logic:** All hardware metrics (Core Count, RAM size, VRAM footprint, Bandwidth GB/s) are standardized here.

---

### Phase 2: Concrete OS Sensors (The "Extractors")

The `SovereignProfile` is constructed dynamically by the following physical probes. **They are forbidden from talking to each other** to enforce strict decoupling.

#### `windows_sensor.rs` (Windows Native Ecosystem)
- **Functions:** Leverages WMI, DXGI, and PowerShell to extract Windows-specific mother-board information.
- **Isolation Goal:** Maps `AdapterRAM` to `vram_gb` and applies specific chipset bandwidth heuristics (e.g., RTX 3050 vs RTX 4090) natively, ensuring CUDA isn't just "assumed" because it's a Windows machine. 

#### `linux_sensor.rs` & `darwin_sensor.rs`
- **Linux Function:** Deep-scans `/sys/class/drm/` and `procfs` to identify attached PCI Graphic Devices (AMDGPU, Intel i915, Nouveau). Validates if ROCm or CUDA actually exists.
- **Darwin Function:** Connects strictly to Apple's Metal Performance APIs (`MTLDevice`) and `sysctl` to retrieve Silicon SoC Unified Memory capacities (M1/M2/M3).

#### `mobile_probe.rs` & `tpu.rs` (The Edge Domain)
- **Functions:** Bridges connections to Android NDK APIs (Adreno/Mali detection) or Edge devices (Google Coral TPU/Raspberry). Detects extreme-low bandwidth constraints cleanly.

---

### Phase 3: The Agnostic API Wrappers

The core AI engine *does not* call `windows_sensor` directly. It interacts purely with the Wrappers.

#### `gpu.rs` & `cpu.rs` & `npu.rs`
- **Role:** The facade layer for the rest of the engine.
- **Logic:** A caller executing `gpu.probe_brand()` fires a request that safely chains into `get_provider().detect_specs()`. These files act as dedicated namespaces to manage complex accelerator types (Neural Processing Units vs standard compute blocks).

#### `hal.rs` (Hardware Abstraction Layer) 
- Acts as a bridge to align external libraries (like `llama-cpp-sys` or `Candle`) to the detected backend capabilities.

#### `isa_probe.rs`
- Instruction Set Architecture (ISA) Validation. Evaluates if the current CPU supports strict tensor acceleration paths (`AVX2`, `AVX-512`, `F16C`, or `NEON`).

---

### Phase 4: Mathematics, Physics, & Thermal Governance

The Engine calculates theoretical capability *before* loading immense AI models.

#### `speed_checker.rs` (The Physics Engine)
- **Role:** Calculates Tokens Per Second ($TPS$) boundaries dynamically.
- **Architecture Priority:** `speed_checker.rs` is strictly forbidden from executing hardware evaluation. It acts purely as a physics calculator mapping the constraints defined by `ComputeProfile`.
- **The Hybrid Logic:** Evaluates Dual-Channel execution (`Time = GPU_Time + CPU_Time` -> $TPS$). Enforces the strict "Block Protocol" (⚫) if expected hybrid performance crashes below 5.0 TPS due to slow System RAM offloading.

#### `benchmark.rs`
- **Role:** Active Memory Stress Tester. 
- **Function:** Runs a 50ms lock-free memory memcpy probe immediately upon engine boot to record practical System RAM Bandwidth ($B_{sys}$), replacing theoretical numbers with the stark reality of the physical machine's current load.

#### `telemetry.rs`
- **Role:** Live dashboarding without latency disruption.
- **Architecture:** Binds directly to an isolated background thread (`GhostObserver`) operating on Lock-Free Atomics. Monitors inference heat loops (`KV Cache Allocation`, `TPS`, `RAM Pressure`) continuously without locking mutexes that would pause token generation.

#### `governor.rs` & `scheduler.rs`
- **Role:** Real-time throttling algorithms to protect hardware lifespan.
- **Mechanics:** Analyzes thermal metrics returned from the Provider. If `temperature_celsius > Limit`, forces the `scheduler.rs` to insert micro-delays (nanosecond yielding) between tensor calculations, ensuring laptops don't critically overheat during massive 100% LLM prompts.

---

### 🛡️ Core Rules for Contributors (CTO Mandate)

1. **NO HARDCODING METRICS:** A calculation needing `Bandwidth` must obtain it through the `SiliconProvider`. You cannot guess `300.0`.
2. **NO ASSUMPTIONS ON CUDA:** GPU capabilities are OS-Agnostic. A machine without CUDA must fallback gracefully to Vulkan, CPU, or Metal seamlessly via the `has_cuda` or `has_gpu` booleans provided.
3. **RESPECT THE PROFILES:** Changes to the metric definitions belong in `mod_types.rs`. Changes to extraction belong in `<os>_sensor.rs`. Changes to physics predictions belong in `speed_checker.rs`. Never co-mingle extraction logic with mathematical calculation.
