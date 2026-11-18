import { esc } from "../../shared/app";
import type { HandshakePair } from "../../shared/monitor";

export const handshakeHtml = `<div class="pad dim small">Listens for WPA/WPA2 four-way handshakes that happen on their own (a client joining or re-authenticating by itself). It never sends anything, including deauthentication frames: without an active client nearby, there is nothing to capture. It shares the radio with the phone link and the Wi-Fi sniffer, so it is off until you turn it on.</div>
<div class="toolbar">
  <button class="small" data-toggle disabled>TURN ON</button>
  <span class="dim grow small" data-state></span>
</div>
<ul class="nets" data-list></ul>`;

const dot = (seen: boolean) => `<span class="hs-dot${seen ? " on" : ""}" aria-hidden="true"></span>`;

export const rowHtml = (p: HandshakePair) => {
  const msgs = [1, 2, 3, 4].map((m) => dot(p.messages.has(m))).join("");
  const anyTruncated = [...p.messages.values()].some((m) => m.truncated);
  return `<li class="net static" data-pair="${p.bssid}|${p.client}">
    <span class="net-main"><b>${esc(p.bssid)}</b><small>client ${esc(p.client)}${anyTruncated ? " · truncated" : ""}</small></span>
    <span class="hs-msgs" title="messages seen: 1 2 3 4">${msgs}</span>
    <span class="dbm">${p.complete ? "complete" : "partial"}</span>
    <button type="button" class="small" data-export ${p.messages.size ? "" : "disabled"}>EXPORT</button>
  </li>`;
};

export const emptyHtml = (text: string) => `<li class="empty dim">${text}</li>`;
