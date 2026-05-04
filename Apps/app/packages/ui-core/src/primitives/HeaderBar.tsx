import React from "react";

export interface HeaderBarProps {
  left: React.ReactNode;
  right: React.ReactNode;
}

export function HeaderBar({ left, right }: HeaderBarProps): JSX.Element {
  return (
    <header style={{ borderBottom: "1px solid var(--border)", padding: "12px 14px", display: "flex", alignItems: "center", justifyContent: "space-between", backdropFilter: "blur(8px)", background: "rgba(8, 12, 20, 0.5)" }}>
      {left}
      {right}
    </header>
  );
}
