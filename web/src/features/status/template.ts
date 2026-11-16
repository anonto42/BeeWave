const cell = (key: string, label: string) => `<div><b data-${key}>-</b><span>${label}</span></div>`;

export const statusHtml = `<div class="stat">
  ${cell("channel", "CHANNEL")}${cell("heap", "FREE HEAP")}${cell("cpu", "CPU %")}${cell("dropped", "DEV DROPPED")}${cell("lost", "BLE LOST")}
</div>`;
