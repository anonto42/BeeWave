/** Device key length in bytes (64 hex characters). */
export const KEY_BYTES = 32;

/** A fresh random device key, for claiming an unclaimed device. */
export function randomKey(): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(KEY_BYTES));
}

/** The key as 64 lowercase hex characters (what the key store saves and the user can paste). */
export function formatKey(key: Uint8Array): string {
  return [...key].map((b) => b.toString(16).padStart(2, "0")).join("");
}

/** Parse 64 hex characters into the key, or `null` if the text is not a valid key. */
export function parseKey(text: string): Uint8Array | null {
  const hex = text.trim();
  if (!/^[0-9a-fA-F]{64}$/.test(hex)) return null;
  return Uint8Array.from(hex.match(/../g)!.map((b) => parseInt(b, 16)));
}
