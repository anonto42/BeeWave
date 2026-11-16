const channelOptions = `<option value="0">AUTO</option>` + Array.from({ length: 13 }, (_, i) => `<option>${i + 1}</option>`).join("");

export const settingsHtml = `<div class="pad row">
  <label>CHANNEL <select data-channel>${channelOptions}</select></label>
  <label><input type="checkbox" data-autoscan /> SCAN AT BOOT</label>
  <button data-save disabled>SAVE TO DEVICE</button>
  <span data-status class="dim"></span>
</div>`;
