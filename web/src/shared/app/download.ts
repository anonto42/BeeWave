/** Offer `data` to the user as a file download (assembled on the phone; nothing is sent anywhere). */
export function download(filename: string, mime: string, data: string | Uint8Array<ArrayBuffer>): void {
  const url = URL.createObjectURL(new Blob([data], { type: mime }));
  const a = Object.assign(document.createElement("a"), { href: url, download: filename });
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
