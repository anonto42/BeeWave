// Composition root: choose transport, create context, build the tab shell, mount feature slices into tabs.
import "./style.css";
import { registerSW } from "virtual:pwa-register";
import { createContext } from "./shared/app";
import { createTabShell, ICONS } from "./shared/shell";
import { BleTransport, MockTransport } from "./shared/transport";
import * as connection from "./features/connection";
import * as auth from "./features/auth";
import * as status from "./features/status";
import * as control from "./features/control";
import * as scanner from "./features/scanner";
import * as log from "./features/log";
import * as ota from "./features/ota";
import * as settings from "./features/settings";
import * as targets from "./features/targets";
import * as compat from "./features/compat";
import * as alerts from "./features/alerts";
import * as reboot from "./features/reboot";
import * as results from "./features/results";
import * as channelChart from "./features/channel-chart";
import * as rssiHistory from "./features/rssi-history";
import * as compare from "./features/compare";
import * as anomalies from "./features/anomalies";
import * as deauth from "./features/deauth";
import * as bleMonitor from "./features/ble-monitor";
import * as watchlist from "./features/watchlist";
import * as exportSession from "./features/export";

registerSW({ immediate: true });

const mock = new URLSearchParams(location.search).has("mock");
const ctx = createContext(mock ? new MockTransport() : new BleTransport());

const root = document.getElementById("app")!;
const shell = createTabShell(root, `BEEWAVE${mock ? " [MOCK]" : ""}`, [
  { id: "device", label: "Device", icon: ICONS.device },
  { id: "scan", label: "Scan", icon: ICONS.scan },
  { id: "analyze", label: "Analyze", icon: ICONS.analyze },
  { id: "alerts", label: "Alerts", icon: ICONS.alerts },
  { id: "settings", label: "Settings", icon: ICONS.settings },
  { id: "log", label: "Log", icon: ICONS.log },
]);

const layout = {
  device: [compat, status],
  scan: [control, scanner, targets],
  analyze: [results, channelChart, rssiHistory, compare],
  alerts: [anomalies, deauth, bleMonitor, watchlist],
  settings: [settings, auth, reboot, exportSession, ota],
  log: [log],
} as const;
for (const [tab, features] of Object.entries(layout)) for (const f of features) f.mount(shell.panes[tab], ctx);

// Connect / disconnect and the live link + key state live in the title bar, visible from every tab.
connection.mount(shell.bar, ctx);
// Device errors pop up over every tab.
alerts.mount(root, ctx);
