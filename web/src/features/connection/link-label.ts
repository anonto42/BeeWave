import type { AuthState } from "../../shared/auth";
import type { TransportState } from "../../shared/transport";

export type LinkTone = "off" | "warn" | "ok" | "bad";

/** One short line for the title bar: link state plus, once connected, where the key handshake stands. */
export function linkLabel(link: TransportState, auth: AuthState): { text: string; tone: LinkTone } {
  if (link === "disconnected") return { text: "disconnected", tone: "off" };
  if (link === "connecting") return { text: "connecting...", tone: "warn" };
  switch (auth) {
    case "authenticated":
      return { text: "connected", tone: "ok" };
    case "authenticating":
      return { text: "authenticating...", tone: "warn" };
    case "no-key":
      return { text: "needs key", tone: "warn" };
    case "failed":
      return { text: "wrong key", tone: "bad" };
    default:
      return { text: "connected", tone: "warn" };
  }
}
