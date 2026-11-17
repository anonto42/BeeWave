/** The tab named by a URL hash ("#scan"), or `fallback` when it is empty or unknown. */
export function tabFromHash(hash: string, ids: readonly string[], fallback: string): string {
  const id = hash.replace(/^#/, "");
  return ids.includes(id) ? id : fallback;
}
