import { crestSvg } from "../brand";
import { NAVIGATE_EVENT } from "./events";
import { tabFromHash } from "./tab-id";
import type { TabDef, TabShell } from "./types";

/** Builds the title bar, one pane per tab and the bottom tab bar inside `root`. */
export function createTabShell(root: HTMLElement, title: string, tabs: readonly TabDef[], badge?: string): TabShell {
  const ids = tabs.map((t) => t.id);
  const header = document.createElement("header");
  header.className = "appbar";
  header.innerHTML = `<span class="brand">${crestSvg(34)}<h1 class="sr-only">${title}</h1>${badge ? `<span class="badge">${badge}</span>` : ""}</span><div class="appbar-slot"></div>`;

  const main = document.createElement("main");
  const heading = document.createElement("h2"); // names the screen beside the sidebar; hidden on phones (style.css)
  heading.className = "pane-title";
  main.append(heading);
  const panes: Record<string, HTMLElement> = {};
  for (const t of tabs) {
    const pane = document.createElement("section");
    pane.className = "pane";
    pane.id = `pane-${t.id}`;
    pane.dataset.tab = t.id;
    pane.hidden = true;
    main.append(pane);
    panes[t.id] = pane;
  }

  const nav = document.createElement("nav");
  nav.className = "tabbar";
  nav.setAttribute("aria-label", "Sections");
  const buttons = new Map<string, HTMLButtonElement>();
  for (const t of tabs) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "tab";
    b.innerHTML = `${t.icon}<span>${t.label}</span>`;
    b.onclick = () => select(t.id);
    nav.append(b);
    buttons.set(t.id, b);
  }

  function select(id: string): void {
    root.dataset.tab = id; // CSS lays out each screen by this attribute
    heading.textContent = tabs.find((t) => t.id === id)?.label ?? "";
    for (const t of tabs) {
      const on = t.id === id;
      panes[t.id].hidden = !on;
      buttons.get(t.id)!.classList.toggle("active", on);
      buttons.get(t.id)!.setAttribute("aria-current", on ? "page" : "false");
    }
    if (location.hash !== `#${id}`) history.replaceState(null, "", `#${id}`);
    window.scrollTo(0, 0);
  }

  // The URL decides the screen: back/forward and edited hashes switch tabs too.
  window.addEventListener("hashchange", () => select(tabFromHash(location.hash, ids, ids[0])));
  window.addEventListener(NAVIGATE_EVENT, (e) => {
    const id = (e as CustomEvent<string>).detail;
    if (ids.includes(id)) select(id);
  });

  root.append(header, main, nav);
  select(tabFromHash(location.hash, ids, ids[0]));
  return { panes, bar: header.querySelector<HTMLElement>(".appbar-slot")!, select };
}
