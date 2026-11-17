import { $, coalesce, type AppContext } from "../../shared/app";
import { OPEN_SETTINGS_SECTION_EVENT } from "../../shared/shell";
import type { SectionEnv, SectionId } from "./env";
import { SECTIONS } from "./sections";
import { barHtml, bannerText, shellHtml } from "./template";

/**
 * The Settings surface: eleven sections, one Apply / Revert bar for everything stored on the device. Device-side
 * edits stay in a draft until Apply sends one `saveSettingsAll`; phone-side settings save as soon as they change.
 */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = document.createElement("div");
  el.className = "panel settings";
  el.innerHTML = shellHtml(SECTIONS);
  root.append(el);
  const model = ctx.settings.device;
  const refreshers: Array<() => void> = [];
  let active: SectionId = "connection";

  const deviceReady = () => ctx.auth.state === "authenticated" && model.loaded && model.support === "ok";
  const env: SectionEnv = {
    ctx,
    deviceReady,
    apply() {
      const r = model.beginSave();
      if (r && "send" in r) {
        ctx.transport.send({ type: "saveSettingsAll", settings: r.send }).catch((e) => {
          ctx.log(`saveSettingsAll failed: ${e}`);
          model.saveFailed();
        });
      }
    },
    open: (id) => show(id),
    onRefresh: (cb) => refreshers.push(cb),
  };

  const hosts = new Map<SectionId, HTMLElement>();
  for (const s of SECTIONS) {
    const host = $(el, `[data-section="${s.id}"]`);
    hosts.set(s.id, host);
    s.mount(host, env);
  }

  const show = (id: SectionId) => {
    active = id;
    for (const s of SECTIONS) {
      hosts.get(s.id)!.hidden = s.id !== id;
      el.querySelector(`[data-tab="${s.id}"]`)!.classList.toggle("active", s.id === id);
    }
    el.querySelector(`[data-tab="${id}"]`)?.scrollIntoView({ block: "nearest", inline: "center" });
  };
  $(el, ".stabs").onclick = (e) => {
    const id = (e.target as HTMLElement).closest<HTMLElement>("[data-tab]")?.dataset.tab as SectionId | undefined;
    if (id) show(id);
  };
  window.addEventListener(OPEN_SETTINGS_SECTION_EVENT, (e) => show((e as CustomEvent<SectionId>).detail));

  const bar = $(el, ".applybar");
  const render = () => {
    const n = model.dirtyCount();
    const ready = deviceReady();
    const saving = model.save.state === "saving";
    const b = bannerText(model.support, ctx.auth.state === "authenticated", model.loaded);
    const banner = $(el, "[data-banner]");
    banner.hidden = b === null;
    banner.textContent = b ?? "";
    for (const s of SECTIONS) {
      const host = hosts.get(s.id)!;
      host.classList.toggle("greyed", s.device && !ready);
      const dirty = s.device ? model.sectionDirty(s.id as "scan") : 0;
      const tab = el.querySelector(`[data-tab="${s.id}"]`)!;
      tab.toggleAttribute("data-dirty", dirty > 0);
    }
    bar.innerHTML = barHtml({ changes: n, saving, ready, state: model.save.state, reason: model.save.reason, revision: model.revision });
    bar.querySelector<HTMLButtonElement>("[data-apply]")!.onclick = () => env.apply();
    bar.querySelector<HTMLButtonElement>("[data-revert]")!.onclick = () => model.revert();
    refreshers.forEach((f) => f());
  };
  // The Apply bar is the one thing that must answer a tap at once (the spinner), so it is not held to the refresh-rate cap.
  const later = coalesce(render, 50);
  model.onChange(render);
  ctx.auth.onChange(render);
  ctx.onState(render);
  ctx.settings.phone.onChange(later);
  ctx.settings.state.onChange(later);
  show(active);
  render();
}
