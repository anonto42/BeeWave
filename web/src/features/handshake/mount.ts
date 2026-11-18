import { $, coalesce, download, panel, type AppContext } from "../../shared/app";
import { buildPcap, captureFilename, pmkidToHashcat22000, type HandshakePair } from "../../shared/monitor";
import { emptyTargetsHtml, handshakeHtml, targetRowHtml } from "./template";


/**
 * Per target: pick which targeted networks to capture a handshake for, see progress (which of the 4
 * messages arrived) once something happens on its own, and export a complete one as `.cap`/`.pcap`.
 * Passively extracted PMKIDs (from EAPOL M1) are shown with a Hashcat 22000 export.
 *
 * The device has a single on/off switch (`SetHandshakeCapture`), not one per network, so this panel
 * keeps it on exactly while at least one target below has capture requested, and reads the shared
 * handshake store (fed from the device's own telemetry) filtered to each target's BSSID for display.
 */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("HANDSHAKE CAPTURE", handshakeHtml);
  root.append(el);
  const list = $(el, "[data-list]");
  const { handshake } = ctx.monitor;
  const { targets } = ctx;
  let connected = false;
  /** Which targets the owner asked to capture for. Session-only: it decides whether the device's single
   * switch is on, it is not itself sent anywhere. */
  const wanted = new Set<string>();
  /** The on/off state last sent to the device, so a repeated render does not resend it. `null` forces a
   * resend (used right after (re)connecting, since the device always starts off). */
  let lastRequested: boolean | null = null;

  const syncDevice = () => {
    const desired = wanted.size > 0;
    if (desired === lastRequested) return;
    lastRequested = desired;
    ctx.transport.send({ type: "setHandshakeCapture", enabled: desired }).catch((e) => ctx.log(`setHandshakeCapture failed: ${e}`));
  };

  const pairsFor = (bssid: string): HandshakePair[] => handshake.list().filter((p) => p.bssid === bssid);

  const render = () => {
    const ready = connected && ctx.auth.state === "authenticated";
    $(el, "[data-state]").textContent = `device: ${handshake.enabled ? "listening" : "off"}`;
    const items = targets.list();
    list.innerHTML = items.length
      ? items.map((t) => targetRowHtml(t, wanted.has(t.bssid), ready, pairsFor(t.bssid), handshake.pmkidsFor(t.bssid))).join("")
      : emptyTargetsHtml;
  };
  const later = coalesce(render);

  list.onclick = (e) => {
    const want = (e.target as HTMLElement).closest<HTMLElement>("[data-want]");
    if (want) {
      const bssid = want.closest<HTMLElement>("[data-target]")!.dataset.target!;
      if (wanted.has(bssid)) wanted.delete(bssid);
      else wanted.add(bssid);
      syncDevice();
      render();
      return;
    }
    // .cap / .pcap export (4-way handshake).
    const exp = (e.target as HTMLElement).closest<HTMLElement>("[data-export]");
    if (exp) {
      const { bssid, client, ext } = exp.dataset as { bssid: string; client: string; ext: "cap" | "pcap" };
      const pair = handshake.list().find((p) => p.bssid === bssid && p.client === client);
      const target = targets.list().find((t) => t.bssid === bssid);
      if (!pair || !target) return;
      const disambiguate = pairsFor(bssid).length > 1;
      download(captureFilename(pair, target.ssid, ext, disambiguate), "application/vnd.tcpdump.pcap", buildPcap(pair));
      return;
    }
    // Hashcat 22000 PMKID export.
    const pmkidBtn = (e.target as HTMLElement).closest<HTMLElement>("[data-export-pmkid]");
    if (pmkidBtn) {
      const { bssid, client, pmkid } = pmkidBtn.dataset as { bssid: string; client: string; pmkid: string };
      const target = targets.list().find((t) => t.bssid === bssid);
      const essid = target?.ssid ?? "";
      const record = handshake.pmkidsFor(bssid).find((r) => r.client === client && r.pmkid === pmkid);
      if (!record) return;
      const line = pmkidToHashcat22000(record, essid);
      const bytes = new TextEncoder().encode(line + "\n");
      const safeBssid = bssid.replace(/:/g, "");
      download(`${essid || safeBssid}_${safeBssid}.22000`, "text/plain", bytes);
    }
  };

  ctx.onState((s) => {
    connected = s === "connected";
    if (!connected) lastRequested = null; // the device forgets its state on disconnect; resync on the next one
    render();
  });
  ctx.auth.onChange(render);
  targets.onChange(render);
  handshake.onChange(later);
  render();
}
