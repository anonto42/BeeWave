import type { AuthState } from "../../shared/auth";
import type { TransportState } from "../../shared/transport";

/** Why the status cells may be empty. "" = nothing to explain. */
export function statusNote(link: TransportState, auth: AuthState, gotStatus: boolean): string {
  if (link === "disconnected") return "Not connected. Tap CONNECT in the top bar.";
  if (link === "connecting") return "Connecting...";
  if (auth === "no-key") return "Locked: this device is already claimed. Paste its key in Settings > Device Key, or reset the device with make fw-unclaim.";
  if (auth === "failed") return "Locked: the device rejected the key. Fix it in Settings > Device Key.";
  if (auth !== "authenticated") return "Authenticating...";
  return gotStatus ? "" : "Waiting for the first status...";
}
