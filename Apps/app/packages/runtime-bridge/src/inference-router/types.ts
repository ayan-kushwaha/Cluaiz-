export type InferenceRoute = "local" | "cloud";

export interface InferenceDecision {
  route: InferenceRoute;
  reason: string;
}
