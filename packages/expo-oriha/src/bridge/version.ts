import ExpoOrihaModule from "./ExpoOrihaModule";
import type { NativeBridgeVersionInfo } from "../types";

export async function nativeBridgeVersionInfo(): Promise<NativeBridgeVersionInfo> {
  return ExpoOrihaModule.nativeBridgeVersionInfo();
}
