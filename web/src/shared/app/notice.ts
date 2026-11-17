let box: HTMLElement | null = null;
let timer: ReturnType<typeof setTimeout> | undefined;

/** A small, short-lived message above the tab bar ("saved locally"). */
export function notice(text: string, ms = 1600): void {
  if (!box) {
    box = Object.assign(document.createElement("div"), { className: "notice", role: "status" });
    document.body.append(box);
  }
  box.textContent = text;
  box.classList.add("on");
  clearTimeout(timer);
  timer = setTimeout(() => box?.classList.remove("on"), ms);
}
