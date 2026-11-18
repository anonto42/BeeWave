import type { Command, Telemetry } from "../protocol";
import { MockSettings } from "./mock-settings";
import { MockWorld } from "./mock-world";
import type { Transport, TransportState } from "./types";

const rand = (n: number) => Math.floor(Math.random() * n);

/** Simulated device so the UI can be developed without hardware (`?mock`). */
export class MockTransport implements Transport {
  readonly lost = 0;
  readonly deviceName = "BeeWave-mock";
  private onTele?: (t: Telemetry) => void;
  private onStateCb?: (s: TransportState, info?: string) => void;
  private timer?: number;
  private scanning = false;
  private channel = 1;
  private pinned = 0; // 0 = auto-hop
  /** True once a client has provisioned a key (the mock starts unclaimed, like fresh firmware). */
  private claimed = false;
  /** The simulated radio environment (access points, deauth, BLE, anomalies). */
  private readonly world = new MockWorld();
  /** The simulated device's persisted settings. */
  private readonly stored = new MockSettings();

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
    const handled = this.stored.handle(cmd, { scanning: this.scanning });
    if (handled) {
      this.world.applyScanSettings(this.stored.settings.scan);
      handled.replies.forEach((t) => this.onTele?.(t));
      if (handled.factoryReset) {
        this.claimed = false; // the key goes with the factory reset, like on the real device
        setTimeout(() => this.disconnect(), 200);
      }
      return;
    }
    if (cmd.type === "getMonitor") {
      this.onTele?.(this.world.report());
      return;
    }
    if (cmd.type === "setMonitor") {
      this.onTele?.(this.world.setMonitor({ deauthThreshold: cmd.deauthThreshold, bleEnabled: cmd.bleEnabled, bleSpikeFactor: cmd.bleSpikeFactor }));
      return;
    }
    if (cmd.type === "getHandshakeCapture") {
      this.onTele?.(this.world.handshakeReport());
      return;
    }
    if (cmd.type === "setHandshakeCapture") {
      this.onTele?.(this.world.setHandshakeCapture(cmd.enabled));
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
    const { telemetry, channel } = this.world.step(this.scanning, this.channel, this.pinned);
    this.channel = channel;
    telemetry.forEach((t) => this.onTele?.(t));
  }
}
