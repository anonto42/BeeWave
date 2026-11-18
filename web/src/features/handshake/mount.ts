import { $, download, panel, type AppContext } from "../../shared/app";
import { pairToHex } from "../../shared/monitor";
import { emptyHtml, handshakeHtml, rowHtml } from "./template";

const stamp = () => new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);

/** Pairs the passive handshake listener has captured, with a per-pair hex export. */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("HANDSHAKE CAPTURE", handshakeHtml);
  root.append(el);
  const toggle = $<HTMLButtonElement>(el, "[data-toggle]");
  const list = $(el, "[data-list]");
  const { handshake } = ctx.monitor;
  let connected = false;

  const render = () => {
    const ready = connected && ctx.auth.state === "authenticated";
    toggle.disabled = !ready;
    toggle.textContent = handshake.enabled ? "TURN OFF" : "TURN ON";
    $(el, "[data-state]").textContent = handshake.enabled ? "listening" : "off";
    const pairs = handshake.list();
    list.innerHTML = pairs.length ? pairs.map(rowHtml).join("") : emptyHtml(handshake.enabled ? "Listening. Nothing captured yet." : "Off.");
  };

  toggle.onclick = () =>
    ctx.transport.send({ type: "setHandshakeCapture", enabled: !handshake.enabled }).catch((e) => ctx.log(`setHandshakeCapture failed: ${e}`));
  list.onclick = (e) => {
    const row = (e.target as HTMLElement).closest<HTMLElement>("[data-pair]");
    if (!row || !(e.target as HTMLElement).closest("[data-export]")) return;
    const [bssid, client] = row.dataset.pair!.split("|");
    const pair = handshake.list().find((p) => p.bssid === bssid && p.client === client);
    if (pair) download(`beewave-handshake-${stamp()}.txt`, "text/plain", pairToHex(pair));
  };

  ctx.onState((s) => {
    connected = s === "connected";
    render();
  });
  ctx.auth.onChange(render);
  handshake.onChange(render);
  render();
}
