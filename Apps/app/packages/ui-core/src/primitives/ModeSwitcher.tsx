import React from "react";
import type { AppMode } from "@cluaiz/protocol";

export interface ModeSwitcherProps {
  mode: AppMode;
  onChange: (mode: AppMode) => void;
}

export function ModeSwitcher({ mode, onChange }: ModeSwitcherProps): JSX.Element {
  const options: AppMode[] = ["normal", "coding", "mobile_focus"];
  return (
    <div style={{ display: "flex", gap: 8 }}>
      {options.map((option) => (
        <button
          key={option}
          style={{
            border: "1px solid var(--border)",
            borderRadius: 10,
            padding: "7px 11px",
            cursor: "pointer",
            background: mode === option ? "linear-gradient(180deg, #59d7ff 0%, #33bdf0 100%)" : "linear-gradient(180deg, #19263d 0%, #152036 100%)",
            color: mode === option ? "#04111a" : "var(--foreground)"
          }}
          onClick={() => onChange(option)}
        >
          {option}
        </button>
      ))}
    </div>
  );
}
