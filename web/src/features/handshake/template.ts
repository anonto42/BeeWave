import { esc } from "../../shared/app";
import { rssiHtml, securityLabel, type ConnectedStation, type DiscoveredClient, type HandshakePair, type PmkidRecord, type SecurityRecord } from "../../shared/monitor";
import { vendorOf } from "../../shared/oui";
import type { Target } from "../../shared/targets";

export const handshakeHtml = `<div class="pad dim small">Target Audit Workspace: capture WPA/WPA2 handshakes (M1+M2 is sufficient for Hashcat mode 22000), clientless PMKIDs, connected stations, and active deauth threats. Pin the radio to a target's channel for 100% reliable 4-way capture.</div>
<div data-advisory></div>
<div class="toolbar"><span class="dim grow small" data-state></span></div>
<ul class="nets" data-list></ul>`;

export const emptyTargetsHtml = `<li class="empty dim">No targets selected yet. In the Scanner above, tap any network to add it to your Target Audit Workspace.</li>`;

const clientRowHtml = (bssid: string, p: HandshakePair | undefined, disambiguate: boolean) => {
  if (!p) {
    return `<li class="hs-client hs-waiting">
      <div class="hs-client-info">
        <span class="spinner"></span>
        <span class="small dim">Listening passively for client traffic...</span>
      </div>
      <div class="hs-step-pills dim">
        <span class="hs-step" title="Message 1: AP ANonce">M1</span>
        <span class="hs-step" title="Message 2: Client SNonce & MIC">M2</span>
        <span class="hs-step" title="Message 3: Install Keys">M3</span>
        <span class="hs-step" title="Message 4: Client ACK">M4</span>
      </div>
    </li>`;
  }

  const hasM1 = p.messages.has(1) || p.messages.has(3);
  const hasM2 = p.messages.has(2);
  const hasM3 = p.messages.has(3);
  const hasM4 = p.messages.has(4);
  const can22000 = hasM1 && hasM2;
  const isCrackable = can22000 || p.complete;
  const anyTruncated = [...p.messages.values()].some((m) => m.truncated);

  const statusBadge = p.complete
    ? `<span class="badge ok">FULL 4-WAY (4/4)</span>`
    : can22000
    ? `<span class="badge ok">CRACKABLE (M1+M2)</span>`
    : hasM1
    ? `<span class="badge warn">WAITING CLIENT (M1 SEEN)</span>`
    : `<span class="badge dim">WAITING FOR M1/M2</span>`;

  const exportBtn = (ext: "cap" | "pcap") =>
    `<button type="button" class="small" data-export data-bssid="${bssid}" data-client="${p.client}" data-ext="${ext}" ${isCrackable ? "" : "disabled"}>.${ext.toUpperCase()}</button>`;
  const btn22000 = `<button type="button" class="small" data-export-22000 data-bssid="${bssid}" data-client="${p.client}" ${can22000 ? "" : "disabled"}>.22000</button>`;
  const copyBtn = `<button type="button" class="small hs-copy-btn" data-copy-22000 data-bssid="${bssid}" data-client="${p.client}" ${can22000 ? "" : "disabled"}>COPY 22000</button>`;

  return `<li class="hs-client" data-bssid="${bssid}" data-client="${p.client}">
    <div class="hs-client-info">
      <span class="net-main">
        <small>${disambiguate ? `client ${esc(p.client)}` : "station " + esc(p.client)}${anyTruncated ? " · truncated" : ""}</small>
      </span>
      <div class="hs-step-pills" title="WPA/WPA2 4-way handshake progression">
        <span class="hs-step${hasM1 ? " on" : ""}" title="Message 1: AP ANonce">M1: ANonce</span>
        <span class="hs-step${hasM2 ? " on" : ""}" title="Message 2: Client SNonce & MIC">M2: SNonce/MIC</span>
        <span class="hs-step${hasM3 ? " on" : ""}" title="Message 3: Install Keys">M3: Keys</span>
        <span class="hs-step${hasM4 ? " on" : ""}" title="Message 4: Client ACK">M4: ACK</span>
      </div>
      ${statusBadge}
    </div>
    <div class="hs-actions">
      ${exportBtn("cap")}${exportBtn("pcap")}${btn22000}${copyBtn}
    </div>
  </li>`;
};

/** Renders PMKID rows captured from EAPOL M1 frames for a given target. */
const pmkidRowsHtml = (bssid: string, pmkids: readonly PmkidRecord[]) => {
  if (pmkids.length === 0) return "";
  return pmkids
    .map(
      (r) =>
        `<li class="hs-client hs-pmkid" data-bssid="${bssid}" data-client="${r.client}" data-pmkid="${r.pmkid}">
          <div class="hs-client-info">
            <span class="net-main">
              <small>clientless capture · client ${esc(r.client)}</small>
              <code class="dim small">${r.pmkid}</code>
            </span>
            <span class="badge ok">PMKID READY</span>
          </div>
          <div class="hs-actions">
            <button type="button" class="small" data-export-pmkid data-bssid="${bssid}" data-client="${r.client}" data-pmkid="${r.pmkid}">.22000</button>
            <button type="button" class="small hs-copy-btn" data-copy-pmkid data-bssid="${bssid}" data-client="${r.client}" data-pmkid="${r.pmkid}">COPY PMKID</button>
          </div>
        </li>`
    )
    .join("");
};

/** Renders actively associated client stations heard from 802.11 Data frame headers. */
const assocRowsHtml = (bssid: string, stations: readonly ConnectedStation[]) => {
  if (stations.length === 0) return "";
  return stations
    .map(
      (s) =>
        `<li class="hs-client hs-assoc" data-bssid="${bssid}" data-client="${s.client}">
          <div class="hs-client-info">
            <span class="net-main">
              <small>active connected station · ${esc(vendorOf(s.client))}</small>
              <code>${esc(s.client)}</code>
            </span>
            <span class="dim small">${s.frameCount} pkts</span>
          </div>
          <div class="hs-assoc-sig">
            <span class="dbm">${rssiHtml(s.rssi)}</span>
          </div>
        </li>`
    )
    .join("");
};

/** Renders probing client stations that requested this target's SSID. */
const probeRowsHtml = (ssid: string, probes: readonly DiscoveredClient[]) => {
  if (!ssid) return "";
  const matching = probes.filter((p) => p.probedSsids.includes(ssid));
  if (matching.length === 0) return "";
  return matching
    .slice(0, 3)
    .map(
      (c) =>
        `<li class="hs-client hs-probe" data-client="${c.client}">
          <div class="hs-client-info">
            <span class="net-main">
              <small>client probing for this SSID · ${esc(vendorOf(c.client))}</small>
              <code>${esc(c.client)}</code>
            </span>
            <span class="dim small">${c.probeCount} probes</span>
          </div>
          <div class="hs-assoc-sig">
            <span class="dbm">${rssiHtml(c.rssi)}</span>
          </div>
        </li>`
    )
    .join("");
};

/** Renders deauth threat alert if this AP is under attack. */
const deauthWarningHtml = (alertsCount: number) => {
  if (alertsCount <= 0) return "";
  return `<li class="hs-client hs-threat">
    <div class="hs-client-info">
      <span class="badge alert">DEAUTH FLOOD DETECTED</span>
      <span class="small dim">${alertsCount} deauth alerts triggered for this BSSID</span>
    </div>
  </li>`;
};

export const targetRowHtml = (
  t: Target,
  wanted: boolean,
  ready: boolean,
  pairs: readonly HandshakePair[],
  pmkids: readonly PmkidRecord[],
  stations: readonly ConnectedStation[] = [],
  probes: readonly DiscoveredClient[] = [],
  deauthCount = 0,
  targetChannel = 0,
  pinnedChannel = 0,
  security?: SecurityRecord,
) => {
  const disambiguate = pairs.length > 1;
  const rows = pairs.length
    ? pairs.map((p) => clientRowHtml(t.bssid, p, disambiguate)).join("")
    : clientRowHtml(t.bssid, undefined, false);

  const isChannelPinned = targetChannel > 0 && pinnedChannel === targetChannel;
  const channelLockBtn = targetChannel > 0
    ? `<button type="button" class="small lock-ch${isChannelPinned ? " on" : ""}" data-lock-ch="${targetChannel}" data-bssid="${t.bssid}" title="${isChannelPinned ? "Radio is locked to Channel " + targetChannel + ". Click to return to auto-hopping." : "Lock sniffer to Channel " + targetChannel + " for 100% reliable 4-way capture."}">${isChannelPinned ? `LOCKED CH ${targetChannel} ✓` : `LOCK CH ${targetChannel}`}</button>`
    : "";

  const channelAdvisory = isChannelPinned
    ? `<span class="hs-ch-status on">Radio locked on CH ${targetChannel} (100% focused)</span>`
    : targetChannel > 0
    ? `<span class="hs-ch-status dim">CH ${targetChannel} (Hopping across channels)</span>`
    : "";

  const secBadge = security ? `<span class="badge small">${securityLabel(security)}</span>` : "";

  return `<li class="hs-target" data-target="${t.bssid}">
    <div class="hs-target-head">
      <div class="net-main">
        <b>${esc(t.ssid || "(hidden)")}</b>
        <small>${t.bssid}${targetChannel ? ` · Channel ${targetChannel}` : ""}${stations.length ? ` · ${stations.length} station${stations.length === 1 ? "" : "s"}` : ""}</small>
      </div>
      <div class="hs-target-badges">
        ${secBadge}
      </div>
    </div>
    <div class="toolbar hs-toolbar">
      <button type="button" class="small${wanted ? " on" : ""}" data-want ${ready ? "" : "disabled"} aria-pressed="${wanted}">${wanted ? "CAPTURING ON" : "START CAPTURE"}</button>
      ${channelLockBtn}
      ${channelAdvisory}
    </div>
    <ul class="nets hs-clients">
      ${deauthWarningHtml(deauthCount)}
      ${rows}
      ${pmkidRowsHtml(t.bssid, pmkids)}
      ${assocRowsHtml(t.bssid, stations)}
      ${probeRowsHtml(t.ssid, probes)}
    </ul>
  </li>`;
};
