import type { ThemeProfile } from "../commands/types";

export interface ThemeUpdatedV1 {
  type: "theme.updated.v1";
  profile: ThemeProfile;
  timestamp: string;
}
