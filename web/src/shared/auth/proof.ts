import { AUTH_CONTEXT } from "../protocol";

/** `HMAC-SHA256(key, AUTH_CONTEXT || nonce)`: the proof the device checks. */
export async function computeProof(key: Uint8Array, nonce: Uint8Array): Promise<Uint8Array<ArrayBuffer>> {
  const cryptoKey = await crypto.subtle.importKey(
    "raw",
    key as Uint8Array<ArrayBuffer>,
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const context = new TextEncoder().encode(AUTH_CONTEXT);
  const message = new Uint8Array(context.length + nonce.length);
  message.set(context);
  message.set(nonce, context.length);
  return new Uint8Array(await crypto.subtle.sign("HMAC", cryptoKey, message));
}
