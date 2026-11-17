import type { Command, Telemetry } from "../protocol";
import type { Transport, TransportState } from "./types";

const SSIDS = ["Home", "Office", "Cafe", "", "Guest"];
const rand = (n: number) => Math.floor(Math.random() * n);
const randomBssid = () => Array.from({ length: 6 }, () => rand(256).toString(16).padStart(2, "0")).join(":");

/** Simulated device so the UI can be developed without hardware (`?mock`). */
export class MockTransport implements Transport {
  readonly lost = 0;
  private onTele?: (t: Telemetry) => void;
  private onStateCb?: (s: TransportState, info?: string) => void;
  private timer?: number;
  private scanning = false;
  private channel = 1;
  private pinned = 0; // 0 = auto-hop
  /** True once a client has provisioned a key (the mock starts unclaimed, like fresh firmware). */
  private claimed = false;

  onTelemetry(cb: (t: Telemetry) => void): void { this.onTele = cb; }
  onState(cb: (s: TransportState, info?: string) => void): void { this.onStateCb = cb; }

  async connect(): Promise<void> {
    this.onStateCb?.("connected");
    this.timer = window.setInterval(() => this.tick(), 700);
  }

  disconnect(): void {
    clearInterval(this.timer);
    this.onStateCb?.("disconnected");
  }

  async send(cmd: Command): Promise<void> {
    if (cmd.type === "authBegin" && !this.claimed) {
      this.onTele?.({ type: "unclaimed" });
      return;
    }
    if (cmd.type === "provision") {
      const ok = !this.claimed;
      this.claimed = true;
      this.onTele?.({ type: "authResult", ok });
      return;
    }
    if (cmd.type === "authBegin") {
      const nonce = Uint8Array.from({ length: 16 }, () => rand(256));
      this.onTele?.({ type: "authChallenge", nonce });
      return;
    }
    if (cmd.type === "authProve") {
      this.onTele?.({ type: "authResult", ok: true }); // the mock accepts any key
      return;
    }
    if (cmd.type === "startScan") this.scanning = true;
    if (cmd.type === "stop") this.scanning = false;
    if (cmd.type === "configureRadio") {
      this.pinned = cmd.channel;
      if (cmd.channel) this.channel = cmd.channel;
    }
    this.onTele?.({ type: "event", code: 100, message: `ack ${cmd.type}` });
  }

  private tick(): void {
    this.onTele?.({ type: "status", activeChannel: this.channel, freeHeap: 90_000 + rand(2000), cpuLoad: 20 + rand(30), dropped: 0, scanning: this.scanning });
    if (!this.scanning) return;
    if (!this.pinned) this.channel = (this.channel % 13) + 1;
    this.onTele?.({ type: "scan", bssid: randomBssid(), channel: this.channel, rssi: -40 - rand(50), ssid: SSIDS[rand(SSIDS.length)] });
  }
}
