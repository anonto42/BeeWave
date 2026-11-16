import { describe, expect, it } from "vitest";
import { SeqTracker } from "../../src/shared/transport";

describe("SeqTracker", () => {
  it("counts gaps and wraps at 255", () => {
    const t = new SeqTracker();
    [0, 1, 4, 255, 2].forEach((s) => t.push(s));
    expect(t.lost).toBe(2 /* 2,3 */ + 250 /* 5..254 */ + 2 /* 0,1 */);
  });
});
