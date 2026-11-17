import type { AuthState } from "../../shared/auth";
import type { TransportState } from "../../shared/transport";

export type ControlMode = "OFFLINE" | "AUTHENTICATING" | "LOCKED" | "IDLE" | "SCANNING";

export interface ControlState {
  readonly mode: ControlMode;
  /** The one scan button: starts a scan while idle, stops it while scanning. */
  readonly run: { readonly enabled: boolean; readonly label: string };
  readonly channelEnabled: boolean;
}

/**
 * What the controls show. The device ignores every command until the key handshake succeeds, so the
 * controls stay off until then and the badge says why (instead of buttons that silently do nothing).
 */
export function controlState(link: TransportState, auth: AuthState, scanning: boolean): ControlState {
  const mode: ControlMode =
    link !== "connected"
      ? "OFFLINE"
      : auth === "authenticating"
        ? "AUTHENTICATING"
        : auth !== "authenticated"
          ? "LOCKED"
          : scanning
            ? "SCANNING"
            : "IDLE";
  const usable = mode === "IDLE" || mode === "SCANNING";
  return { mode, run: { enabled: usable, label: scanning ? "STOP SCAN" : "START SCAN" }, channelEnabled: usable };
}
