import type { Telemetry } from "../protocol";

/** Pairs tracked at once (mirrors the device's own bound, `handshake::CAPACITY`). */
const CAPACITY = 32;

export interface HandshakeMessage {
  message: number;
  replayCounter: number;
  truncated: boolean;
  eapol: Uint8Array;
  at: number;
}

export interface HandshakePair {
  bssid: string;
  client: string;
  /** Keyed by message number (1-4); the latest frame of each. */
  messages: Map<number, HandshakeMessage>;
  complete: boolean;
  firstSeen: number;
  lastSeen: number;
}

const key = (bssid: string, client: string) => `${bssid}|${client}`;

/**
 * What the passive handshake listener has reported: on/off state and the captured pairs. A pair holds
 * the raw EAPOL bytes of whichever messages were seen, for the hex export; nothing here is sent anywhere.
 */
export class HandshakeState {
  enabled = false;
  private pairs = new Map<string, HandshakePair>();
  private order: string[] = []; // insertion order, for bounding
  private subs: Array<() => void> = [];

  constructor(private readonly now: () => number = Date.now) {}

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  list(): HandshakePair[] {
    return [...this.pairs.values()].sort((a, b) => b.lastSeen - a.lastSeen);
  }

  ingest(m: Telemetry): void {
    if (m.type === "handshakeCaptureState") {
      this.enabled = m.enabled;
      if (!m.enabled) {
        this.pairs.clear();
        this.order = [];
      }
    } else if (m.type === "handshakeFrame") {
      const k = key(m.bssid, m.client);
      let pair = this.pairs.get(k);
      if (!pair) {
        if (this.order.length >= CAPACITY) {
          const oldest = this.order.shift();
          if (oldest) this.pairs.delete(oldest);
        }
        pair = { bssid: m.bssid, client: m.client, messages: new Map(), complete: false, firstSeen: this.now(), lastSeen: this.now() };
        this.pairs.set(k, pair);
        this.order.push(k);
      }
      pair.messages.set(m.message, { message: m.message, replayCounter: m.replayCounter, truncated: m.truncated, eapol: m.eapol, at: this.now() });
      pair.lastSeen = this.now();
    } else if (m.type === "handshakeComplete") {
      const pair = this.pairs.get(key(m.bssid, m.client));
      if (pair) pair.complete = true;
    } else return;
    this.subs.forEach((f) => f());
  }
}

const toHex = (b: Uint8Array) => [...b].map((x) => x.toString(16).padStart(2, "0")).join(" ");

/** A readable hex dump of one pair's captured messages, in message order, each flagged if truncated. */
export function pairToHex(p: HandshakePair): string {
  const lines = [`# ${p.bssid} <-> ${p.client}${p.complete ? " (complete)" : ""}`];
  for (const m of [...p.messages.values()].sort((a, b) => a.message - b.message)) {
    lines.push(`## message ${m.message}${m.truncated ? " (truncated: longer than the capture cap)" : ""}`);
    lines.push(toHex(m.eapol));
  }
  return lines.join("\n") + "\n";
}
