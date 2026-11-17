import { esc, notice } from "../../shared/app";
import { REASON_TEXT, getField, type FieldPath } from "../../shared/settings";
import type { SectionEnv } from "./env";

/** A control plus the function that brings it back in line with the data. */
export interface Bound {
  el: HTMLElement;
  sync(): void;
}

let uid = 0;
const id = (p: string) => `f${++uid}-${p.replace(/\W/g, "")}`;

/** The row every setting sits in: label, control, optional hint; marked when changed or refused. */
function row(label: string, control: HTMLElement, hint?: string): { el: HTMLElement; state(s: { dirty: boolean; err: string | null; off: boolean; why?: string }): void } {
  const el = document.createElement("div");
  el.className = "frow";
  el.innerHTML = `<div class="frow-head"><span class="frow-label">${esc(label)}</span><span class="frow-mark" aria-hidden="true"></span></div><div class="frow-body"></div>${hint ? `<div class="frow-hint">${esc(hint)}</div>` : ""}<div class="frow-err" role="alert"></div>`;
  el.querySelector(".frow-body")!.append(control);
  return {
    el,
    state({ dirty, err, off, why }) {
      el.classList.toggle("dirty", dirty);
      el.classList.toggle("err", err !== null);
      el.classList.toggle("off", off);
      el.title = off && why ? why : "";
      el.querySelector(".frow-mark")!.textContent = dirty ? "changed" : "";
      el.querySelector(".frow-err")!.textContent = err ?? "";
    },
  };
}

const OFF_WHY = "Available once connected, authenticated and the device has sent its settings.";

/** Shared glue for device-side fields: reads and writes the draft, shows changed / refused state. */
function deviceField(env: SectionEnv, path: FieldPath, label: string, control: HTMLElement, read: () => void, hint?: string, extraOff?: () => boolean): Bound {
  const r = row(label, control, hint);
  const model = env.ctx.settings.device;
  return {
    el: r.el,
    sync() {
      read();
      const off = !env.deviceReady() || (extraOff?.() ?? false);
      const err = model.save.state === "rejected" && model.save.field === path && model.save.reason ? REASON_TEXT[model.save.reason] : null;
      r.state({ dirty: model.isDirty(path), err, off, why: OFF_WHY });
      control.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLButtonElement>("input, select, button").forEach((c) => (c.disabled = off));
    },
  };
}

export function toggle(env: SectionEnv, path: FieldPath, label: string, hint?: string): Bound {
  const input = Object.assign(document.createElement("input"), { type: "checkbox", id: id(path) });
  const wrap = document.createElement("label");
  wrap.className = "switch";
  wrap.append(input, Object.assign(document.createElement("span"), { textContent: "on" }));
  const model = env.ctx.settings.device;
  input.onchange = () => model.set(path, input.checked);
  return deviceField(env, path, label, wrap, () => (input.checked = model.draft ? Boolean(getField(model.draft, path)) : false), hint);
}

export interface SliderOpts {
  min: number;
  max: number;
  step: number;
  /** Text shown next to the value (for example "250 ms" or "4 Hz"). */
  show?: (v: number) => string;
}

/** A slider with a number box; the label shows the current value. */
export function slider(env: SectionEnv, path: FieldPath, label: string, o: SliderOpts, hint?: string): Bound {
  const range = Object.assign(document.createElement("input"), { type: "range", min: String(o.min), max: String(o.max), step: String(o.step) });
  const num = Object.assign(document.createElement("input"), { type: "number", min: String(o.min), max: String(o.max), step: String(o.step), inputMode: "numeric" });
  const out = document.createElement("span");
  out.className = "frow-value";
  const wrap = document.createElement("div");
  wrap.className = "slider";
  wrap.append(range, num, out);
  const model = env.ctx.settings.device;
  const set = (v: number) => {
    if (!Number.isNaN(v)) model.set(path, v);
  };
  range.oninput = () => set(Number(range.value));
  num.onchange = () => set(Number(num.value));
  return deviceField(env, path, label, wrap, () => {
    const v = model.draft ? Number(getField(model.draft, path)) : o.min;
    range.value = num.value = String(v);
    out.textContent = o.show ? o.show(v) : "";
  }, hint);
}

export function textField(env: SectionEnv, path: FieldPath, label: string, max: number, hint?: string): Bound {
  const input = Object.assign(document.createElement("input"), { type: "text", maxLength: max, autocomplete: "off", spellcheck: false });
  const clear = Object.assign(document.createElement("button"), { type: "button", className: "small", textContent: "CLEAR" });
  const wrap = document.createElement("div");
  wrap.className = "textrow";
  wrap.append(input, clear);
  const model = env.ctx.settings.device;
  input.oninput = () => model.set(path, input.value);
  clear.onclick = () => model.set(path, "");
  return deviceField(env, path, label, wrap, () => {
    const v = model.draft ? String(getField(model.draft, path)) : "";
    if (document.activeElement !== input) input.value = v;
  }, hint);
}

export function select(env: SectionEnv, path: FieldPath, label: string, options: Array<[string | number, string]>, hint?: string, parse: (v: string) => unknown = (v) => v): Bound {
  const sel = document.createElement("select");
  sel.innerHTML = options.map(([v, t]) => `<option value="${v}">${esc(t)}</option>`).join("");
  const model = env.ctx.settings.device;
  sel.onchange = () => model.set(path, parse(sel.value));
  return deviceField(env, path, label, sel, () => (sel.value = model.draft ? String(getField(model.draft, path)) : String(options[0][0])), hint);
}

export interface ChoiceOpts {
  options: Array<[string, string]>;
}

/** A group of radio buttons (one choice). */
export function choice(env: SectionEnv, path: FieldPath, label: string, o: ChoiceOpts, hint?: string): Bound {
  const name = id(path);
  const wrap = document.createElement("div");
  wrap.className = "choices";
  wrap.innerHTML = o.options.map(([v, t]) => `<label class="opt"><input type="radio" name="${name}" value="${v}"><span>${esc(t)}</span></label>`).join("");
  const model = env.ctx.settings.device;
  wrap.onchange = (e) => model.set(path, (e.target as HTMLInputElement).value);
  return deviceField(env, path, label, wrap, () => {
    const cur = model.draft ? String(getField(model.draft, path)) : "";
    wrap.querySelectorAll<HTMLInputElement>("input").forEach((i) => (i.checked = i.value === cur));
  }, hint);
}

/** Channels 1-13 as checkboxes; `mask` stores them as bits (bit 0 = channel 1), `list` as visiting order. */
export function channelBoxes(env: SectionEnv, path: FieldPath, label: string, kind: "mask" | "list", hint?: string, extraOff?: () => boolean, extra?: (wrap: HTMLElement) => void, summary?: (v: number[]) => string): Bound {
  const wrap = document.createElement("div");
  wrap.className = "chboxes";
  wrap.innerHTML = Array.from({ length: 13 }, (_, i) => `<label class="ch"><input type="checkbox" data-ch="${i + 1}"><span>${i + 1}</span></label>`).join("");
  const note = document.createElement("div");
  note.className = "frow-hint";
  const all = document.createElement("div");
  all.append(wrap, note);
  const model = env.ctx.settings.device;
  const read = (): number[] => {
    const v = model.draft ? getField(model.draft, path) : kind === "mask" ? 0 : [];
    return kind === "mask" ? Array.from({ length: 13 }, (_, i) => i + 1).filter((c) => ((v as number) >> (c - 1)) & 1) : (v as number[]).filter((c) => c !== 0);
  };
  const write = (chs: number[]) => model.set(path, kind === "mask" ? chs.reduce((m, c) => m | (1 << (c - 1)), 0) : [...chs, ...Array(13).fill(0)].slice(0, 13));
  wrap.onchange = (e) => {
    const box = e.target as HTMLInputElement;
    const c = Number(box.dataset.ch);
    const cur = read();
    // a list keeps the order the channels were ticked in; a mask has no order
    write(box.checked ? [...cur.filter((x) => x !== c), c] : cur.filter((x) => x !== c));
  };
  extra?.(all);
  const bound = deviceField(env, path, label, all, () => {
    const cur = read();
    wrap.querySelectorAll<HTMLInputElement>("input").forEach((i) => (i.checked = cur.includes(Number(i.dataset.ch))));
    note.textContent = summary ? summary(cur) : "";
  }, hint, extraOff);
  return { el: bound.el, sync: bound.sync };
}

/** Helpers for phone-side settings: saved at once, with a "saved locally" notice. */
export function phoneToggle(env: SectionEnv, label: string, get: () => boolean, set: (v: boolean) => void, hint?: string): Bound {
  const input = Object.assign(document.createElement("input"), { type: "checkbox", id: id(label) });
  const wrap = document.createElement("label");
  wrap.className = "switch";
  wrap.append(input, Object.assign(document.createElement("span"), { textContent: "on" }));
  input.onchange = () => {
    set(input.checked);
    notice("saved locally");
  };
  const r = row(label, wrap, hint);
  return { el: r.el, sync: () => (input.checked = get()) };
}

export function phoneSelect(env: SectionEnv, label: string, options: Array<[string, string]>, get: () => string, set: (v: string) => void, hint?: string): Bound {
  const sel = document.createElement("select");
  sel.innerHTML = options.map(([v, t]) => `<option value="${v}">${esc(t)}</option>`).join("");
  sel.onchange = () => {
    set(sel.value);
    notice("saved locally");
  };
  const r = row(label, sel, hint);
  void env;
  return { el: r.el, sync: () => (sel.value = get()) };
}

export function phoneText(env: SectionEnv, label: string, max: number, get: () => string, set: (v: string) => void, hint?: string): Bound {
  const input = Object.assign(document.createElement("input"), { type: "text", maxLength: max, autocomplete: "off", spellcheck: false });
  input.onchange = () => {
    set(input.value.trim());
    notice("saved locally");
  };
  const r = row(label, input, hint);
  void env;
  return { el: r.el, sync: () => document.activeElement !== input && (input.value = get()) };
}

export function phoneSlider(env: SectionEnv, label: string, o: SliderOpts, get: () => number, set: (v: number) => void, hint?: string): Bound {
  const range = Object.assign(document.createElement("input"), { type: "range", min: String(o.min), max: String(o.max), step: String(o.step) });
  const out = document.createElement("span");
  out.className = "frow-value";
  const wrap = document.createElement("div");
  wrap.className = "slider";
  wrap.append(range, out);
  range.oninput = () => (out.textContent = o.show ? o.show(Number(range.value)) : "");
  range.onchange = () => {
    set(Number(range.value));
    notice("saved locally");
  };
  const r = row(label, wrap, hint);
  void env;
  return {
    el: r.el,
    sync() {
      range.value = String(get());
      out.textContent = o.show ? o.show(get()) : "";
    },
  };
}

/** A plain labelled block for read-only facts. */
export function readOnly(label: string, value: string, why?: string, copy = false): Bound & { set(v: string): void } {
  const val = document.createElement("span");
  val.className = "ro-value";
  val.textContent = value;
  const wrap = document.createElement("div");
  wrap.className = "textrow";
  wrap.append(val);
  if (copy) {
    const b = Object.assign(document.createElement("button"), { type: "button", className: "small", textContent: "COPY" });
    b.onclick = () => void navigator.clipboard?.writeText(val.textContent ?? "").then(() => notice("copied"), () => notice("could not copy"));
    wrap.append(b);
  }
  const r = row(label, wrap, why);
  r.el.classList.add("readonly");
  if (why) r.el.title = why;
  return { el: r.el, sync() {}, set: (v) => (val.textContent = v) };
}

/** Mount a list of bound fields into `host` and keep them in sync. */
export function mountFields(host: HTMLElement, env: SectionEnv, fields: Bound[]): void {
  host.append(...fields.map((f) => f.el));
  const sync = () => fields.forEach((f) => f.sync());
  env.onRefresh(sync);
  sync();
}
