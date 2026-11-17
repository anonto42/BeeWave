import { describe, expect, it } from "vitest";
import { tabFromHash } from "../../src/shared/shell";

const ids = ["device", "scan", "settings", "log"];

describe("tabFromHash", () => {
  it("selects the tab named by the hash", () => {
    expect(tabFromHash("#scan", ids, "device")).toBe("scan");
  });
  it("falls back for empty or unknown hashes", () => {
    expect(tabFromHash("", ids, "device")).toBe("device");
    expect(tabFromHash("#nope", ids, "device")).toBe("device");
  });
});
