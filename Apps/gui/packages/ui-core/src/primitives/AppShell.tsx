import React from "react";

export interface AppShellProps {
  header?: React.ReactNode;
  body: React.ReactNode;
}

export function AppShell({ header, body }: AppShellProps): JSX.Element {
  const hasHeader = Boolean(header);
  return (
    <div
      style={{
        minHeight: "100vh",
        display: "grid",
        gridTemplateRows: hasHeader ? "auto 1fr" : "1fr",
        background: "var(--background)",
        color: "var(--foreground)",
        fontFamily: "var(--font-family)"
      }}
    >
      {hasHeader ? header : null}
      {body}
    </div>
  );
}
