import React from "react";

export interface PanelFrameProps {
  title: string;
  children?: React.ReactNode;
  className?: string;
}

export function PanelFrame({ title, children, className = "" }: PanelFrameProps): JSX.Element {
  return (
    <section className={className} style={{ border: "1px solid var(--border)", background: "var(--card)", minHeight: "68vh", padding: "var(--panel-padding)", borderRadius: "var(--radius-panel)", boxShadow: "var(--panel-shadow)", transition: "all var(--motion-duration) ease" }}>
      <h3 style={{ marginTop: 0, marginBottom: 8 }}>{title}</h3>
      {children}
    </section>
  );
}
