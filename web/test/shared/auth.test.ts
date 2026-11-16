import { describe, expect, it } from "vitest";
import { Authenticator, KeyStore, computeProof, parseKey } from "../../src/shared/auth";
import type { Command, Telemetry } from "../../src/shared/protocol";

const hex = (b: Uint8Array) => [...b].map((x) => x.toString(16).padStart(2, "0")).join("");
const KEY_HEX = Array.from({ length: 32 }, (_, i) => i.toString(16).padStart(2, "0")).join("");
const NONCE = Uint8Array.from({ length: 16 }, (_, i) => 16 + i);

function memoryStorage(initial?: string) {
  const data = new Map<string, string>(initial ? [["beewave.key", initial]] : []);
  return {
    getItem: (k: string) => data.get(k) ?? null,
    setItem: (k: string, v: string) => void data.set(k, v),
    removeItem: (k: string) => void data.delete(k),
  };
}

describe("device key", () => {
  it("parses exactly 64 hex characters", () => {
    expect(parseKey(KEY_HEX)).toHaveLength(32);
    expect(parseKey(` ${KEY_HEX.toUpperCase()}\n`)).toHaveLength(32);
    expect(parseKey(KEY_HEX.slice(2))).toBeNull();
    expect(parseKey("zz".repeat(32))).toBeNull();
    expect(parseKey("")).toBeNull();
  });

  it("is saved, reloaded and cleared", () => {
    const storage = memoryStorage();
    const store = new KeyStore(storage);
    expect(store.get()).toBeNull();
    expect(store.set("nope")).toBe(false);
    expect(store.set(KEY_HEX)).toBe(true);
    expect(hex(new KeyStore(storage).get()!)).toBe(KEY_HEX);
    store.clear();
    expect(new KeyStore(storage).get()).toBeNull();
  });

  it("works in memory when storage throws", () => {
    const broken = {
      getItem: () => { throw new Error("blocked"); },
      setItem: () => { throw new Error("blocked"); },
      removeItem: () => { throw new Error("blocked"); },
    };
    const store = new KeyStore(broken);
    expect(store.set(KEY_HEX)).toBe(true);
    expect(store.get()).not.toBeNull();
  });
});

describe("proof", () => {
  it("matches the Rust known answer", async () => {
    const proof = await computeProof(parseKey(KEY_HEX)!, NONCE);
    expect(hex(proof)).toBe("7395f8286d558150f51cdba47200dcd4020f5a5abf2748d8170071d24158ca02");
  });
});

describe("Authenticator", () => {
  /** A fake device that checks the proof the way the firmware does. */
  function setup(deviceKeyHex: string, storedKeyHex?: string) {
    const keys = new KeyStore(memoryStorage(storedKeyHex));
    const sent: Command[] = [];
    const auth: Authenticator = new Authenticator(async (c) => {
      sent.push(c);
      if (c.type === "authBegin") await auth.handle({ type: "authChallenge", nonce: NONCE });
      if (c.type === "authProve") {
        const expected = await computeProof(parseKey(deviceKeyHex)!, NONCE);
        await auth.handle({ type: "authResult", ok: hex(expected) === hex(c.mac) });
      }
    }, keys);
    return { auth, sent };
  }

  it("authenticates with the right key", async () => {
    const { auth, sent } = setup(KEY_HEX, KEY_HEX);
    await auth.start();
    expect(auth.state).toBe("authenticated");
    expect(sent.map((c) => c.type)).toEqual(["authBegin", "authProve"]);
  });

  it("reports `authenticated` only after the device accepts the proof", async () => {
    const { auth } = setup(KEY_HEX, KEY_HEX);
    const seen: string[] = [];
    auth.onChange((s) => seen.push(s));
    await auth.start();
    expect(seen).toEqual(["authenticating", "authenticated"]);

    const wrong = setup("ff".repeat(32), KEY_HEX);
    const wrongSeen: string[] = [];
    wrong.auth.onChange((s) => wrongSeen.push(s));
    await wrong.auth.start();
    expect(wrongSeen).not.toContain("authenticated");
  });

  it("reports failure with the wrong key", async () => {
    const { auth } = setup("ff".repeat(32), KEY_HEX);
    await auth.start();
    expect(auth.state).toBe("failed");
  });

  it("does nothing without a key", async () => {
    const { auth, sent } = setup(KEY_HEX);
    await auth.start();
    expect(auth.state).toBe("no-key");
    expect(sent).toEqual([]);
  });

  it("ignores unrelated telemetry and forgets the session on reset", async () => {
    const { auth } = setup(KEY_HEX, KEY_HEX);
    await auth.start();
    await auth.handle({ type: "event", code: 1, message: "x" } satisfies Telemetry);
    expect(auth.state).toBe("authenticated");
    auth.reset();
    expect(auth.state).toBe("idle");
  });

  it("fails when the send is rejected", async () => {
    const auth = new Authenticator(async () => { throw new Error("not connected"); }, new KeyStore(memoryStorage(KEY_HEX)));
    await auth.start();
    expect(auth.state).toBe("failed");
  });
});
