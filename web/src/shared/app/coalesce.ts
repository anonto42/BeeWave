/** Run `fn` at most once per `ms`, after the last call in that window (store updates arrive in bursts). */
export function coalesce(fn: () => void, ms = 150): () => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return () => {
    if (timer !== undefined) return;
    timer = setTimeout(() => {
      timer = undefined;
      fn();
    }, ms);
  };
}
