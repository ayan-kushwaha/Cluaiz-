# 🏛️ Cluaiz Nebula Library: Sovereign Registry Rules
**Version: 1.0** | **Status: Industrial Standard**

This document defines the rigid architecture for adding and managing models within the Cluaiz Nebula Registry. Every model family must adhere to these standards to ensure a "Makkhan" (Smooth) experience across the CLI, Mobile App, and Web.

---

## 📂 1. Folder Hierarchy Protocol
Every model must be organized in the following surgical structure:
```
library/
└── [model-family]/
    ├── logo.png            <-- (Required) Small square icon
    ├── poster.webp         <-- (Optional) Large banner image
    └── v-[version]/
        ├── manifest.json   <-- (Required) Sovereign metadata
        ├── README.md       <-- (Required) The Identity file
        └── assets/        <-- (Required) Fig/Demo assets
```

---

## 📜 2. The Identity (README.md) Standard
Every `README.md` within a version folder must follow this exact visual flow:

1. **Top Header**: Logo and Model Name in H1.
2. **Title**: A catchy, 1-line professional description.
3. **Sovereign Command**: A code block for the `cluaiz run` command.
4. **Model Matrix Table**: A table containing:
   - Version, Parameters, Architecture, Memory (VRAM), and Download Size.
5. **Visual Assets**: The `poster.webp` (if available) followed by Benchmarks/Figures from the `assets/` folder.
6. **Detailed Narrative**: Rich markdown paragraphs explaining the model's "Soul" (Reasoning, Use-cases, Training).

---

## 🎨 3. Asset Specifications
- **Logos**: Must be transparent PNG (min 512x512).
- **Posters**: Must be high-quality WebP (16:9 ratio).
- **Benchmarks**: Figures must be clear PNGs with descriptive captions.
- **Demos**: Video files must be optimized MP4/WebM.

---

## ⚖️ 4. Sovereign Content Rules
- **No Copyright Infringement**: All images and text must be authorized for use within the Cluaiz ecosystem.
- **Hardware Aware**: Descriptions must mention the recommended hardware (GPU/NPU/CPU).
- **SEO Ready**: Use appropriate headers and keywords (1-bit, LLM, Inference) to ensure search visibility on the Next.js frontend.

---

**Failure to comply with these rules will result in the model being marked as "Non-Sovereign" and hidden from the registry.** 🧿
