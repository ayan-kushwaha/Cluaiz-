# 🔬 Sovereign Logic Trace: The Inference Heartbeat
## Archer Engine Hardware Execution Lifecycle

This document traces the step-by-step logic flow inside the Silicon Kernel during a real-time prompt cycle. It highlights the interaction between the **Intelligence Tier** and the **Physical Sensing Tier**.

---

### Step 1: Physical Boundary Lockdown [GOVERNOR]
**Location:** `intelligence/governor.rs`  
**Logic:** Upon engine boot, the Governor invokes `hal::detect_silicon()`.  
- **Action:** Captures Base Clock (GHz), Total Threads, and VRAM Capacity.
- **Trace:** If `system_control.json` is missing, it auto-calibrates and locks these physical limits permanently. 
- **Goal:** Zero dynamic probing during critical token generation.

### Step 2: Adaptive Frequency Initiation [GHOST OBSERVER]
**Location:** `intelligence/telemetry.rs`  
**Logic:** A background thread is spawned using atomic synchronization.
- **Action:** Polls `sensors/` at variable frequencies (1000ms idle, 50ms under load).
- **Trace:** Captains the `SILICON_PULSE` static Arc, storing AtomicU32 values for temperature and VRAM pressure.

### Step 3: Compute Routing [SCHEDULING]
**Location:** `intelligence/scheduler.rs`  
**Logic:** When a model is loaded, the Scheduler evaluates the `SovereignProfile`.
- **Action:** It does not check the OS. It checks **Capability Flags** (e.g., `has_avx512`, `has_cuda`, `has_metal`).
- **Trace:** Resolves the `ComputeBackend` (Vulkan/Metal/Cuda). It also checks VRAM pressure from the Pulse to decide if Hybrid Offloading is required.

### Step 4: Memory Block Reservation [ALLOCATOR]
**Location:** `memory/allocator.rs`  
**Logic:** The engine reserves memory pages.
- **Action:** Carves out `PastKeyValues` (KV-Cache) blocks.
- **Trace:** Prepares for potential **AtmaSteer** injection by mapping physical RAM pointers to the model's attention layers via FFI.

### Step 5: Thermal Guard Intervention [GOVERNOR]
**Location:** `intelligence/governor.rs`  
**Logic:** During heavy generation, the Governor monitors `SILICON_PULSE`.
- **Action:** If thermal limits exceed 90°C, it triggers "Survival Gear".
- **Trace:** Adjusts the global `ENGINE_GEAR`, which the scheduler uses to insert micro-delays (nanosecond yielding), protecting the lifespan of the silicon.

### Step 6: Atomic Metrics Relay
**Location:** `hardware/mod.rs` (`get_live_metrics`)
**Logic:** The CLI UI calls the root facade.
- **Action:** Returns a zero-latency snapshot of system health.
- **Trace:** Dispatched via the HAL provider, ensuring the UI remains flicker-free and detached from inference stalls.

---

## 🔱 The "Native" Difference
Unlike standard wrappers that act as an *external shell*, the Silicon Kernel acts as a **Sovereign Interface**. 

| Lifecycle Phase | Standard Wrapper Mode | Archer Sovereign Mode |
|:--- |:--- |:--- |
| **Detection** | Calls OS API on every prompt | Locks Physical Identity at Boot |
| **Logic** | OS-Specific branches everywhere | Universal HAL Trait Dispatch |
| **Telemetry** | Blocks thread with Mutex | Lock-Free Atomic Observation |
| **Architecture** | Leaky abstraction | Strict 7-Tier Domain Separation |

---
*Maintained by the Archer Architecture Focal Point*
