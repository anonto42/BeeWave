import { describe, expect, it } from "vitest";
import { TargetStore } from "../../src/shared/targets";

const memory = () => {
  const m = new Map<string, string>();
  return { getItem: (k: string) => m.get(k) ?? null, setItem: (k: string, v: string) => void m.set(k, v) };
};

describe("TargetStore", () => {
  it("toggles, notifies and persists", () => {
    const storage = memory();
    const a = new TargetStore(storage);
    let changes = 0;
    a.onChange(() => changes++);
    a.toggle({ bssid: "aa", ssid: "Home" });
    expect(a.has("aa")).toBe(true);
    expect(new TargetStore(storage).list()).toEqual([{ bssid: "aa", ssid: "Home" }]);
    a.toggle({ bssid: "aa", ssid: "Home" });
    expect(a.list()).toEqual([]);
    expect(changes).toBe(2);
  });

  it("survives corrupted or missing storage", () => {
    const bad = { getItem: () => "{not json", setItem: () => { throw new Error("quota"); } };
    const s = new TargetStore(bad);
    expect(s.list()).toEqual([]);
    expect(() => s.toggle({ bssid: "b", ssid: "" })).not.toThrow();
    expect(new TargetStore(null).list()).toEqual([]);
  });
});
