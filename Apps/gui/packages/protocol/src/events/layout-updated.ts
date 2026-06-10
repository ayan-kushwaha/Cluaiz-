import type { LayoutModeConfig } from "../commands/types";

export interface LayoutUpdatedV1 {
  type: "layout.updated.v1";
  config: LayoutModeConfig;
  timestamp: string;
}
