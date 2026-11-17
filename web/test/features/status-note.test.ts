import { describe, expect, it } from "vitest";
import { statusNote } from "../../src/features/status/note";
import { linkLabel } from "../../src/features/connection/link-label";

describe("statusNote", () => {
  it("explains why status is missing", () => {
    expect(statusNote("disconnected", "idle", false)).toMatch(/Not connected/);
    expect(statusNote("connected", "no-key", false)).toMatch(/already claimed/);
    expect(statusNote("connected", "failed", false)).toMatch(/rejected/);
    expect(statusNote("connected", "authenticated", false)).toMatch(/Waiting/);
  });
  it("is empty once status flows", () => {
    expect(statusNote("connected", "authenticated", true)).toBe("");
  });
});

describe("linkLabel", () => {
  it("combines link and key state", () => {
    expect(linkLabel("disconnected", "idle")).toEqual({ text: "disconnected", tone: "off" });
    expect(linkLabel("connected", "authenticated")).toEqual({ text: "connected", tone: "ok" });
    expect(linkLabel("connected", "no-key")).toEqual({ text: "needs key", tone: "warn" });
    expect(linkLabel("connected", "failed").tone).toBe("bad");
  });
});
