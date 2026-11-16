import { panel, type AppContext } from "../../shared/app";
import { compatHtml } from "./template";

/** Shows a notice when Web Bluetooth is missing (and we are not in ?mock mode). */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const mock = new URLSearchParams(location.search).has("mock");
  if (mock || "bluetooth" in navigator) return;
  root.prepend(panel("UNSUPPORTED BROWSER", compatHtml));
  ctx.log("Web Bluetooth unavailable in this browser");
}
