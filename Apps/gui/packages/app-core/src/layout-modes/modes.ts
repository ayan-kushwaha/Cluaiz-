import type { AppMode, LayoutModeConfig } from "@cluaiz/protocol";
import { createLayoutStore } from "../state/layout-store";

const layoutStore = createLayoutStore();

export const getModeLayout = (mode: AppMode): LayoutModeConfig => layoutStore.getModeConfig(mode);

export const modeDefaults: Record<AppMode, { left: boolean; right: boolean; header: boolean }> = {
  normal: {
    left: getModeLayout("normal").panels.left.visible,
    right: getModeLayout("normal").panels.right.visible,
    header: getModeLayout("normal").panels.header.visible
  },
  coding: {
    left: getModeLayout("coding").panels.left.visible,
    right: getModeLayout("coding").panels.right.visible,
    header: getModeLayout("coding").panels.header.visible
  },
  mobile_focus: {
    left: getModeLayout("mobile_focus").panels.left.visible,
    right: getModeLayout("mobile_focus").panels.right.visible,
    header: getModeLayout("mobile_focus").panels.header.visible
  }
};
