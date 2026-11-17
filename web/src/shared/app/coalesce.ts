let floorMs = 250;

/** The fastest any panel repaints, from the phone-side "refresh rate" setting (1-10 times a second). */
export function setRefreshHz(hz: number): void {
  floorMs = Math.round(1000 / Math.min(10, Math.max(1, hz)));
}

/** Run `fn` at most once per `ms` (but never faster than the refresh-rate setting), after the last call in that window. */
export function coalesce(fn: () => void, ms = 150): () => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return () => {
    if (timer !== undefined) return;
    timer = setTimeout(() => {
      timer = undefined;
      fn();
    }, Math.max(ms, floorMs));
  };
}
