import { NAVIGATE_EVENT } from "./events";
import { tabFromHash } from "./tab-id";
import type { TabDef, TabShell } from "./types";

/** Builds the title bar, one pane per tab and the bottom tab bar inside `root`. */
export function createTabShell(root: HTMLElement, title: string, tabs: readonly TabDef[]): TabShell {
  const ids = tabs.map((t) => t.id);
  const header = document.createElement("header");
  header.className = "appbar";
  header.innerHTML = `<h1>${title}</h1><div class="appbar-slot"></div>`;

  const main = document.createElement("main");
  const panes: Record<string, HTMLElement> = {};
  for (const t of tabs) {
    const pane = document.createElement("section");
    pane.className = "pane";
    pane.id = `pane-${t.id}`;
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
    for (const t of tabs) {
      const on = t.id === id;
      panes[t.id].hidden = !on;
      buttons.get(t.id)!.classList.toggle("active", on);
      buttons.get(t.id)!.setAttribute("aria-current", on ? "page" : "false");
    }
    if (location.hash !== `#${id}`) history.replaceState(null, "", `#${id}`);
    window.scrollTo(0, 0);
  }

  window.addEventListener(NAVIGATE_EVENT, (e) => {
    const id = (e as CustomEvent<string>).detail;
    if (ids.includes(id)) select(id);
  });

  root.append(header, main, nav);
  select(tabFromHash(location.hash, ids, ids[0]));
  return { panes, bar: header.querySelector<HTMLElement>(".appbar-slot")!, select };
}
