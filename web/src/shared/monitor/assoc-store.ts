import type { Telemetry } from "../protocol";

const MAX_ASSOCIATIONS = 64;

export interface ConnectedStation {
  client: string;
  bssid: string;
  channel: number;
  rssi: number;
  firstSeen: number;
  lastSeen: number;
  frameCount: number;
}

/**
 * Stores passively intercepted client-to-AP associations from 802.11 Data frame headers.
 * Maps connected stations to Access Points.
 */
export class AssocStore {
  private assocs = new Map<string, ConnectedStation>();
  private order: string[] = []; // Insertion order for bounding
  private subs: Array<() => void> = [];

  constructor(private readonly now: () => number = Date.now) {}

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  list(): ConnectedStation[] {
    return [...this.assocs.values()].sort((a, b) => b.lastSeen - a.lastSeen);
  }

  stationsFor(bssid: string): ConnectedStation[] {
    const keyBssid = bssid.toLowerCase();
    return [...this.assocs.values()]
      .filter((a) => a.bssid.toLowerCase() === keyBssid)
      .sort((a, b) => b.lastSeen - a.lastSeen);
  }

  associationFor(client: string): ConnectedStation | undefined {
    const keyClient = client.toLowerCase();
    return [...this.assocs.values()]
      .filter((a) => a.client.toLowerCase() === keyClient)
      .sort((a, b) => b.lastSeen - a.lastSeen)[0];
  }

  clear(): void {
    this.assocs.clear();
    this.order = [];
    this.notify();
  }

  ingest(m: Telemetry): void {
    if (m.type !== "assocResult") return;

    const pairKey = `${m.bssid.toLowerCase()}|${m.client.toLowerCase()}`;
    let a = this.assocs.get(pairKey);
    const ts = this.now();
    if (!a) {
      if (this.order.length >= MAX_ASSOCIATIONS) {
        const oldest = this.order.shift();
        if (oldest) this.assocs.delete(oldest);
      }
      a = {
        client: m.client.toLowerCase(),
        bssid: m.bssid.toLowerCase(),
        channel: m.channel,
        rssi: m.rssi,
        firstSeen: ts,
        lastSeen: ts,
        frameCount: 1,
      };
      this.assocs.set(pairKey, a);
      this.order.push(pairKey);
    } else {
      a.channel = m.channel;
      a.rssi = m.rssi;
      a.lastSeen = ts;
      a.frameCount++;
    }
    this.notify();
  }

  private notify(): void {
    for (const sub of this.subs) sub();
  }
}
