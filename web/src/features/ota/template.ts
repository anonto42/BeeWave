export const otaHtml = `<div class="pad">
  <div class="row">
    <input type="file" accept=".bin" data-file title="beewave-app.bin" />
    <input type="file" accept=".sig" data-sig title="beewave-app.bin.sig" />
    <button data-start disabled>UPDATE FIRMWARE</button>
  </div>
  <progress data-bar value="0" max="100" style="width:100%;margin-top:8px"></progress>
  <div data-status class="dim">select the image (beewave-app.bin) and its signature (beewave-app.bin.sig)</div>
</div>`;
