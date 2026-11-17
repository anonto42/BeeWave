const svg = (body: string) =>
  `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">${body}</svg>`;

/** 24x24, 2 px stroke, square caps to match the boxy theme. */
export const ICONS = {
  device: svg(`<path d="M7 4l10 8-5 4V4l5 4-10 8" stroke-linecap="square"/>`),
  scan: svg(`<path d="M3 9a13 13 0 0 1 18 0M6 12.5a8.5 8.5 0 0 1 12 0M9 16a4 4 0 0 1 6 0" stroke-linecap="square"/><path d="M12 19.5v.01" stroke-linecap="square"/>`),
  settings: svg(`<path d="M4 7h10M18 7h2M4 17h2M10 17h10" stroke-linecap="square"/><path d="M14 4v6M6 14v6" stroke-linecap="square"/>`),
  log: svg(`<path d="M5 5h14M5 10h14M5 15h9M5 20h6" stroke-linecap="square"/>`),
} as const;
