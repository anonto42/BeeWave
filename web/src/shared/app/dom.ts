/** Typed querySelector. */
export const $ = <T extends HTMLElement>(root: ParentNode, selector: string) => root.querySelector(selector) as T;

/** Escape text before putting it in innerHTML. */
export const esc = (s: string) => s.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);

/** A bordered panel with a title row. */
export function panel(title: string, bodyHtml: string): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `<h2>${title}</h2>${bodyHtml}`;
  return el;
}
