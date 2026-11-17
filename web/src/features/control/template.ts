const channels = [0, ...Array.from({ length: 13 }, (_, i) => i + 1)];

export const channelButtons = channels
  .map((c) => `<button class="chip" type="button" data-ch="${c}" aria-pressed="${c === 0}" disabled>${c === 0 ? "AUTO" : c}</button>`)
  .join("");

export const controlHtml = `<div class="ctl">
  <div class="ctl-top"><span class="mode" data-mode>OFFLINE</span><span class="dim" data-hint></span></div>
  <button class="primary" data-run disabled>START SCAN</button>
  <div class="field-label">RADIO CHANNEL <span class="dim">AUTO hops 1-13, a number pins the radio</span></div>
  <div class="chipgrid" role="group" aria-label="Radio channel" data-channels>${channelButtons}</div>
</div>`;
