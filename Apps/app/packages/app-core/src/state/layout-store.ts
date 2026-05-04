import type { AppMode, LayoutModeConfig, PanelKey, PanelState } from "@cluaiz/protocol";

const STORAGE_KEY = "cluaiz.layout.config.v1";

const defaultPanelState = (visible: boolean, width: number): PanelState => ({
  visible,
  collapsed: false,
  width,
  pinned: true
});

const defaultsByMode: Record<AppMode, LayoutModeConfig> = {
  normal: {
    mode: "normal",
    panels: {
      header: defaultPanelState(true, 0),
      left: defaultPanelState(true, 280),
      center: defaultPanelState(true, 0),
      right: defaultPanelState(true, 360),
      bottom: defaultPanelState(false, 0)
    },
    leftDrawerOpen: false,
    rightDrawerOpen: false
  },
  coding: {
    mode: "coding",
    panels: {
      header: defaultPanelState(true, 0),
      left: defaultPanelState(true, 320),
      center: defaultPanelState(true, 0),
      right: defaultPanelState(true, 380),
      bottom: defaultPanelState(false, 0)
    },
    leftDrawerOpen: false,
    rightDrawerOpen: false
  },
  mobile_focus: {
    mode: "mobile_focus",
    panels: {
      header: defaultPanelState(true, 0),
      left: defaultPanelState(false, 280),
      center: defaultPanelState(true, 0),
      right: defaultPanelState(false, 320),
      bottom: defaultPanelState(false, 0)
    },
    leftDrawerOpen: false,
    rightDrawerOpen: false
  }
};

const clone = (value: Record<AppMode, LayoutModeConfig>) => JSON.parse(JSON.stringify(value)) as Record<AppMode, LayoutModeConfig>;

const read = (): Record<AppMode, LayoutModeConfig> => {
  if (typeof window === "undefined") {
    return clone(defaultsByMode);
  }

  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) return clone(defaultsByMode);

  try {
    return { ...clone(defaultsByMode), ...(JSON.parse(raw) as Record<AppMode, LayoutModeConfig>) };
  } catch {
    return clone(defaultsByMode);
  }
};

const write = (value: Record<AppMode, LayoutModeConfig>): void => {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
};

export interface LayoutActions {
  setMode: (mode: AppMode) => LayoutModeConfig;
  togglePanel: (mode: AppMode, panel: PanelKey) => LayoutModeConfig;
  resizePanel: (mode: AppMode, panel: PanelKey, width: number) => LayoutModeConfig;
  setDrawerState: (mode: AppMode, side: "left" | "right", open: boolean) => LayoutModeConfig;
  persistLayout: () => void;
}

export interface LayoutStore extends LayoutActions {
  getModeConfig: (mode: AppMode) => LayoutModeConfig;
  getCurrentMode: () => AppMode;
}

export const createLayoutStore = (initialMode: AppMode = "normal"): LayoutStore => {
  let state = read();
  let currentMode = initialMode;

  const getModeConfig = (mode: AppMode): LayoutModeConfig => {
    return state[mode];
  };

  const updateMode = (mode: AppMode, next: LayoutModeConfig): LayoutModeConfig => {
    state = { ...state, [mode]: next };
    write(state);
    return next;
  };

  return {
    getModeConfig,
    getCurrentMode: () => currentMode,

    setMode: (mode: AppMode) => {
      currentMode = mode;
      return getModeConfig(mode);
    },

    togglePanel: (mode: AppMode, panel: PanelKey) => {
      const prev = getModeConfig(mode);
      const next = {
        ...prev,
        panels: {
          ...prev.panels,
          [panel]: {
            ...prev.panels[panel],
            visible: !prev.panels[panel].visible
          }
        }
      };
      return updateMode(mode, next);
    },

    resizePanel: (mode: AppMode, panel: PanelKey, width: number) => {
      const prev = getModeConfig(mode);
      const next = {
        ...prev,
        panels: {
          ...prev.panels,
          [panel]: {
            ...prev.panels[panel],
            width: Math.max(220, Math.min(width, 520))
          }
        }
      };
      return updateMode(mode, next);
    },

    setDrawerState: (mode: AppMode, side: "left" | "right", open: boolean) => {
      const prev = getModeConfig(mode);
      const next: LayoutModeConfig = {
        ...prev,
        leftDrawerOpen: side === "left" ? open : prev.leftDrawerOpen,
        rightDrawerOpen: side === "right" ? open : prev.rightDrawerOpen
      };
      return updateMode(mode, next);
    },

    persistLayout: () => {
      write(state);
    }
  };
};
