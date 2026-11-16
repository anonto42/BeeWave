import type { Command, Telemetry } from "../protocol";
import type { KeyStore } from "./key-store";
import { computeProof } from "./proof";
import type { AuthState } from "./types";

/** Runs the challenge-response handshake. Fed device telemetry by the app context. */
export class Authenticator {
  private current: AuthState;
  private subs: Array<(s: AuthState) => void> = [];

  constructor(
    private readonly send: (cmd: Command) => Promise<void>,
    private readonly keys: KeyStore,
  ) {
    this.current = keys.get() ? "idle" : "no-key";
  }

  get state(): AuthState {
    return this.current;
  }

  onChange(cb: (s: AuthState) => void): void {
    this.subs.push(cb);
  }

  /** Call when the link comes up (or the key changes while connected). */
  async start(): Promise<void> {
    if (!this.keys.get()) return this.set("no-key");
    this.set("authenticating");
    try {
      await this.send({ type: "authBegin" });
    } catch {
      this.set("failed");
    }
  }

  /** Call when the link goes down: the device forgets the session, so do we. */
  reset(): void {
    this.set(this.keys.get() ? "idle" : "no-key");
  }

  /** Feed every telemetry message here; only auth messages are acted on. */
  async handle(msg: Telemetry): Promise<void> {
    const key = this.keys.get();
    if (msg.type === "authChallenge" && key) {
      try {
        await this.send({ type: "authProve", mac: await computeProof(key, msg.nonce) });
      } catch {
        this.set("failed");
      }
    } else if (msg.type === "authResult") {
      this.set(msg.ok ? "authenticated" : "failed");
    }
  }

  private set(s: AuthState): void {
    if (s === this.current) return;
    this.current = s;
    this.subs.forEach((f) => f(s));
  }
}
