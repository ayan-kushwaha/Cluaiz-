import React from "react";
import type { ThemeTokenSet } from "@cluaiz/protocol";

export interface TokenEditorProps {
  tokens: ThemeTokenSet;
  onChange: (tokens: ThemeTokenSet) => void;
}

export function TokenEditor({ tokens, onChange }: TokenEditorProps): JSX.Element {
  const update = <K extends keyof ThemeTokenSet>(key: K, value: ThemeTokenSet[K]) => {
    onChange({ ...tokens, [key]: value });
  };

  return (
    <div style={{ display: "grid", gap: 10, fontSize: 13 }}>
      <label style={{ display: "grid", gap: 6 }}>
        Accent (HSL)
        <input
          className="input"
          value={tokens.color.accent}
          onChange={(e) => update("color", { ...tokens.color, accent: e.currentTarget.value })}
        />
      </label>

      <label style={{ display: "grid", gap: 6 }}>
        Font Scale
        <input type="range" min={0.85} max={1.3} step={0.01} value={tokens.typography.fontScale} onChange={(e) => update("typography", { ...tokens.typography, fontScale: Number(e.currentTarget.value) })} />
      </label>

      <label style={{ display: "grid", gap: 6 }}>
        Spacing Base
        <input type="range" min={6} max={14} step={1} value={tokens.spacing.base} onChange={(e) => update("spacing", { base: Number(e.currentTarget.value) })} />
      </label>

      <label style={{ display: "grid", gap: 6 }}>
        Radius
        <input type="range" min={8} max={24} step={1} value={tokens.radius.panel} onChange={(e) => update("radius", { panel: Number(e.currentTarget.value) })} />
      </label>

      <label style={{ display: "grid", gap: 6 }}>
        Motion Intensity
        <input type="range" min={0.5} max={1.8} step={0.1} value={tokens.motion.intensity} onChange={(e) => update("motion", { ...tokens.motion, intensity: Number(e.currentTarget.value) })} />
      </label>
    </div>
  );
}
