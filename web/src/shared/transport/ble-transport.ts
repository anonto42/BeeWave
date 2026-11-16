import { decodeTelemetry, encodeCommand, type Command, type Telemetry } from "../protocol";
import { COMMAND_CHAR_UUID, DEVICE_NAME_PREFIX, SERVICE_UUID, TELEMETRY_CHAR_UUID } from "./constants";
import { SeqTracker } from "./seq-tracker";
import type { Transport, TransportState } from "./types";

/** Web Bluetooth implementation of Transport. */
export class BleTransport implements Transport {
  private onTele?: (t: Telemetry) => void;
  private onStateCb?: (s: TransportState, info?: string) => void;
  private device?: BluetoothDevice;
  private cmdChar?: BluetoothRemoteGATTCharacteristic;
  private seq = 0;
  private readonly tracker = new SeqTracker();

  get lost(): number {
    return this.tracker.lost;
  }

  onTelemetry(cb: (t: Telemetry) => void): void { this.onTele = cb; }
  onState(cb: (s: TransportState, info?: string) => void): void { this.onStateCb = cb; }

  async connect(): Promise<void> {
    if (!navigator.bluetooth) {
      throw new Error("Web Bluetooth unavailable (use Chrome/Edge on Android or desktop; iOS needs Bluefy)");
    }
    this.onStateCb?.("connecting");
    try {
      this.device = await navigator.bluetooth.requestDevice({
        filters: [{ namePrefix: DEVICE_NAME_PREFIX }],
        optionalServices: [SERVICE_UUID],
      });
      this.device.addEventListener("gattserverdisconnected", () => this.onStateCb?.("disconnected"));

      const server = await this.device.gatt!.connect();
      const service = await server.getPrimaryService(SERVICE_UUID);
      this.cmdChar = await service.getCharacteristic(COMMAND_CHAR_UUID);
      const telemetry = await service.getCharacteristic(TELEMETRY_CHAR_UUID);

      this.tracker.reset();
      telemetry.addEventListener("characteristicvaluechanged", (e) => this.handlePacket(e));
      await telemetry.startNotifications();
      this.onStateCb?.("connected");
    } catch (e) {
      this.onStateCb?.("disconnected", String(e));
      throw e;
    }
  }

  disconnect(): void {
    this.device?.gatt?.disconnect();
  }

  async send(cmd: Command): Promise<void> {
    if (!this.cmdChar) throw new Error("not connected");
    await this.cmdChar.writeValueWithoutResponse(encodeCommand(this.seq++ & 0xff, cmd));
  }

  private handlePacket(e: Event): void {
    const dv = (e.target as BluetoothRemoteGATTCharacteristic).value!;
    try {
      const { seq, msg } = decodeTelemetry(new Uint8Array(dv.buffer, dv.byteOffset, dv.byteLength));
      this.tracker.push(seq);
      this.onTele?.(msg);
    } catch (err) {
      console.warn("bad telemetry packet", err);
    }
  }
}
