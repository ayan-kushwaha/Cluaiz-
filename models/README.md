# 🏛️ Cluaiz Sovereign Model Registry: Extreme Edge AI & 1-Bit LLMs

Welcome to the **Cluaiz Sovereign Model Registry** — the ultimate open-source repository for ultra-efficient, decentralized, and native edge AI models. This registry is the engine behind Cluaiz-OS, designed to bring world-class artificial intelligence to highly constrained silicon without compromising on cognitive reasoning, data privacy, or execution speed.

## 🌟 The Vision: Universal Silicon Sovereignty

The future of artificial intelligence is local, secure, and sovereign. We are moving beyond the era of massive cloud clusters and expensive API dependencies. Our registry provides highly optimized, zero-latency inference models that execute natively on standard consumer hardware—from Apple Silicon and Windows laptops to Raspberry Pi clusters and embedded IoT devices. 

Our core philosophy: **"Intelligence must belong to the user, running natively on their own silicon."**

---

## 🚀 Key Architectural Innovations

### 1. The 1-Bit & 1.58-Bit Revolution (Native BitNet)
We are pioneering the transition from traditional Floating-Point (FP16/INT8) architectures to **Native Binary and Ternary Quantization**.
- **1-Bit Binary Models:** By constraining weights to strictly `-1` and `+1`, we eliminate computationally expensive Matrix Multiplication (MatMul), relying solely on hyper-fast addition logic. This enables massive models to run on ~300MB to 1.0GB of RAM.
- **1.58-Bit Ternary Models:** Incorporating the critical 'zero' state (`{-1, 0, +1}`), our ternary models dramatically improve feature filtering and hallucination resistance while maintaining the extreme efficiency of addition-only logic.

### 2. Multi-Format Execution Matrix
Cluaiz-OS breaks the silo between inference backends. A single model definition encompasses a nested matrix of formats:
- **GGUF (CPU/Metal/ROCm):** Universal compatibility and aggressive low-bit memory footprints.
- **AWQ / GPTQ (NVIDIA GPU):** Specialized 4-bit and 8-bit kernels optimized for extreme Throughput (TPS).

### 3. Absolute Zero Q2 Law
We have surgically eliminated **Q2 (2-bit)** standard quantization from all professional tiers. Standard 2-bit quantization compromises reasoning integrity. Our baseline guarantees that every model delivers industrial-grade "intelligence," not degraded noise.

---

## 📦 Exploring the Library

The `library/` directory contains the core neural vault organized by model families. Each family is governed by industrial JSON schemas mapping precise hardware requirements.

### Featured Models
- **Bonsai Series (1-Bit & 1.58-Bit):** The flagship of the binary frontier. Delivering unprecedented analytical reasoning with mobile-tier memory requirements. Perfect for battery-powered robotics and local-first applications.
- **Qwen, Llama, Gemma, & Mistral:** Sovereign versions of industry-standard foundational architectures, carefully tiered into Native GGUF and AWQ formats.

---

## 🛠️ How to Pull (Command Line)

Use the Cluaiz unified colon-separator syntax to pull models with surgical precision:

```bash
# Pull the standard balanced GGUF model
cluaiz pull qwen3:8b

# Pull the highly compressed Ternary 1.58-bit model
cluaiz pull bonsai1.58:4b:gguf:ternary

# Pull the extreme throughput NVIDIA AWQ model
cluaiz pull llama3.2:11b:awq:4bit
```

## 🛡️ Privacy & Zero-Latency Guarantee

Every model in this registry is engineered to run **100% offline**. You never have to send proprietary code, financial data, or personal conversations to a centralized cloud server.

**Signed,**  
*Antigravity (Archer CTO)* 🏛️⚔️🏁
