import React from "react";

export interface DrawerProps {
  side: "left" | "right";
  open: boolean;
  onClose: () => void;
  children?: React.ReactNode;
}

export function Drawer({ side, open, onClose, children }: DrawerProps): JSX.Element | null {
  if (!open) return null;

  return (
    <div style={{ position: "fixed", inset: 0, zIndex: 40 }}>
      <button style={{ position: "absolute", inset: 0, background: "rgba(0,0,0,0.5)", border: "none" }} onClick={onClose} aria-label="Close drawer" />
      <aside
        style={{
          position: "absolute",
          top: 0,
          [side]: 0,
          height: "100%",
          width: "min(84vw, 420px)",
          background: "var(--card)",
          border: "1px solid var(--border)",
          padding: 14,
          transition: "transform var(--motion-duration) ease"
        }}
      >
        {children}
      </aside>
    </div>
  );
}
