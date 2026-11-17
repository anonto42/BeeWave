import { $, coalesce, panel, type AppContext } from "../../shared/app";
import { channelActivity, channelOccupancy } from "../../shared/monitor";
import { chartSvg } from "./chart";

const html = `<div class="pad">
  <div class="legend"><span class="key ap"></span>access points <span class="key act"></span>reports in the last 60 s</div>
  <div data-chart></div>
  <div class="dim small">Channels 1-13. Activity is how many reports arrived, a proxy for how busy a channel is, not airtime use.</div>
</div>`;

/** Access points per channel, and a second series of recent activity; recomputed from the scan stream. */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("CHANNELS", html);
  root.append(el);
  const chart = $(el, "[data-chart]");
  const render = () => {
    const s = ctx.monitor.scans;
    chart.innerHTML = chartSvg(channelOccupancy(s.networks()), channelActivity(s.events(), Date.now()));
  };
  const later = coalesce(render, 250);
  ctx.monitor.scans.onChange(later);
  setInterval(render, 5000); // let the 60 s window slide even when nothing new arrives
  render();
}
