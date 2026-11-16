import { parseKey } from "./key";

const STORAGE_KEY = "beewave.key";

/** The device key, kept in this browser only. Works in memory when storage is blocked. */
export class KeyStore {
  private key: Uint8Array | null = null;

  constructor(private readonly storage: Pick<Storage, "getItem" | "setItem" | "removeItem"> | null = safeStorage()) {
    try {
      const raw = storage?.getItem(STORAGE_KEY);
      this.key = raw ? parseKey(raw) : null;
    } catch {
      /* blocked storage: start without a key */
    }
  }

  get(): Uint8Array | null {
    return this.key;
  }

  /** Save a key typed by the user. Returns `false` (and keeps the old key) if it is not valid. */
  set(text: string): boolean {
    const key = parseKey(text);
    if (!key) return false;
    this.key = key;
    try {
      this.storage?.setItem(STORAGE_KEY, text.trim().toLowerCase());
    } catch {
      /* private mode / quota: keep working in memory */
    }
    return true;
  }

  clear(): void {
    this.key = null;
    try {
      this.storage?.removeItem(STORAGE_KEY);
    } catch {
      /* ignore */
    }
  }
}

function safeStorage(): Storage | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    return null;
  }
}
