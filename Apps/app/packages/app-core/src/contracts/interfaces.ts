import type { AppMode, Capability, PermissionState } from "@cluaiz/protocol";

export interface PermissionBroker {
  request(capability: Capability): Promise<PermissionState>;
  status(capability: Capability): Promise<PermissionState>;
  revoke(capability: Capability): Promise<PermissionState>;
}

export interface AppCoreState {
  mode: AppMode;
  activeWorkspaceId: string | null;
}
