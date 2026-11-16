// Shared kernel: the app context features are wired with. Features depend on this, never on each other.
import type { Telemetry } from "./protocol";
import type { Transport, TransportState } from "./transport";

type Kind = Telemetry["type"];
export interface AppContext {
  readonly transport: Transport;
  /** Subscribe to one telemetry kind. */
  onTelemetry<K extends Kind>(kind: K, cb: (m: Extract<Telemetry, { type: K }>) => void): void;
  onState(cb: (s: TransportState) => void): void;
  log(msg: string): void;
  onLog(cb: (msg: string) => void): void;
}

export function createContext(transport: Transport): AppContext {
  const tele: Array<[Kind, (m: never) => void]> = [];
  const states: Array<(s: TransportState) => void> = [];
  const logs: Array<(m: string) => void> = [];
  const log = (m: string) => logs.forEach((f) => f(m));
  transport.onTelemetry((m) => tele.forEach(([k, f]) => k === m.type && (f as (m: Telemetry) => void)(m)));
  transport.onState((s, info) => { states.forEach((f) => f(s)); if (info) log(info); });
  return {
    transport,
    onTelemetry: (k, cb) => tele.push([k, cb as (m: never) => void]),
    onState: (cb) => states.push(cb),
    log,
    onLog: (cb) => logs.push(cb),
  };
}

export const $ = <T extends HTMLElement>(root: ParentNode, sel: string) => root.querySelector(sel) as T;
export const esc = (s: string) => s.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
export function panel(title: string, body: string): HTMLElement {
  const el = document.createElement("div");
  el.className = "panel";
  el.innerHTML = `<h2>${title}</h2>${body}`;
  return el;
}
