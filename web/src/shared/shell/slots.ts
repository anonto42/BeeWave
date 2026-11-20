/** Mounts one feature into `pane` and tags the elements it added with `data-slot`, so CSS can place it by name. */
export function mountSlot(pane: HTMLElement, slot: string, mount: (root: HTMLElement) => void): void {
  const before = new Set(Array.from(pane.children));
  mount(pane);
  for (const el of Array.from(pane.children)) if (!before.has(el)) (el as HTMLElement).dataset.slot = slot;
}
