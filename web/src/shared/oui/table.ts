/**
 * A small, best-effort local OUI table (first three BSSID bytes, lowercase, no separators) of vendors common
 * on consumer Wi-Fi. It is NOT the full IEEE registry (about 35,000 entries) and there is no network lookup:
 * anything not listed shows as "Unknown". Add rows as you verify them against the IEEE registry.
 */
export const OUI_VENDORS: Readonly<Record<string, string>> = {
  // Espressif (this board's own radio is 3c:71:bf)
  "3c71bf": "Espressif",
  "240ac4": "Espressif",
  "30aea4": "Espressif",
  "24a160": "Espressif",
  "7cdfa1": "Espressif",
  "a4cf12": "Espressif",
  "84f3eb": "Espressif",
  "cc50e3": "Espressif",
  "bcddc2": "Espressif",
  "ecfabc": "Espressif",
  // Raspberry Pi
  "b827eb": "Raspberry Pi",
  "dca632": "Raspberry Pi",
  "e45f01": "Raspberry Pi",
  // Apple
  "0017f2": "Apple",
  "001cb3": "Apple",
  "001ff3": "Apple",
  "0025bc": "Apple",
  "3c0754": "Apple",
  // TP-Link
  "50c7bf": "TP-Link",
  "ec086b": "TP-Link",
  "f4f26d": "TP-Link",
  "a42bb0": "TP-Link",
  "d807b6": "TP-Link",
  "14cc20": "TP-Link",
  // Ubiquiti
  "fcecda": "Ubiquiti",
  "24a43c": "Ubiquiti",
  "788a20": "Ubiquiti",
  "f09fc2": "Ubiquiti",
  "dc9fdb": "Ubiquiti",
  "802aa8": "Ubiquiti",
  "b4fbe4": "Ubiquiti",
  "0418d6": "Ubiquiti",
  // Netgear
  "00146c": "Netgear",
  "0024b2": "Netgear",
  "a021b7": "Netgear",
  "9c3dcf": "Netgear",
  // D-Link
  "00179a": "D-Link",
  "001cf0": "D-Link",
  "0022b0": "D-Link",
  "14d64d": "D-Link",
  "c8be19": "D-Link",
  // AVM (FRITZ!Box)
  "001f3f": "AVM",
  "3810d5": "AVM",
  "c80e14": "AVM",
  "2c3afd": "AVM",
  "7cff4d": "AVM",
  // Chipset / generic vendors
  "001018": "Broadcom",
  "00e04c": "Realtek",
  "0050f2": "Microsoft",
};
