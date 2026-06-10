import type { Capability } from "@cluaiz/protocol";

export interface CapabilityDescriptor {
  capability: Capability;
  supported: boolean;
  permissionRequired: boolean;
}

export const capabilityRegistry: CapabilityDescriptor[] = [
  { capability: "camera", supported: true, permissionRequired: true },
  { capability: "filesystem", supported: true, permissionRequired: true },
  { capability: "notifications", supported: true, permissionRequired: true },
  { capability: "browser", supported: true, permissionRequired: false },
  { capability: "device", supported: true, permissionRequired: false },
  { capability: "sensors", supported: true, permissionRequired: true }
];
