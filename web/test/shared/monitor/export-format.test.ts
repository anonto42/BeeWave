import { describe, expect, it } from "vitest";
import { createMonitorKernel, csvText, sessionToCsv, sessionToJson } from "../../../src/shared/monitor";

function session() {
  let t = Date.UTC(2026, 10, 17, 12, 0, 0);
  const k = createMonitorKernel(() => (t += 1000), { getItem: () => null, setItem: () => {} });
  k.session.addAuth("authenticating");
  k.session.addAuth("authenticated");
  k.session.addLog("link connected");
  k.ingest({ type: "scan", bssid: "aa:bb:cc:dd:ee:01", channel: 6, rssi: -55, ssid: "Home" });
  k.ingest({ type: "scan", bssid: "aa:bb:cc:dd:ee:02", channel: 11, rssi: -70, ssid: 'Cafe, "Main"' });
  k.ingest({ type: "deauth", bssid: "aa:bb:cc:dd:ee:01", count: 33 });
  k.scans.snapshot("first");
  return k.session.build(k.scans.events(), k.alerts.list(), k.scans.snapshots());
}

describe("session export", () => {
  it("JSON holds scans, alerts, log, auth events and snapshots, oldest first", () => {
    const d = JSON.parse(sessionToJson(session()));
    expect(Object.keys(d).sort()).toEqual(["alerts", "auth", "exportedAt", "log", "scans", "snapshots", "startedAt"]);
    expect(d.scans.map((s: { bssid: string }) => s.bssid)).toEqual(["aa:bb:cc:dd:ee:01", "aa:bb:cc:dd:ee:02"]);
    expect(d.auth.map((a: { state: string }) => a.state)).toEqual(["authenticating", "authenticated"]);
    expect(d.alerts[0]).toMatchObject({ source: "deauth", severity: "warn" });
    expect(d.log[0].text).toBe("link connected");
    expect(d.snapshots[0].label).toBe("first");
    expect(new Date(d.exportedAt).toString()).not.toBe("Invalid Date");
  });

  it("CSV has one header and one flat row per record, with kind and ISO time", () => {
    const lines = sessionToCsv(session()).trimEnd().split("\r\n");
    expect(lines[0]).toBe("kind,time,bssid,ssid,channel,rssi,severity,source,message");
    const kinds = lines.slice(1).map((l) => l.split(",")[0]);
    expect(kinds).toEqual(["scan", "scan", "alert", "log", "auth", "auth"]);
    expect(lines[1]).toMatch(/^scan,2026-11-17T12:00:\d\d\.\d{3}Z,aa:bb:cc:dd:ee:01,Home,6,-55,,,$/);
  });

  it("quotes commas and quotes in text, and keeps negative numbers as numbers", () => {
    const row = sessionToCsv(session()).split("\r\n")[2];
    expect(row).toContain('"Cafe, ""Main"""');
    expect(row).toContain(",11,-70,");
  });

  it("neutralises spreadsheet formulas in SSIDs and messages", () => {
    expect(csvText("=HYPERLINK(\"http://x\")")).toBe("\"'=HYPERLINK(\"\"http://x\"\")\"");
    expect(csvText("+1")).toBe("'+1");
    expect(csvText("-cmd")).toBe("'-cmd");
    expect(csvText("@SUM(1)")).toBe("'@SUM(1)");
    expect(csvText("plain")).toBe("plain");
  });
});
