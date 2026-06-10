import type { ThemeProfile, ThemeTokenSet } from "@cluaiz/protocol";

const now = () => new Date().toISOString();

export const defaultThemeTokens: ThemeTokenSet = {
  color: {
    background: "220 28% 8%",
    foreground: "210 40% 96%",
    card: "220 24% 12%",
    border: "219 23% 25%",
    accent: "190 90% 55%"
  },
  typography: {
    fontFamily: "'Segoe UI', system-ui, sans-serif",
    fontScale: 1
  },
  spacing: {
    base: 8
  },
  radius: {
    panel: 12
  },
  elevation: {
    panelShadow: "0 10px 30px rgba(0,0,0,0.25)"
  },
  motion: {
    durationMs: 220,
    intensity: 1
  },
  panelDensity: {
    panelPadding: 12
  }
};

export const presetThemes: ThemeProfile[] = [
  {
    id: "midnight-core",
    name: "Midnight Core",
    isPreset: true,
    updatedAt: now(),
    tokens: defaultThemeTokens
  },
  {
    id: "ice-light",
    name: "Ice Light",
    isPreset: true,
    updatedAt: now(),
    tokens: {
      ...defaultThemeTokens,
      color: {
        background: "220 20% 96%",
        foreground: "222 28% 12%",
        card: "0 0% 100%",
        border: "220 18% 86%",
        accent: "198 90% 45%"
      }
    }
  }
];

export const cloneTheme = (theme: ThemeProfile): ThemeProfile =>
  JSON.parse(JSON.stringify(theme)) as ThemeProfile;
