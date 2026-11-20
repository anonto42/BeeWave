// Composition root: choose transport, create context, build the tab shell, mount feature slices into tabs.
import "./style.css";
import { registerSW } from "virtual:pwa-register";
import { applyUiSettings, createContext } from "./shared/app";
import { createTabShell, ICONS, mountSlot } from "./shared/shell";
import { BleTransport, MockTransport } from "./shared/transport";
import * as connection from "./features/connection";
import * as status from "./features/status";
import * as control from "./features/control";
import * as scanner from "./features/scanner";
import * as log from "./features/log";
import * as ota from "./features/ota";
import * as settings from "./features/settings";
import * as targets from "./features/targets";
import * as compat from "./features/compat";
import * as alerts from "./features/alerts";
import * as results from "./features/results";
import * as clients from "./features/clients";
import * as channelChart from "./features/channel-chart";
import * as rssiHistory from "./features/rssi-history";
import * as compare from "./features/compare";
import * as anomalies from "./features/anomalies";
import * as deauth from "./features/deauth";
import * as bleMonitor from "./features/ble-monitor";
import * as handshake from "./features/handshake";

registerSW({ immediate: true });

const mock = new URLSearchParams(location.search).has("mock");
const ctx = createContext(mock ? new MockTransport() : new BleTransport());

const root = document.getElementById("app")!;
root.replaceChildren(); // drop the splash from index.html
const shell = createTabShell(root, "BEEWAVE", [
  { id: "device", label: "Device", icon: ICONS.device },
  { id: "scan", label: "Scan", icon: ICONS.scan },
  { id: "analyze", label: "Analyze", icon: ICONS.analyze },
  { id: "alerts", label: "Alerts", icon: ICONS.alerts },
  { id: "settings", label: "Settings", icon: ICONS.settings },
  { id: "log", label: "Log", icon: ICONS.log },
], mock ? "MOCK" : undefined);

// Slot names are the CSS layout hooks (`[data-tab="scan"] [data-slot="control"]` in style.css).
const layout = {
  device: { compat, status },
  scan: { control, scanner, targets, handshake },
  analyze: { results, clients, channelChart, rssiHistory, compare },
  alerts: { anomalies, deauth, bleMonitor },
  settings: { settings, ota },
  log: { log },
} as const;
for (const [tab, features] of Object.entries(layout))
  for (const [slot, f] of Object.entries(features)) mountSlot(shell.panes[tab], slot, (el) => f.mount(el, ctx));

// Phone-side look and feel: theme, density, signal unit and repaint rate apply now and on every change.
applyUiSettings(ctx.settings.phone.get().ui);
ctx.settings.phone.onChange((p) => applyUiSettings(p.ui));

// Connect / disconnect and the live link + key state live in the title bar, visible from every tab.
connection.mount(shell.bar, ctx);
// Device errors pop up over every tab.
alerts.mount(root, ctx);
