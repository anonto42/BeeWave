import { $, coalesce, download, panel, type AppContext } from "../../shared/app";
import { buildMultiPcap, buildPcap, captureFilename, exportAll22000, handshakeToHashcat22000, pmkidToHashcat22000, type HandshakePair } from "../../shared/monitor";
import { emptyTargetsHtml, handshakeHtml, targetRowHtml } from "./template";

function copyToClipboard(text: string, btn: HTMLButtonElement): void {
  const orig = btn.textContent;
  const showFeedback = () => {
    btn.textContent = "COPIED!";
    btn.disabled = true;
    setTimeout(() => {
      btn.textContent = orig;
      btn.disabled = false;
    }, 1500);
  };

  if (navigator.clipboard?.writeText) {
    navigator.clipboard.writeText(text).then(showFeedback).catch(() => {
      fallbackCopy(text);
      showFeedback();
    });
  } else {
    fallbackCopy(text);
    showFeedback();
  }
}

function fallbackCopy(text: string): void {
  const ta = document.createElement("textarea");
  ta.value = text;
  ta.style.position = "fixed";
  ta.style.opacity = "0";
  document.body.appendChild(ta);
  ta.focus();
  ta.select();
  try {
    document.execCommand("copy");
  } catch {
    // Ignore copy error in unsupported environment
  } finally {
    document.body.removeChild(ta);
  }
}

/**
 * Target Audit Workspace: captures 4-way handshakes, PMKIDs, connected stations, and active deauth threats.
 * Lets the user lock reception to a target's channel for 100% reliable capture, with multi-target guidance.
 */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("TARGET AUDIT & HANDSHAKE WORKSPACE", handshakeHtml);
  root.append(el);
  const list = $(el, "[data-list]");
  const advisory = $(el, "[data-advisory]");
  const { handshake } = ctx.monitor;
  const { targets } = ctx;
  let connected = false;

  /** Which targets the owner asked to capture for. */
  const wanted = new Set<string>();
  let lastRequested: boolean | null = null;
  let currentPinnedChannel = ctx.settings.device.draft?.radio.defaultChannel ?? 0;

  const syncDevice = () => {
    const desired = wanted.size > 0;
    if (desired === lastRequested) return;
    lastRequested = desired;
    ctx.transport.send({ type: "setHandshakeCapture", enabled: desired }).catch((e) => ctx.log(`setHandshakeCapture failed: ${e}`));
  };

  const pairsFor = (bssid: string): HandshakePair[] => handshake.list().filter((p) => p.bssid === bssid);

  const render = () => {
    const ready = connected && ctx.auth.state === "authenticated";
    $(el, "[data-state]").textContent = `device sniffer: ${handshake.enabled ? "listening" : "off"}${currentPinnedChannel ? ` · locked CH ${currentPinnedChannel}` : " · hopping"}`;
    const items = targets.list();

    if (items.length > 1) {
      const distinct = [...new Set(items.map((t) => ctx.monitor.scans.get(t.bssid)?.channel).filter((c): c is number => typeof c === "number" && c > 0))];
      if (distinct.length > 1) {
        advisory.innerHTML = `<div class="hs-advisory"><span class="dim small"><b>Multi-Target Mode:</b> Selected targets operate on distinct channels (${distinct.map((c) => "CH " + c).join(", ")}). The sniffer hops across channels. For 100% reliable 4-way capture, click <b>LOCK CH</b> on a priority target to lock reception.</span></div>`;
      } else if (distinct.length === 1 && currentPinnedChannel !== distinct[0]) {
        advisory.innerHTML = `<div class="hs-advisory ok"><span class="dim small">All targets operate on <b>Channel ${distinct[0]}</b>. Click <b>LOCK CH ${distinct[0]}</b> below to lock reception and prevent missed packets.</span></div>`;
      } else {
        advisory.innerHTML = "";
      }
    } else if (items.length === 1) {
      const ch = ctx.monitor.scans.get(items[0].bssid)?.channel ?? 0;
      if (ch > 0 && currentPinnedChannel !== ch) {
        advisory.innerHTML = `<div class="hs-advisory ok"><span class="dim small">Target operates on <b>Channel ${ch}</b>. Click <b>LOCK CH ${ch}</b> below to lock reception and guarantee capture.</span></div>`;
      } else {
        advisory.innerHTML = "";
      }
    } else {
      advisory.innerHTML = "";
    }

    list.innerHTML = items.length
      ? items.map((t) => {
          const net = ctx.monitor.scans.get(t.bssid);
          const channel = net?.channel ?? 0;
          const security = net?.security;
          const deauthCount = ctx.monitor.state.deauth.get(t.bssid)?.alerts ?? 0;
          const stations = ctx.monitor.assoc.stationsFor(t.bssid);
          const probes = ctx.monitor.probes.list();
          return targetRowHtml(
            t,
            wanted.has(t.bssid),
            ready,
            pairsFor(t.bssid),
            handshake.pmkidsFor(t.bssid),
            stations,
            probes,
            deauthCount,
            channel,
            currentPinnedChannel,
            security
          );
        }).join("")
      : emptyTargetsHtml;

    const allPairs = handshake.list();
    const allPmkids = handshake.allPmkids();
    const crackablePairs = allPairs.filter((p) => (p.messages.has(1) || p.messages.has(3)) && p.messages.has(2));
    const totalReadyCaptures = crackablePairs.length + allPmkids.length;
    const totalFrames = allPairs.reduce((n, p) => n + p.messages.size, 0);

    const batchActions = $(el, "[data-batch-actions]");
    if (totalFrames > 0 || totalReadyCaptures > 0) {
      batchActions.style.display = "inline-flex";
      $(el, "[data-all-pcap-count]").textContent = String(allPairs.length);
      $(el, "[data-all-22000-count]").textContent = String(totalReadyCaptures);
      $<HTMLButtonElement>(el, "[data-export-all-pcap]").disabled = totalFrames === 0;
      $<HTMLButtonElement>(el, "[data-export-all-22000]").disabled = totalReadyCaptures === 0;
      $<HTMLButtonElement>(el, "[data-copy-all-22000]").disabled = totalReadyCaptures === 0;
    } else {
      batchActions.style.display = "none";
    }
  };
  const later = coalesce(render);

  el.onclick = (e) => {
    // Batch export all captured frames to a single PCAP
    const allPcapBtn = (e.target as HTMLElement).closest<HTMLElement>("[data-export-all-pcap]");
    if (allPcapBtn) {
      const pairs = handshake.list();
      if (pairs.length === 0) return;
      const bytes = buildMultiPcap(pairs);
      const stamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
      download(`beewave_handshakes_${stamp}.pcap`, "application/vnd.tcpdump.pcap", bytes);
      return;
    }

    // Batch export all crackable handshakes and PMKIDs in Hashcat mode 22000 format
    const all22000Btn = (e.target as HTMLElement).closest<HTMLElement>("[data-export-all-22000]");
    if (all22000Btn) {
      const pairs = handshake.list();
      const pmkids = handshake.allPmkids();
      const text = exportAll22000(
        pairs,
        pmkids,
        (bssid) => ctx.monitor.scans.get(bssid)?.ssid ?? targets.list().find((t) => t.bssid === bssid)?.ssid ?? ""
      );
      if (!text) return;
      const stamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
      const bytes = new TextEncoder().encode(text);
      download(`beewave_hashes_${stamp}.22000`, "text/plain", bytes);
      return;
    }

    // Batch copy all 22000 hashes to clipboard
    const copyAll22000Btn = (e.target as HTMLElement).closest<HTMLElement>("[data-copy-all-22000]");
    if (copyAll22000Btn) {
      const pairs = handshake.list();
      const pmkids = handshake.allPmkids();
      const text = exportAll22000(
        pairs,
        pmkids,
        (bssid) => ctx.monitor.scans.get(bssid)?.ssid ?? targets.list().find((t) => t.bssid === bssid)?.ssid ?? ""
      );
      if (!text) return;
      copyToClipboard(text, copyAll22000Btn as HTMLButtonElement);
      return;
    }

    // 1-Click Channel Lock
    const lockBtn = (e.target as HTMLElement).closest<HTMLElement>("[data-lock-ch]");
    if (lockBtn) {
      const ch = Number(lockBtn.dataset.lockCh);
      const isCurrentlyPinned = currentPinnedChannel === ch;
      const newCh = isCurrentlyPinned ? 0 : ch;
      currentPinnedChannel = newCh;
      ctx.settings.device.set("radio.defaultChannel", newCh);
      ctx.transport.send({ type: "configureRadio", channel: newCh }).catch((err) => ctx.log(`configureRadio failed: ${err}`));
      render();
      return;
    }

    // Toggle capture on target
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

    // Hashcat 22000 4-way handshake file export (.22000).
    const exp22000 = (e.target as HTMLElement).closest<HTMLElement>("[data-export-22000]");
    if (exp22000) {
      const { bssid, client } = exp22000.dataset as { bssid: string; client: string };
      const pair = handshake.list().find((p) => p.bssid === bssid && p.client === client);
      const target = targets.list().find((t) => t.bssid === bssid);
      if (!pair || !target) return;
      const essid = target.ssid ?? "";
      const line = handshakeToHashcat22000(pair, essid);
      if (!line) return;
      const bytes = new TextEncoder().encode(line + "\n");
      const safeBssid = bssid.replace(/:/g, "");
      download(`${essid || safeBssid}_${safeBssid}.22000`, "text/plain", bytes);
      return;
    }

    // Hashcat 22000 4-way handshake clipboard copy.
    const copy22000 = (e.target as HTMLElement).closest<HTMLElement>("[data-copy-22000]");
    if (copy22000) {
      const btn = copy22000 as HTMLButtonElement;
      const { bssid, client } = copy22000.dataset as { bssid: string; client: string };
      const pair = handshake.list().find((p) => p.bssid === bssid && p.client === client);
      const target = targets.list().find((t) => t.bssid === bssid);
      if (!pair || !target) return;
      const essid = target.ssid ?? "";
      const line = handshakeToHashcat22000(pair, essid);
      if (!line) return;
      copyToClipboard(line, btn);
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
      return;
    }

    // Hashcat 22000 PMKID clipboard copy.
    const copyPmkid = (e.target as HTMLElement).closest<HTMLElement>("[data-copy-pmkid]");
    if (copyPmkid) {
      const btn = copyPmkid as HTMLButtonElement;
      const { bssid, client, pmkid } = copyPmkid.dataset as { bssid: string; client: string; pmkid: string };
      const target = targets.list().find((t) => t.bssid === bssid);
      const essid = target?.ssid ?? "";
      const record = handshake.pmkidsFor(bssid).find((r) => r.client === client && r.pmkid === pmkid);
      if (!record) return;
      const line = pmkidToHashcat22000(record, essid);
      copyToClipboard(line, btn);
      return;
    }
  };

  ctx.onState((s) => {
    connected = s === "connected";
    if (!connected) lastRequested = null;
    render();
  });
  ctx.auth.onChange(render);
  targets.onChange(render);
  handshake.onChange(later);
  ctx.monitor.scans.onChange(later);
  ctx.monitor.state.onChange(later);
  ctx.monitor.assoc.onChange(later);
  ctx.monitor.probes.onChange(later);
  ctx.settings.device.onChange(() => {
    currentPinnedChannel = ctx.settings.device.draft?.radio.defaultChannel ?? 0;
    later();
  });
  ctx.onTelemetry("status", (m) => {
    if (m.type === "status" && currentPinnedChannel !== m.activeChannel && m.activeChannel > 0) {
      currentPinnedChannel = m.activeChannel;
      later();
    }
  });

  render();
}
