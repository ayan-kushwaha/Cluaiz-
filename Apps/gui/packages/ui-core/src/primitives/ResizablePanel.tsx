import React from "react";

export interface ResizablePanelProps {
  width: number;
  min?: number;
  max?: number;
  onWidthChange: (width: number) => void;
  children?: React.ReactNode;
  edge?: "left" | "right";
}

export function ResizablePanel({
  width,
  min = 220,
  max = 520,
  onWidthChange,
  children,
  edge = "right"
}: ResizablePanelProps): JSX.Element {
  const [dragging, setDragging] = React.useState(false);

  React.useEffect(() => {
    if (!dragging) return;

    const onMove = (event: MouseEvent) => {
      const viewportWidth = window.innerWidth;
      if (edge === "right") {
        onWidthChange(Math.min(max, Math.max(min, event.clientX)));
      } else {
        onWidthChange(Math.min(max, Math.max(min, viewportWidth - event.clientX)));
      }
    };

    const onUp = () => setDragging(false);

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
  }, [dragging, edge, max, min, onWidthChange]);

  return (
    <div style={{ width, position: "relative" }}>
      {children}
      <div
        onMouseDown={() => setDragging(true)}
        style={{
          position: "absolute",
          top: 0,
          bottom: 0,
          width: 6,
          cursor: "col-resize",
          right: edge === "right" ? -6 : undefined,
          left: edge === "left" ? -6 : undefined,
          background: dragging ? "rgba(79,209,255,0.45)" : "transparent"
        }}
      />
    </div>
  );
}
