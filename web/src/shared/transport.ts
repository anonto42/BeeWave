import { type Command, type Telemetry, decodeTelemetry, encodeCommand } from "./protocol";

export const SERVICE_UUID = "7a1c0001-9d3e-4b8a-a1f0-5e5b1c0de001";
export const COMMAND_CHAR_UUID = "7a1c0002-9d3e-4b8a-a1f0-5e5b1c0de001";
export const TELEMETRY_CHAR_UUID = "7a1c0003-9d3e-4b8a-a1f0-5e5b1c0de001";

export type TransportState = "disconnected" | "connecting" | "connected";

export interface Transport {
  connect(): Promise<void>;
  disconnect(): void;
  send(cmd: Command): Promise<void>;
  onTelemetry(cb: (t: Telemetry) => void): void;
  onState(cb: (s: TransportState, info?: string) => void): void;
  /** Telemetry packets lost, detected from sequence gaps. */
  readonly lost: number;
}

/** Tracks sequence gaps so the UI can show real BLE loss. */
export class SeqTracker {
  lost = 0;
  private last = -1;
  push(seq: number) {
    if (this.last >= 0) this.lost += (seq - this.last - 1 + 256) & 0xff;
    this.last = seq;
  }
  reset() { this.last = -1; }
}

export class BleTransport implements Transport {
  private tele?: (t: Telemetry) => void;
  private state?: (s: TransportState, info?: string) => void;
  private dev?: BluetoothDevice;
  private cmdChar?: BluetoothRemoteGATTCharacteristic;
  private seq = 0;
  private tracker = new SeqTracker();
  get lost() { return this.tracker.lost; }

  onTelemetry(cb: (t: Telemetry) => void) { this.tele = cb; }
  onState(cb: (s: TransportState, info?: string) => void) { this.state = cb; }

  async connect() {
    if (!navigator.bluetooth) throw new Error("Web Bluetooth unavailable (use Chrome/Edge on Android or desktop; iOS needs Bluefy)");
    this.state?.("connecting");
    try {
      this.dev = await navigator.bluetooth.requestDevice({
        filters: [{ namePrefix: "WalPad" }],
        optionalServices: [SERVICE_UUID],
      });
      this.dev.addEventListener("gattserverdisconnected", () => this.state?.("disconnected"));
      const server = await this.dev.gatt!.connect();
      const svc = await server.getPrimaryService(SERVICE_UUID);
      this.cmdChar = await svc.getCharacteristic(COMMAND_CHAR_UUID);
      const tc = await svc.getCharacteristic(TELEMETRY_CHAR_UUID);
      this.tracker.reset();
      tc.addEventListener("characteristicvaluechanged", (e) => {
        const dv = (e.target as BluetoothRemoteGATTCharacteristic).value!;
        try {
          const { seq, msg } = decodeTelemetry(new Uint8Array(dv.buffer, dv.byteOffset, dv.byteLength));
          this.tracker.push(seq);
          this.tele?.(msg);
        } catch (err) { console.warn("bad telemetry packet", err); }
      });
      await tc.startNotifications();
      this.state?.("connected");
    } catch (e) {
      this.state?.("disconnected", String(e));
      throw e;
    }
  }

  disconnect() { this.dev?.gatt?.disconnect(); }

  async send(cmd: Command) {
    if (!this.cmdChar) throw new Error("not connected");
    await this.cmdChar.writeValueWithoutResponse(encodeCommand(this.seq++ & 0xff, cmd));
  }
}

/** Fake device so the UI can be developed without hardware. */
export class MockTransport implements Transport {
  private tele?: (t: Telemetry) => void;
  private state?: (s: TransportState, info?: string) => void;
  private timer?: number;
  private scanning = false;
  private channel = 1;
  lost = 0;
  onTelemetry(cb: (t: Telemetry) => void) { this.tele = cb; }
  onState(cb: (s: TransportState, info?: string) => void) { this.state = cb; }
  async connect() {
    this.state?.("connected");
    this.timer = window.setInterval(() => {
      this.tele?.({ type: "status", activeChannel: this.channel, freeHeap: 90_000 + Math.floor(Math.random() * 2000), cpuLoad: 20 + Math.floor(Math.random() * 30), dropped: 0 });
      if (this.scanning) {
        this.channel = (this.channel % 13) + 1;
        const b = Array.from({ length: 6 }, () => Math.floor(Math.random() * 256).toString(16).padStart(2, "0")).join(":");
        this.tele?.({ type: "scan", bssid: b, channel: this.channel, rssi: -40 - Math.floor(Math.random() * 50), ssid: ["Home", "Office", "Cafe", "", "Guest"][Math.floor(Math.random() * 5)] });
      }
    }, 700);
  }
  disconnect() { clearInterval(this.timer); this.state?.("disconnected"); }
  async send(cmd: Command) {
    if (cmd.type === "startScan") this.scanning = true;
    if (cmd.type === "stop") this.scanning = false;
    if (cmd.type === "configureRadio") this.channel = cmd.channel;
    this.tele?.({ type: "event", code: 1, message: `ack ${cmd.type}` });
  }
}
