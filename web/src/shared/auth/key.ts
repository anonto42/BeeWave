/** Device key length in bytes (64 hex characters). */
export const KEY_BYTES = 32;

/** Parse 64 hex characters into the key, or `null` if the text is not a valid key. */
export function parseKey(text: string): Uint8Array | null {
  const hex = text.trim();
  if (!/^[0-9a-fA-F]{64}$/.test(hex)) return null;
  return Uint8Array.from(hex.match(/../g)!.map((b) => parseInt(b, 16)));
}
