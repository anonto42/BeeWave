import type { Command, Telemetry } from "../protocol";
import type { KeyStore } from "./key-store";
import { formatKey, randomKey } from "./key";
import { computeProof } from "./proof";
import type { AuthState } from "./types";

/** Runs the challenge-response handshake. Fed device telemetry by the app context. */
export class Authenticator {
  private current: AuthState;
  private subs: Array<(s: AuthState) => void> = [];
  /** The key we offered an unclaimed device; saved only once the device accepts it. */
  private claiming: Uint8Array | null = null;

  constructor(
    private readonly send: (cmd: Command) => Promise<void>,
    private readonly keys: KeyStore,
    private readonly newKey: () => Uint8Array = randomKey,
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
    // Always ask: a device without a key answers `unclaimed` and we claim it; otherwise it sends a
    // challenge, which needs a saved key.
    this.claiming = null;
    this.set("authenticating");
    try {
      await this.send({ type: "authBegin" });
    } catch {
      this.set("failed");
    }
  }

  /** Call when the link goes down: the device forgets the session, so do we. */
  reset(): void {
    this.claiming = null;
    this.set(this.keys.get() ? "idle" : "no-key");
  }

  /** Feed every telemetry message here; only auth messages are acted on. */
  async handle(msg: Telemetry): Promise<void> {
    const key = this.keys.get();
    if (msg.type === "unclaimed") {
      await this.claim();
    } else if (msg.type === "authChallenge") {
      if (!key) return this.set("no-key"); // claimed by another browser: the user must supply its key
      try {
        await this.send({ type: "authProve", mac: await computeProof(key, msg.nonce) });
      } catch {
        this.set("failed");
      }
    } else if (msg.type === "authResult") {
      if (this.claiming && msg.ok) this.keys.set(formatKey(this.claiming));
      this.claiming = null;
      this.set(msg.ok ? "authenticated" : "failed");
    }
  }

  /** First connect to an unclaimed device: generate a key and offer it. */
  private async claim(): Promise<void> {
    this.claiming = this.newKey();
    try {
      await this.send({ type: "provision", key: this.claiming });
    } catch {
      this.claiming = null;
      this.set("failed");
    }
  }

  private set(s: AuthState): void {
    if (s === this.current) return;
    this.current = s;
    this.subs.forEach((f) => f(s));
  }
}
