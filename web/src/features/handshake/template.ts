import { esc } from "../../shared/app";
import type { HandshakePair, PmkidRecord } from "../../shared/monitor";
import type { Target } from "../../shared/targets";

export const handshakeHtml = `<div class="pad dim small">Per target: capture the WPA/WPA2 four-way handshake of a network you have tapped as a target, only while it happens on its own (a client joining or re-authenticating by itself). Nothing is ever sent, including deauthentication frames — without an active client nearby, there is nothing to capture.</div>
<div class="toolbar"><span class="dim grow small" data-state></span></div>
<ul class="nets" data-list></ul>`;

export const emptyTargetsHtml = `<li class="empty dim">No targets yet. Tap a network in the Scanner, then capture its handshake here.</li>`;

const dot = (seen: boolean) => `<span class="hs-dot${seen ? " on" : ""}" aria-hidden="true"></span>`;

const clientRowHtml = (bssid: string, p: HandshakePair | undefined, disambiguate: boolean) => {
  if (!p) return `<li class="hs-client dim small">Not captured yet.</li>`;
  const msgs = [1, 2, 3, 4].map((m) => dot(p.messages.has(m))).join("");
  const anyTruncated = [...p.messages.values()].some((m) => m.truncated);
  const exportBtn = (ext: "cap" | "pcap") =>
    `<button type="button" class="small" data-export data-bssid="${bssid}" data-client="${p.client}" data-ext="${ext}" ${p.complete ? "" : "disabled"}>.${ext.toUpperCase()}</button>`;
  return `<li class="hs-client" data-bssid="${bssid}" data-client="${p.client}">
    <span class="net-main"><small>${disambiguate ? `client ${esc(p.client)}` : "handshake"}${anyTruncated ? " · truncated" : ""}</small></span>
    <span class="hs-msgs" title="messages seen: 1 2 3 4">${msgs}</span>
    <span class="dbm">${p.complete ? "complete" : "partial"}</span>
    ${exportBtn("cap")}${exportBtn("pcap")}
  </li>`;
};

/** Renders PMKID rows captured from EAPOL M1 frames for a given target. */
const pmkidRowsHtml = (bssid: string, pmkids: readonly PmkidRecord[]) => {
  if (pmkids.length === 0) return "";
  return pmkids
    .map(
      (r) =>
        `<li class="hs-client hs-pmkid" data-bssid="${bssid}" data-client="${r.client}" data-pmkid="${r.pmkid}">
          <span class="net-main"><small>PMKID · client ${esc(r.client)}</small><code class="dim small">${r.pmkid}</code></span>
          <button type="button" class="small" data-export-pmkid data-bssid="${bssid}" data-client="${r.client}" data-pmkid="${r.pmkid}">.22000</button>
        </li>`
    )
    .join("");
};

export const targetRowHtml = (
  t: Target,
  wanted: boolean,
  ready: boolean,
  pairs: readonly HandshakePair[],
  pmkids: readonly PmkidRecord[],
) => {
  const disambiguate = pairs.length > 1;
  const rows = pairs.length ? pairs.map((p) => clientRowHtml(t.bssid, p, disambiguate)).join("") : clientRowHtml(t.bssid, undefined, false);
  return `<li class="hs-target" data-target="${t.bssid}">
    <div class="net-main"><b>${esc(t.ssid || "(hidden)")}</b><small>${t.bssid}</small></div>
    <div class="toolbar hs-toolbar">
      <button type="button" class="small${wanted ? " on" : ""}" data-want ${ready ? "" : "disabled"} aria-pressed="${wanted}">${wanted ? "CAPTURING" : "CAPTURE HANDSHAKE"}</button>
    </div>
    <ul class="nets hs-clients">${rows}${pmkidRowsHtml(t.bssid, pmkids)}</ul>
  </li>`;
};
