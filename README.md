<p align="center">
  <img src="assets/logo.png" width="300" alt="Cluaiz Logo">
</p>

<h1 align="center">Cluaiz Neural Ecosystem</h1>

<p align="center">
  <b>Industrial Silicon-Native Inference Infrastructure</b><br>
  <i>Near-cache-latency signaling | CluaizDNA Modular Architecture | Native Silicon Interface</i>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Status-Industrial_Alpha-orange?style=for-the-badge" alt="Status">
  <img src="https://img.shields.io/badge/Logic-Native_FFI-green?style=for-the-badge" alt="Silicon">
  <img src="https://img.shields.io/badge/Architecture-CluaizDNA_Modular-blue?style=for-the-badge" alt="Architecture">
  <img src="https://img.shields.io/badge/Security-Isolated_Sandbox-red?style=for-the-badge" alt="Security">
</p>

---

## 🏛️ **Current Project Status**

> [!IMPORTANT]
> **Status**: **Industrial Alpha (Research Phase)**.
> Cluaiz is an experimental neural infrastructure. While the core architecture is build-stable, hardware-constrained guarantees and specialized ternary kernels are undergoing rigorous validation.

---

## 🧭 **What is Cluaiz? (The Infrastructure Layer)**

Cluaiz is a **Silicon-Native Neural Kernel** designed to orchestrate inference directly on local silicon with a minimized-copy architecture.

| Component | Role | Implementation |
| :--- | :--- | :--- |
| **Cluaiz Engine** | Orchestrator | Rust-Native Kernel Management |
| **Cluaiz DNA** | Manifest | Unified Identity & Versioning |
| **AtmaSteer** | Steering | Hardware-Constrained Decoding |
| **Drivers** | Handshake | Native Silicon FFI (CUDA, Metal, Vulkan) |

---

## 📂 **Deep Repository Structure**

The ecosystem is architected into surgically separated layers to ensure zero cross-contamination and maximum portability.

```text
/Apps
  /cli                  # Industrial CLI (User Handshake)
/cluaiz-engine
  /api                  # Low-latency C-API & REST Handshake
  /engines              # Core Orchestration Runtime (CURE)
    /cluaiz-shared      # Universal Types, Constants & DNA Manifest
    /system-booster     # Hardware Governor & LRU Arbiter
/inference-drivers      # Silicon Registry & Dynamic Provisioning
  /drivers              # Bare-metal kernel binary definitions
  /registry.json        # Global Hardware-to-Backend mapping
/interface-engines      # Specialized Inference Backends
  /llama                # High-performance llama.cpp wrapper
  /candle               # Pure-Rust Candle/BitNet support
  /neural_core          # Unified Engine Contract & Traits
```

---

## 🛠️ **Industrial Setup & Installation**

### 1. Prerequisites (Bare-Metal)
- **Rust Toolchain**: Stable (latest)
- **C++ Compiler**: MSVC (Windows) / Clang (Linux/Mac)
- **Silicon Drivers**: 
  - NVIDIA: CUDA Toolkit 12.x
  - Mac: Apple Command Line Tools (Metal)
  - Intel: OpenVINO Runtime (Optional)

### 2. Building from Source
```bash
# Clone the Ecosystem
$ git clone https://github.com/cluaiz/cluaiz
$ cd cluaiz

# Build the Industrial CLI
$ cargo build --release -p cli

# Optional: Add to PATH
$ export PATH=$PATH:$(pwd)/target/release
```

### 3. Post-Install Calibration
```bash
# Deep-probe silicon and initialize system_control.json
$ cluaiz probe

# Verify kernel linkage
$ cluaiz doctor
```

---

## 🕹️ **Operation Guide (The Neural Workflow)**

### 🔹 **1. Acquiring Neural Weights (Pull)**
Cluaiz uses a registry-based acquisition system to ensure the correct quant format for your specific silicon.
```bash
$ cluaiz pull llama3-8b
# Logic: Checks local hardware -> Queries CluaizDNA -> Downloads optimized GGUF/BitNet blob
```

### 🔹 **2. Live Inference (Run)**
Orchestrate a session with hardware-level memory management.
```bash
$ cluaiz run llama3-8b --prompt "Initialize Sovereign Protocol"
# Logic: VRAM Arbiter checks budget -> Loads native driver -> Spawns Sandbox
```

### 🔹 **3. Skill & State Injection (Inject)**
Inject behavioral constraints or external knowledge directly into the neural context via the **AtmaSteer Protocol**.
```bash
$ cluaiz skill inject ./skills/coding_pro.json
# Logic: Tokenizes prefix -> Writes to KV-Cache buffer -> Forces hardware register mask
```

---

## 🧠 **Core Engineering Pillars**

### 🔹 **1. Near-Cache-Latency Signaling**
Utilizes shared-memory pointers and atomic flags to achieve signaling latencies in the **20ns - 100ns** range, bypassing bloated network stacks.

### 🔹 **2. Native Silicon Interface (Direct FFI)**
Binds directly to CUDA/Metal APIs via Rust FFI. We do not use intermediate Python or high-level runtime wrappers, ensuring a **minimized-copy** data path.

### 🔹 **3. Hardware-Constrained Decoding**
Enforces structural output (JSON/Schema) by hard-masking hardware registers during the sampling phase. This physically prevents token deviations that lead to structural hallucinations.

---

## 📊 **Platform & Silicon Matrix**

| OS Target | Architecture | Backend | Status |
| :--- | :--- | :--- | :--- |
| **Windows** | x86_64 | CUDA / Vulkan | ✅ Alpha |
| **Linux** | x86_64 | CUDA / ROCm | ✅ Alpha |
| **macOS** | Apple Silicon | Metal / MPS | ✅ Alpha |
| **Android** | ARM64 | Neon / OpenCL | 🧪 Experimental |
| **iOS** | ARM64 | Metal Native | 🧪 Experimental |

---

## 📜 **License & Compliance**

Cluaiz is governed by the **Cluaiz Industrial License (CSL) v1.0**:
- **Personal Use**: Free for individuals and startups under $10M revenue.
- **Institutional Standing**: Maintained by Cluaiz, a registered Micro Enterprise under the **Ministry of MSME, India** (Reg: UDYAM-UP-03-0131764).

---

### **Documentation**
- **[Architecture Deep-Dive](ARCHITECTURE.md)**
- **[Cluaiz Technical Specification](docs/cluaiz-technical-spec-v1.0.md)**
- **[Security Policy](SECURITY.md)**

---

<p align="center">
  <b>© 2026 Cluaiz. All Rights Reserved.</b><br>
  <i>"Built on Rust. Born on Silicon. Performance is Power."</i>
</p>