import type { SectionMount } from "../env";
import { mountFields, phoneText, phoneToggle, readOnly } from "../fields";

/** Firmware identity (read-only) and the phone-side version check. Signing keys are build-time only. */
export const mount: SectionMount = (host, env) => {
  const { ctx } = env;
  const phone = ctx.settings.phone;
  const dev = ctx.settings.state;
  const version = readOnly("Firmware version", "-");
  const sha = readOnly("Git commit", "-");
  const built = readOnly("Built", "-");
  const fp = readOnly("OTA signing key fingerprint", "-", "First 16 bytes of the SHA-256 of the public key built into this firmware. It must equal the value printed by `make fw-fingerprint` (and shown by `make fw-sign`).", true);
  const warning = document.createElement("div");
  warning.className = "banner warn";
  warning.hidden = true;
  const docs = document.createElement("div");
  docs.className = "frow-hint";
  docs.innerHTML = "Updates must be signed. The first signed update needs one cable flash of a build that contains the public key; see <code>docs/TESTING.md#signed-ota</code>. There is no key editing here: signing keys are build-time only.";

  mountFields(host, env, [
    version, sha, built, fp,
    phoneToggle(env, "Check version on connect", () => phone.get().ota.autoCheckOnConnect, (v) => phone.update((s) => void (s.ota.autoCheckOnConnect = v)),
      "Compare the device's firmware version with the one you expect and warn when they differ."),
    phoneText(env, "Expected version", 16, () => phone.get().ota.expectedVersion, (v) => phone.update((s) => void (s.ota.expectedVersion = v)), "For example 0.1.0. Empty = no check."),
  ]);
  host.append(warning, docs);

  const refresh = () => {
    const i = dev.info;
    version.set(i?.version ?? "-");
    sha.set(i?.gitSha ?? "-");
    built.set(i?.built ?? "-");
    fp.set(i ? (/^0+$/.test(i.otaKeyFp) ? "none: updates are refused" : i.otaKeyFp) : "-");
    const want = phone.get().ota.expectedVersion;
    const mismatch = Boolean(i && phone.get().ota.autoCheckOnConnect && want && i.version !== want);
    warning.hidden = !mismatch;
    warning.textContent = mismatch ? `The device runs ${i!.version}, not the expected ${want}.` : "";
  };
  dev.onChange(refresh);
  env.onRefresh(refresh);
  refresh();
};
