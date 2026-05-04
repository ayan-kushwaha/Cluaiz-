import type { ThemeProfile, ThemeTokenSet } from "@cluaiz/protocol";
import { cloneTheme, presetThemes } from "./theme-presets";

const STORAGE_KEY = "cluaiz.theme.profiles.v1";
const ACTIVE_KEY = "cluaiz.theme.active.v1";

export interface ThemeStudioActions {
  apply: (profileId: string) => ThemeProfile;
  preview: (tokens: ThemeTokenSet) => void;
  saveProfile: (name: string, tokens: ThemeTokenSet) => ThemeProfile;
  resetProfile: (profileId: string) => ThemeProfile;
  exportProfiles: () => string;
  importProfiles: (raw: string) => ThemeProfile[];
}

export interface ThemeStore extends ThemeStudioActions {
  listProfiles: () => ThemeProfile[];
  getActiveProfile: () => ThemeProfile;
}

const hasWindow = () => typeof window !== "undefined";

const read = (): ThemeProfile[] => {
  if (!hasWindow()) {
    return presetThemes.map(cloneTheme);
  }

  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return presetThemes.map(cloneTheme);
  }

  try {
    const parsed = JSON.parse(raw) as ThemeProfile[];
    return parsed.length > 0 ? parsed : presetThemes.map(cloneTheme);
  } catch {
    return presetThemes.map(cloneTheme);
  }
};

const write = (profiles: ThemeProfile[]): void => {
  if (!hasWindow()) return;
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(profiles));
};

const readActiveId = (): string => {
  if (!hasWindow()) return presetThemes[0].id;
  return window.localStorage.getItem(ACTIVE_KEY) || presetThemes[0].id;
};

const writeActiveId = (id: string): void => {
  if (!hasWindow()) return;
  window.localStorage.setItem(ACTIVE_KEY, id);
};

export const applyThemeTokensToDocument = (tokens: ThemeTokenSet): void => {
  if (!hasWindow()) return;
  const root = document.documentElement;
  root.style.setProperty("--background", tokens.color.background);
  root.style.setProperty("--foreground", tokens.color.foreground);
  root.style.setProperty("--card", tokens.color.card);
  root.style.setProperty("--border", tokens.color.border);
  root.style.setProperty("--accent", tokens.color.accent);
  root.style.setProperty("--font-family", tokens.typography.fontFamily);
  root.style.setProperty("--font-scale", String(tokens.typography.fontScale));
  root.style.setProperty("--space-base", `${tokens.spacing.base}px`);
  root.style.setProperty("--radius-panel", `${tokens.radius.panel}px`);
  root.style.setProperty("--panel-shadow", tokens.elevation.panelShadow);
  root.style.setProperty("--motion-duration", `${tokens.motion.durationMs}ms`);
  root.style.setProperty("--motion-intensity", String(tokens.motion.intensity));
  root.style.setProperty("--panel-padding", `${tokens.panelDensity.panelPadding}px`);
};

export const createThemeStore = (): ThemeStore => {
  let profiles = read();
  let activeId = readActiveId();

  const findOrDefault = (id: string): ThemeProfile => {
    return profiles.find((p) => p.id === id) || profiles[0] || presetThemes[0];
  };

  const touch = (profile: ThemeProfile): ThemeProfile => ({
    ...profile,
    updatedAt: new Date().toISOString()
  });

  return {
    listProfiles: () => profiles.map(cloneTheme),
    getActiveProfile: () => cloneTheme(findOrDefault(activeId)),

    apply: (profileId: string) => {
      const profile = findOrDefault(profileId);
      activeId = profile.id;
      writeActiveId(activeId);
      applyThemeTokensToDocument(profile.tokens);
      return cloneTheme(profile);
    },

    preview: (tokens: ThemeTokenSet) => {
      applyThemeTokensToDocument(tokens);
    },

    saveProfile: (name: string, tokens: ThemeTokenSet) => {
      const id = `custom-${name.toLowerCase().replace(/[^a-z0-9]+/g, "-")}-${Date.now()}`;
      const profile = touch({ id, name, isPreset: false, updatedAt: "", tokens });
      profiles = [profile, ...profiles];
      write(profiles);
      return cloneTheme(profile);
    },

    resetProfile: (profileId: string) => {
      const preset = presetThemes.find((p) => p.id === profileId);
      if (!preset) {
        return cloneTheme(findOrDefault(activeId));
      }

      profiles = profiles.map((p) => (p.id === profileId ? cloneTheme(preset) : p));
      write(profiles);
      return cloneTheme(preset);
    },

    exportProfiles: () => JSON.stringify(profiles),

    importProfiles: (raw: string) => {
      try {
        const parsed = JSON.parse(raw) as ThemeProfile[];
        if (!Array.isArray(parsed) || parsed.length === 0) {
          return profiles.map(cloneTheme);
        }
        profiles = parsed;
        write(profiles);
        return profiles.map(cloneTheme);
      } catch {
        return profiles.map(cloneTheme);
      }
    }
  };
};
