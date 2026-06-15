# 💠 ONNX Backend (`interface-engines/onnx/`)

<p align="center"><strong>The ONNX Runtime Execution Engine</strong></p>

---

## 🎯 Deep Purpose

The `onnx/` crate provides execution support for the Open Neural Network Exchange (`.onnx`) format via Microsoft's ONNX Runtime. This backend is primarily used for deterministic, highly optimized enterprise models or specific Vision/Audio models that do not fit into the GGUF text-generation paradigm.

## 🏛️ Architectural Mechanics
- **The Core Logic:** Connects the `dispatcher` to the `libonnxruntime` C-API, mapping Cluaize inputs into ONNX execution graphs.
- **The "Why":** While GGUF is excellent for quantized LLMs, ONNX provides superior cross-platform execution for specialized embedding models and standard deep learning classifiers.
