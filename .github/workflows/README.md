# 🏛️ CLUAIZ-OS: THE SOVEREIGN SYSTEM DESIGN

This document serves as the **Single Source of Truth** for the Cluaiz Neural Ecosystem. It defines the architectural DNA, industrial standards, and universal deployment laws that govern every line of code in this repository.

---

## 🛰️ 1. THE VISION: UNIVERSAL NEURAL SOVEREIGNTY
Cluaiz-OS is designed to be a **Universal Neural Kernel**. Our mission is to provide high-performance, native inference for any model, on any silicon, under any operating system, eliminating hardware boundaries.

### 🚀 Core Directives:
- **Silicon Mastery**: Extract peak performance from CPU, GPU, NPU, and TPU natively.
- **Hardware Agnosticism**: Unified execution across Windows, Linux, Android, iOS, and macOS.
- **Modular Integrity**: Decoupled architecture where components communicate via standardized FFI handshakes.

---

## 🧬 2. MODULAR ARCHITECTURE (THE NEURAL STACK)

The ecosystem is divided into four sovereign layers. Any change to one layer MUST NOT break the handshake of others.

| Layer | Component | Responsibility |
| :--- | :--- | :--- |
| **Edge** | `cluaiz-cli` | User interface, model management, and system orchestration. |
| **Brain** | `cluaiz-engine` | The core orchestrator. Manages memory, context, and driver dispatch. |
| **Kernel** | `archer-llama` | The inference engine. Links natively with modular GGML/Llama components. |
| **Bridge** | `cluaiz-drivers` | Hardware-specific native drivers (CUDA, Vulkan, Metal, CPU). |

---

## 🛰️ 3. THE UNIVERSAL MATRIX LAW (5-OS SUPPORT)
Cluaiz-OS MUST run everywhere. The CI/CD pipeline is mandated to support:

- **Windows**: x64 (Desktop/Surface), x86 (Industrial/Legacy).
- **Linux**: x86_64 (Server), aarch64 (Cloud/Edge), armv7 (IoT).
- **Android**: aarch64 (Mobile/Tablet/Auto).
- **macOS**: arm64 (Apple Silicon), x86_64 (Intel Mac).
- **iOS**: arm64 (iPhone/iPad).

---

## 💎 4. THE NAMING & VERSIONING CONSTITUTION
To ensure zero-latency binary mapping, all artifacts MUST follow the **Sovereign Naming Convention**:

### 📦 Binary Naming Pattern:
`cluaiz-<component>-<version>-<platform>-<backend>.<ext>`

- **Component**: `cli`, `engine`, `llama`, `driver`.
- **Platform**: `win-x64`, `linux-arm64`, `android`, etc.
- **Backend**: `cpu`, `cuda`, `metal`, `vulkan`.
- **Version**: Strictly follows the git tag `v*`.

---

## ⚡ 5. CI/CD PIPELINE INTEGRITY (ZERO-CRASH DEPLOYMENT)
The GitHub Actions pipeline is an industrial foundry. It MUST follow these mandatory stages:

### 🛠️ Stage 1: Native Compilation
- **Multi-Silicon Strike**: Parallel builds for all architectures.
- **Static Linking**: Force `/MT` on Windows and `musl` on Linux where possible for maximum portability.

### 🔬 Stage 2: Sovereign Audit
- **Handshake Verification**: Every binary is scanned for exported symbols (`nm` / `dumpbin`) to ensure FFI compatibility.
- **Integrity Sealing**: Generation of SHA-256 hashes for every artifact.

### 🚀 Stage 3: Hyper-Release
- **Dynamic Release**: Artifacts are uploaded to a release tagged exactly with `${{ github.ref_name }}`.
- **Registry Sync**: The `registry.json` is updated with the new download URLs to trigger the CLI auto-updater.

---

## 🏛️ 6. THE FOUNDER'S MANDATE
1. **Never Drift**: Do not change naming conventions or matrix structures once established.
2. **Standard over Ad-hoc**: Every fix must be architectural, not a "Kach-Khas" (quick-fix).
3. **Total Coverage**: A build is only successful if ALL platforms in the matrix pass.

**This is the Cluaiz Standard. Professional. Optimized. Sovereign.**  
