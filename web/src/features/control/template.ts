const channelOptions = Array.from({ length: 13 }, (_, i) => `<option>${i + 1}</option>`).join("");

export const controlHtml = `<div class="pad row">
  <button data-scan disabled>START SCAN</button>
  <button data-stop disabled>STOP</button>
  <select data-channel>${channelOptions}</select>
  <button data-set-channel disabled>SET CHANNEL</button>
  <button data-reboot disabled>REBOOT</button>
</div>`;
