import { clock, esc } from "../../shared/app";
import { rssiHtml } from "../../shared/monitor";
import { vendorOf } from "../../shared/oui";
import type { DiscoveredClient } from "../../shared/monitor";

export const clientsHtml = `<div class="toolbar">
  <span class="dim grow small" data-count>0 client stations</span>
  <button type="button" class="small" data-clear>CLEAR</button>
</div>
<div class="tablewrap wide"><table class="grid">
  <thead><tr>
    <th>CLIENT MAC</th>
    <th>VENDOR</th>
    <th>RSSI</th>
    <th>CH</th>
    <th>PROBED NETWORKS (PNL)</th>
    <th>PROBES</th>
    <th>LAST SEEN</th>
  </tr></thead>
  <tbody data-body></tbody>
</table></div>`;

export const emptyRow = `<tr><td colspan="7" class="dim">No client probe requests detected yet.</td></tr>`;

export const clientRowHtml = (c: DiscoveredClient) => {
  const vendor = vendorOf(c.client);
  const pnl = c.probedSsids.length
    ? c.probedSsids.map((s) => `<code class="small">${esc(s)}</code>`).join(", ")
    : `<span class="dim small">(wildcard / broadcast)</span>`;
  return `<tr>
    <td><code>${esc(c.client)}</code></td>
    <td><span class="dim small">${esc(vendor)}</span></td>
    <td>${rssiHtml(c.rssi)}</td>
    <td>${c.channel}</td>
    <td>${pnl}</td>
    <td>${c.probeCount}</td>
    <td>${clock(c.lastSeen)}</td>
  </tr>`;
};
