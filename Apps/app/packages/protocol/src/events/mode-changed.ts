import type { AppMode } from "../commands/types";

export interface ModeChangedV1 {
  type: "mode.changed.v1";
  mode: AppMode;
  timestamp: string;
}
