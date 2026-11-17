const pad = (n: number) => String(n).padStart(2, "0");

/** HH:MM:SS in local time. */
export function clock(ms: number): string {
  const d = new Date(ms);
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}
