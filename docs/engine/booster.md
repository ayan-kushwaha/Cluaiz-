# System Booster

The **System Booster** is the core acceleration sub-system of `cluaiz-engine`. It operates as a bare-metal hardware optimizer, executing specialized layers to maximize inference speed and compression without losing precision.

---

## Architectural Layout

The booster splits optimization tasks across highly isolated modules:

### 1. Governance Layer (`manager/`)
Decides which optimizations are safe to execute based on active hardware limits:
*   **Conflict Resolver (`conflict_resolver.rs`):** The logical arbiter that blocks incompatible features from loading (e.g., preventing speculative decoding from running on Low-VRAM environments to prevent crash/OOM).
*   **Auto Tuner (`auto_tuner.rs`):** The silicon sniper that checks host hardware profiles and adjusts settings dynamically for NVIDIA, Apple, or Qualcomm chips.

### 2. Speculative Decoding (`dflash/`)
Accelerates token output by generating drafts and verifying them in parallel:
*   **DDTree Algorithm:** A verification pass validating multiple draft tokens simultaneously.
*   **Asymmetric KV Caching:** Couples varying precision matrices (K=TQ3_0, V=F16) to conserve memory.

### 3. Precision Quantization (`turbo_quant/`)
Compresses weights to extreme limits using high-fidelity math algorithms:
*   **Polar Coordinate Weights:** Polar rotation vectors to ensure 2nd-order weight error correction.
*   **Hadamard & Givens Rotations:** Rotational matrices to reduce feature decorrelation during quantization.
*   **SIMD Acceleration Kernels:** Native low-level instruction kernels compiled directly for Intel AVX512, Apple AMX, and ARM Neon.

### 4. Attention & Fusions (`flash_attn/` & `auto_round/`)
*   **Flash Attention:** IO-aware sliding-window and paged attention algorithms to reduce memory-bus transfers.
*   **Auto Round:** 2nd-order weight rounding processes to optimize 3-bit and 4-bit quantization outputs.
