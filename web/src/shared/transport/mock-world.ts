import type { Telemetry } from "../protocol";
import { DEFAULT_CONFIG, type MonitorConfig } from "../monitor/monitor-state";
import type { ScanSettings } from "../protocol";

/** A tiny seeded PRNG (mulberry32) so the simulated world is repeatable in tests. */
export function seeded(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

interface Ap {
  bssid: string;
  ssid: string;
  channel: number;
  base: number;
  /** Scanning ticks before the AP exists (0 = always). */
  from: number;
}

const APS: Ap[] = [
  { bssid: "3c:71:bf:00:00:01", ssid: "BeeWave-lab", channel: 1, base: -48, from: 0 },
  { bssid: "fc:ec:da:11:22:01", ssid: "Office", channel: 1, base: -62, from: 0 },
  { bssid: "fc:ec:da:11:22:02", ssid: "Office", channel: 6, base: -70, from: 0 },
  { bssid: "50:c7:bf:aa:bb:01", ssid: "Home", channel: 6, base: -55, from: 0 },
  { bssid: "a0:21:b7:cc:dd:01", ssid: "Cafe", channel: 11, base: -74, from: 0 },
  { bssid: "02:11:22:33:44:55", ssid: "", channel: 4, base: -80, from: 0 },
  { bssid: "3c:37:12:de:ad:01", ssid: "FRITZ!Box 7490", channel: 9, base: -66, from: 0 },
  { bssid: "d4:6e:0e:00:00:09", ssid: "IoT", channel: 13, base: -78, from: 0 },
  // appears late with a SSID that already exists: the "new BSSID for a known SSID" case
  { bssid: "50:c7:bf:aa:bb:99", ssid: "Home", channel: 11, base: -60, from: 40 },
];

const bssidBytes = (b: string) => b;
const CLASS_BASE = { fastPair: 4, ibeacon: 3, proximity: 8, generic: 150 } as const;
const SPIKE_EVERY = 60;
const SPIKE_COUNT = 90;

export interface WorldStep {
  telemetry: Telemetry[];
  /** The channel the radio ends on (it hops while scanning unless pinned). */
  channel: number;
}

/**
 * A simulated radio environment: access points with drifting signal (reported the way the device reports
 * them: only when new or moved by 6 dB), deauth bursts, BLE advertisement counts and anomalies. No timers
 * and no DOM: callers decide when to `step`, which keeps it testable.
 */
export class MockWorld {
  config: MonitorConfig = { ...DEFAULT_CONFIG };
  private rand: () => number;
  private reported = new Map<string, number>();
  /** The owner's scan settings, applied the way the device applies them. */
  private policy = { dedupDb: 6, ssidFilter: "", channelMask: 0 };
  private rssi = new Map<string, number>();
  private scanTicks = 0;
  private bleTicks = 0;
  private handshakeEnabled = false;
  private handshakeTicks = 0;

  constructor(seed = 7) {
    this.rand = seeded(seed);
  }

  /** Adopt the scan settings (dedup threshold and filters) like the device does for following beacons. */
  applyScanSettings(s: Pick<ScanSettings, "dedupRssiDb" | "ssidFilter" | "channelFilterMask">): void {
    this.policy = { dedupDb: s.dedupRssiDb, ssidFilter: s.ssidFilter.toLowerCase(), channelMask: s.channelFilterMask };
  }

  /** `setMonitor` with the device's validation; returns the reply. */
  setMonitor(c: MonitorConfig): Telemetry {
    if (c.bleSpikeFactor < 2 || c.bleSpikeFactor > 20) return { type: "event", code: 5, message: "monitor config out of range" };
    if (c.bleEnabled && !this.config.bleEnabled) this.bleTicks = 0;
    this.config = { ...c };
    return this.report();
  }

  report(): Telemetry {
    return { type: "monitorConfig", ...this.config };
  }

  setHandshakeCapture(enabled: boolean): Telemetry {
    this.handshakeEnabled = enabled;
    this.handshakeTicks = 0;
    return { type: "handshakeCaptureState", enabled };
  }

  handshakeReport(): Telemetry {
    return { type: "handshakeCaptureState", enabled: this.handshakeEnabled };
  }

  /** One simulated second. */
  step(scanning: boolean, channel: number, pinned: number): WorldStep {
    const out: Telemetry[] = [];
    let ch = channel;
    if (scanning) {
      this.scanTicks++;
      if (!pinned) ch = (channel % 13) + 1;
      this.scanBeacons(ch, out);
      this.anomalies(out);
      this.deauth(out);
      this.handshake(out);
      this.probes(ch, out);
      this.associations(ch, out);
    }
    this.ble(out);
    return { telemetry: out, channel: ch };
  }

  private scanBeacons(ch: number, out: Telemetry[]): void {
    for (const ap of APS) {
      if (ap.channel !== ch || this.scanTicks < ap.from) continue;
      const { dedupDb, ssidFilter, channelMask } = this.policy;
      if (channelMask !== 0 && (channelMask & (1 << (ap.channel - 1))) === 0) continue;
      if (ssidFilter && !ap.ssid.toLowerCase().includes(ssidFilter)) continue;
      const prev = this.rssi.get(ap.bssid) ?? ap.base;
      // a slow drift, with the odd big swing
      const swing = this.rand() < 0.1 ? (this.rand() < 0.5 ? -1 : 1) * (8 + Math.floor(this.rand() * 12)) : Math.floor(this.rand() * 5) - 2;
      const now = Math.max(-95, Math.min(-30, prev + swing));
      this.rssi.set(ap.bssid, now);
      const last = this.reported.get(ap.bssid);
      // the device's de-duplication rule: report when new or moved by 6 dB or more
      if (last === undefined || Math.abs(now - last) >= dedupDb) {
        this.reported.set(ap.bssid, now);
        out.push({ type: "scan", bssid: bssidBytes(ap.bssid), channel: ap.channel, rssi: now, ssid: ap.ssid });
      }
    }
  }

  private anomalies(out: Telemetry[]): void {
    const t = this.scanTicks % 100;
    const home2 = APS[8].bssid;
    if (t === 41) out.push({ type: "anomaly", kind: "newBssidForSsid", bssid: home2, detail: 0 });
    if (t === 55) out.push({ type: "anomaly", kind: "twoChannels", bssid: APS[3].bssid, detail: 0 });
    if (t === 70) out.push({ type: "anomaly", kind: "rssiOutlier", bssid: APS[4].bssid, detail: 27 });
    if (t === 85) out.push({ type: "anomaly", kind: "returned", bssid: APS[6].bssid, detail: 8 });
  }

  private deauth(out: Telemetry[]): void {
    if (this.scanTicks % 15 !== 0) return;
    const count = 8 + Math.floor(this.rand() * 70);
    // the device only alerts at or above the threshold (0 = off)
    if (this.config.deauthThreshold > 0 && count >= this.config.deauthThreshold) {
      out.push({ type: "deauth", bssid: APS[3].bssid, count });
    }
  }

  /** Simulates a client re-authenticating on its own every so often: messages 1 then 2, a short while apart. */
  private handshake(out: Telemetry[]): void {
    if (!this.handshakeEnabled) return;
    this.handshakeTicks++;
    const t = this.handshakeTicks % 20;
    const bssid = APS[3].bssid; // "Home"
    const client = "6c:5a:72:00:01:02";
    const eapol = (fill: number) => Uint8Array.from({ length: 99 }, (_, i) => (i + fill) & 0xff);
    if (t === 1) {
      out.push({ type: "handshakeFrame", bssid, client, message: 1, replayCounter: this.handshakeTicks, truncated: false, eapol: eapol(1) });
      out.push({ type: "pmkidFound", bssid, client, pmkid: "00112233445566778899aabbccddeeff" });
    } else if (t === 3) {
      out.push({ type: "handshakeFrame", bssid, client, message: 2, replayCounter: this.handshakeTicks, truncated: false, eapol: eapol(2) });
      out.push({ type: "handshakeComplete", bssid, client }); // message 1 (ANonce) + message 2 (SNonce+MIC) is enough
    }
  }

  private probes(ch: number, out: Telemetry[]): void {
    if (this.scanTicks % 8 === 0) {
      out.push({ type: "probeResult", client: "da:a1:19:22:33:44", channel: ch, rssi: -58, ssid: "HomeNetwork" });
    } else if (this.scanTicks % 15 === 0) {
      out.push({ type: "probeResult", client: "f0:18:98:55:66:77", channel: ch, rssi: -72, ssid: "CoffeeShop" });
    }
  }

  private associations(ch: number, out: Telemetry[]): void {
    if (this.scanTicks % 6 === 0) {
      out.push({ type: "assocResult", bssid: APS[3].bssid, client: "da:a1:19:22:33:44", channel: ch, rssi: -54 });
    }
  }

  private ble(out: Telemetry[]): void {
    if (!this.config.bleEnabled) return;
    this.bleTicks++;
    if (this.bleTicks % 10 !== 0) return;
    const jitter = (n: number) => Math.max(0, n + Math.floor(this.rand() * 5) - 2);
    const spiking = this.bleTicks % SPIKE_EVERY === 0;
    const counts = {
      fastPair: jitter(CLASS_BASE.fastPair),
      ibeacon: jitter(CLASS_BASE.ibeacon),
      proximity: spiking ? SPIKE_COUNT : jitter(CLASS_BASE.proximity),
      generic: jitter(CLASS_BASE.generic),
    };
    out.push({ type: "bleStats", ...counts });
    // the device's rule: above max(baseline x factor, baseline + 20)
    const base = CLASS_BASE.proximity;
    if (spiking && counts.proximity > Math.max(base * this.config.bleSpikeFactor, base + 20)) {
      out.push({ type: "bleSpike", class: "proximity", count: counts.proximity, baseline: base });
    }
  }
}
