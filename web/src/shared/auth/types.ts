/** Where the handshake with the device stands. */
export type AuthState =
  | "no-key" // no key saved yet
  | "idle" // not connected
  | "authenticating"
  | "authenticated"
  | "failed"; // the device rejected our proof (wrong key)
