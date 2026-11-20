import { describe, expect, it } from "vitest";
import { describeEvent } from "../../src/features/alerts/describe";
import { controlState } from "../../src/features/control/enabled";
import { rssiHtml, rssiLevel } from "../../src/shared/monitor";
import { filterNetworks } from "../../src/features/scanner/filter";
import { sortNetworks } from "../../src/features/scanner/sort";
import { rowHtml } from "../../src/features/scanner/template";
import type { ScanResult } from "../../src/features/scanner/types";

const net = (ssid: string, channel: number, rssi: number): ScanResult => ({ type: "scan", bssid: `00:00:00:00:00:${channel}${rssi}`, ssid, channel, rssi });
const nets = [net("b", 6, -70), net("", 1, -40), net("a", 11, -55)];

describe("describeEvent", () => {
  it("titles the device's error codes", () => {
    expect(describeEvent(4, "not authenticated")).toBe("Locked: not authenticated");
    expect(describeEvent(1, "")).toBe("Radio error");
  });
  it("ignores informational events", () => {
    expect(describeEvent(100, "ack startScan")).toBeNull();
  });
});

describe("controlState", () => {
  it("flips the one scan button between start and stop once authenticated", () => {
    expect(controlState("connected", "authenticated", false)).toEqual({ mode: "IDLE", run: { enabled: true, label: "START SCAN" }, channelEnabled: true });
    expect(controlState("connected", "authenticated", true)).toMatchObject({ mode: "SCANNING", run: { label: "STOP SCAN", enabled: true } });
  });
  it("keeps the controls off until the key handshake succeeded, and says why", () => {
    expect(controlState("connected", "authenticating", false)).toMatchObject({ mode: "AUTHENTICATING", channelEnabled: false, run: { enabled: false } });
    expect(controlState("connected", "no-key", false).mode).toBe("LOCKED");
    expect(controlState("connected", "failed", false).mode).toBe("LOCKED");
  });
  it("is off while offline", () => {
    expect(controlState("disconnected", "idle", false)).toMatchObject({ mode: "OFFLINE", channelEnabled: false, run: { enabled: false } });
    expect(controlState("connecting", "idle", false).mode).toBe("OFFLINE");
  });
});

describe("RSSI bands", () => {
  it("maps dBm to 0-5 bars at -30/-67/-70/-80/-90", () => {
    expect([-20, -30, -31, -67, -68, -70, -71, -80, -81, -90, -91].map(rssiLevel)).toEqual([5, 5, 4, 4, 3, 3, 2, 2, 1, 1, 0]);
  });
  it("shows the number or the bars, with the number as the bars' tooltip", () => {
    expect(rssiHtml(-55, "dbm")).toBe('<span class="dbm">-55</span>');
    const bars = rssiHtml(-55, "bars");
    expect(bars).toContain('data-bars="4"');
    expect(bars).toContain('title="-55 dBm"');
    expect(bars.match(/<i><\/i>/g)).toHaveLength(5);
  });
});

describe("sortNetworks", () => {
  it("sorts by signal, channel and name without mutating the input", () => {
    expect(sortNetworks(nets, "rssi").map((n) => n.rssi)).toEqual([-40, -55, -70]);
    expect(sortNetworks(nets, "channel").map((n) => n.channel)).toEqual([1, 6, 11]);
    expect(sortNetworks(nets, "ssid").map((n) => n.ssid)).toEqual(["a", "b", ""]);
    expect(nets[0].ssid).toBe("b");
  });
});

describe("rowHtml with WPS", () => {
  it("renders WPS badges and model information when present", () => {
    const r = rowHtml(net("TestNet", 6, -50), false, false, {
      version: 0x20,
      state: 2,
      locked: true,
      selectedRegistrar: false,
      deviceName: "Archer AX50",
      manufacturer: "TP-Link",
      modelName: "AX3000",
    });
    expect(r).toContain("WPS 2.0 · LOCKED");
    expect(r).toContain("Archer AX50");
  });
});

describe("rowHtml with Security", () => {
  it("renders security and PMF badges when present", () => {
    const r = rowHtml(net("SecureNet", 6, -50), false, false, undefined, {
      protocol: "wpa3",
      pmf: "required",
    });
    expect(r).toContain("WPA3 · PMF");
  });

  it("renders OPEN warning badge for open networks", () => {
    const r = rowHtml(net("OpenNet", 1, -60), false, false, undefined, {
      protocol: "open",
      pmf: "none",
    });
    expect(r).toContain("badge small warn");
    expect(r).toContain("OPEN");
  });
});

describe("rowHtml with Decloaked", () => {
  it("renders DECLOAKED badge when decloaked flag is true", () => {
    const r = rowHtml(net("UncloakedNet", 6, -55), false, false, undefined, undefined, true);
    expect(r).toContain("badge small warn");
    expect(r).toContain("DECLOAKED");
    expect(r).toContain("UncloakedNet");
  });

  it("does not render DECLOAKED badge when decloaked flag is false or undefined", () => {
    const r = rowHtml(net("NormalNet", 6, -55), false, false);
    expect(r).not.toContain("DECLOAKED");
  });
});

describe("rowHtml with PHY standard", () => {
  it("renders Wi-Fi 6 badge for ax standard", () => {
    const r = rowHtml(net("AxNet", 6, -50), false, false, undefined, undefined, false, "ax");
    expect(r).toContain("Wi-Fi 6");
  });

  it("renders Wi-Fi 4 badge for n standard", () => {
    const r = rowHtml(net("NNet", 6, -50), false, false, undefined, undefined, false, "n");
    expect(r).toContain("Wi-Fi 4");
  });

  it("renders 802.11b warn badge for legacy b standard", () => {
    const r = rowHtml(net("BNet", 1, -50), false, false, undefined, undefined, false, "b");
    expect(r).toContain("badge small warn");
    expect(r).toContain("802.11b");
  });

  it("does not render PHY badge when unknown or undefined", () => {
    const r = rowHtml(net("UnknownNet", 6, -50), false, false, undefined, undefined, false, "unknown");
    expect(r).not.toContain("Wi-Fi");
    expect(r).not.toContain("802.11");
  });
});

describe("filterNetworks", () => {
  const sample = [
    net("strong", 1, -45),
    net("medium", 6, -65),
    net("usable", 11, -75),
    net("weak", 6, -85),
  ];

  it("filters by RSSI thresholds", () => {
    expect(filterNetworks(sample, "all")).toHaveLength(4);
    expect(filterNetworks(sample, "-80").map((n) => n.ssid)).toEqual(["strong", "medium", "usable"]);
    expect(filterNetworks(sample, "-70").map((n) => n.ssid)).toEqual(["strong", "medium"]);
    expect(filterNetworks(sample, "-60").map((n) => n.ssid)).toEqual(["strong"]);
  });

  it("filters by presence of associated stations", () => {
    const counts = (bssid: string) => (bssid === sample[1].bssid ? 2 : 0);
    const withClients = filterNetworks(sample, "stations", counts);
    expect(withClients).toHaveLength(1);
    expect(withClients[0].ssid).toBe("medium");
  });
});

describe("rowHtml with stationCount", () => {
  it("renders 1 CLIENT badge with ok styling when stationCount is 1", () => {
    const r = rowHtml(net("TargetAP", 6, -50), false, false, undefined, undefined, false, undefined, 1);
    expect(r).toContain("1 CLIENT");
    expect(r).toContain("badge small ok");
    expect(r).toContain('title="1 connected client station"');
  });

  it("renders plural CLIENTS badge when stationCount is greater than 1", () => {
    const r = rowHtml(net("TargetAP", 6, -50), false, false, undefined, undefined, false, undefined, 3);
    expect(r).toContain("3 CLIENTS");
    expect(r).toContain('title="3 connected client stations"');
  });

  it("omits station badge when stationCount is 0 or undefined", () => {
    const rZero = rowHtml(net("TargetAP", 6, -50), false, false, undefined, undefined, false, undefined, 0);
    expect(rZero).not.toContain("CLIENT");
    const rUndef = rowHtml(net("TargetAP", 6, -50), false, false);
    expect(rUndef).not.toContain("CLIENT");
  });
});
