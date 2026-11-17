import { clock } from "../../../shared/app";
import { confirmButton } from "../confirm";
import type { SectionMount } from "../env";
import { mountFields, readOnly } from "../fields";

const STEP = (ok: boolean) => (ok ? "pass" : "FAIL");

/** Live mirror of Status, reboot, factory reset, self-test and the build-time facts. */
export const mount: SectionMount = (host, env) => {
  const { ctx } = env;
  const dev = ctx.settings.state;
  const heap = readOnly("Free heap", "-");
  const channel = readOnly("Current channel", "-");
  const since = readOnly("Connected since", "-", "The firmware does not report its uptime; this is when this phone connected.");
  const test = document.createElement("div");
  test.className = "frow";
  test.innerHTML = `<div class="frow-head"><span class="frow-label">Self-test</span></div><div class="row"><button type="button" data-test>RUN SELF-TEST</button></div><div class="frow-hint" data-result>Checks radio, NVS, Bluetooth and the update slot. Stop any scan first.</div>`;
  const runBtn = test.querySelector<HTMLButtonElement>("[data-test]")!;
  runBtn.onclick = () => {
    dev.startSelfTest();
    ctx.transport.send({ type: "runSelfTest" }).catch((e) => ctx.log(`runSelfTest failed: ${e}`));
  };

  const reboot = confirmButton({
    label: "REBOOT DEVICE",
    confirmLabel: "CONFIRM REBOOT",
    warning: "Restarts the device. The link drops; the saved settings and key stay.",
    onConfirm: () => ctx.transport.send({ type: "reboot" }).catch((e) => ctx.log(`reboot failed: ${e}`)),
  });
  const reset = confirmButton({
    label: "FACTORY RESET",
    confirmLabel: "ERASE EVERYTHING AND REBOOT",
    word: "RESET",
    warning: "Erases the device's settings AND its claimed key, then reboots it unclaimed. This phone forgets the key and device name too. Other phones lose access.",
    onConfirm: () => {
      ctx.transport.send({ type: "factoryReset" }).catch((e) => ctx.log(`factoryReset failed: ${e}`));
      ctx.keys.clear();
      ctx.auth.reset();
      ctx.settings.phone.update((s) => void (s.session.lastDeviceName = ""));
    },
  });
  const actions = document.createElement("div");
  actions.className = "frow";
  actions.append(Object.assign(document.createElement("div"), { className: "frow-head", innerHTML: '<span class="frow-label">Actions</span>' }), reboot.el, reset.el);

  const build = {
    conn: readOnly("REQUEST_CONN_PARAMS", "-", "Build-time. Turning it on needs a firmware rebuild and a bench measurement; see docs/TESTING.md."),
    coex: readOnly("coex_debug feature", "-", "Build-time cargo feature: set when the firmware was built with FEATURES=coex_debug."),
    bonding: readOnly("BLE bonding", "-", "Optional hardening that is not enabled. Commands are protected by the device key instead."),
  };
  const hint = document.createElement("div");
  hint.className = "frow-hint";
  hint.textContent = "Build-time values are read only: they change only by rebuilding the firmware.";

  mountFields(host, env, [heap, channel, since]);
  host.append(test, actions, Object.assign(document.createElement("h3"), { className: "subhead", textContent: "Build-time (read only)" }), build.conn.el, build.coex.el, build.bonding.el, hint);

  let connectedAt = 0;
  ctx.onState((s) => (connectedAt = s === "connected" ? Date.now() : 0));
  ctx.onTelemetry("status", (m) => {
    heap.set(`${(m.freeHeap / 1024).toFixed(0)} KB`);
    channel.set(m.activeChannel === 0 ? "AUTO" : String(m.activeChannel));
  });

  const refresh = () => {
    const ready = env.deviceReady();
    runBtn.disabled = !ready || dev.selfTestRunning;
    reboot.setDisabled(!ready, "Available once connected and authenticated.");
    reset.setDisabled(!ready, "Available once connected and authenticated.");
    since.set(connectedAt ? clock(connectedAt) : "-");
    const r = dev.selfTest;
    test.querySelector("[data-result]")!.textContent = dev.selfTestRunning
      ? "running..."
      : r?.busy
        ? "Busy: a scan is running. Stop the scan on the Scan tab, then run the test again."
        : r
          ? `Radio ${STEP(r.radio)}, NVS ${STEP(r.nvs)}, Bluetooth ${STEP(r.ble)}, update slot ${STEP(r.otaSlot)}`
          : "Checks radio, NVS, Bluetooth and the update slot. Stop any scan first.";
    const i = dev.info;
    build.conn.set(i ? (i.requestConnParams ? "true" : "false") : "-");
    build.coex.set(i ? (i.coexDebug ? "on in this build" : "off in this build") : "-");
    build.bonding.set(i ? (i.bonding ? "enabled" : "Not enabled") : "-");
    if (dev.lastRejected === "busy") test.querySelector("[data-result]")!.textContent = "The device refused: busy (an update is in progress).";
  };
  dev.onChange(refresh);
  env.onRefresh(refresh);
  refresh();
};
