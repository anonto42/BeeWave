import type { Telemetry } from "../protocol";

const MAX_CLIENTS = 64;

export interface DiscoveredClient {
  client: string;
  channel: number;
  rssi: number;
  probedSsids: string[];
  firstSeen: number;
  lastSeen: number;
  probeCount: number;
}

/**
 * Stores passively intercepted 802.11 Probe Request observations from client stations.
 * Correlates probed SSIDs (Preferred Network Lists) and tracks signal strength.
 */
export class ProbeStore {
  private clients = new Map<string, DiscoveredClient>();
  private order: string[] = []; // Insertion order for bounding
  private subs: Array<() => void> = [];

  constructor(private readonly now: () => number = Date.now) {}

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  list(): DiscoveredClient[] {
    return [...this.clients.values()].sort((a, b) => b.lastSeen - a.lastSeen);
  }

  clear(): void {
    this.clients.clear();
    this.order = [];
    this.notify();
  }

  ingest(m: Telemetry): void {
    if (m.type !== "probeResult") return;

    let c = this.clients.get(m.client);
    const ts = this.now();
    if (!c) {
      if (this.order.length >= MAX_CLIENTS) {
        const oldest = this.order.shift();
        if (oldest) this.clients.delete(oldest);
      }
      c = {
        client: m.client,
        channel: m.channel,
        rssi: m.rssi,
        probedSsids: m.ssid ? [m.ssid] : [],
        firstSeen: ts,
        lastSeen: ts,
        probeCount: 1,
      };
      this.clients.set(m.client, c);
      this.order.push(m.client);
    } else {
      c.channel = m.channel;
      c.rssi = m.rssi;
      c.lastSeen = ts;
      c.probeCount++;
      if (m.ssid && !c.probedSsids.includes(m.ssid)) {
        c.probedSsids.push(m.ssid);
      }
    }
    this.notify();
  }

  private notify(): void {
    this.subs.forEach((f) => f());
  }
}
