export type AppMode = "normal" | "coding" | "mobile_focus";

export type Capability =
  | "camera"
  | "filesystem"
  | "notifications"
  | "browser"
  | "device"
  | "sensors";

export type PermissionState = "granted" | "denied" | "prompt" | "unsupported";

export type ThemeTokenColor = {
  background: string;
  foreground: string;
  card: string;
  border: string;
  accent: string;
};

export type ThemeTokenTypography = {
  fontFamily: string;
  fontScale: number;
};

export type ThemeTokenSpacing = {
  base: number;
};

export type ThemeTokenRadius = {
  panel: number;
};

export type ThemeTokenElevation = {
  panelShadow: string;
};

export type ThemeTokenMotion = {
  durationMs: number;
  intensity: number;
};

export type ThemeTokenDensity = {
  panelPadding: number;
};

export interface ThemeTokenSet {
  color: ThemeTokenColor;
  typography: ThemeTokenTypography;
  spacing: ThemeTokenSpacing;
  radius: ThemeTokenRadius;
  elevation: ThemeTokenElevation;
  motion: ThemeTokenMotion;
  panelDensity: ThemeTokenDensity;
}

export interface ThemeProfile {
  id: string;
  name: string;
  isPreset: boolean;
  updatedAt: string;
  tokens: ThemeTokenSet;
}

export type PanelKey = "header" | "left" | "center" | "right" | "bottom";

export interface PanelState {
  visible: boolean;
  collapsed: boolean;
  width: number;
  pinned: boolean;
}

export interface LayoutModeConfig {
  mode: AppMode;
  panels: Record<PanelKey, PanelState>;
  leftDrawerOpen: boolean;
  rightDrawerOpen: boolean;
}
