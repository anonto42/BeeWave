// Composition root: choose transport, create context, mount feature slices.
import "./style.css";
import { registerSW } from "virtual:pwa-register";
import { createContext } from "./shared/app";
import { BleTransport, MockTransport } from "./shared/transport";
import * as connection from "./features/connection";
import * as status from "./features/status";
import * as control from "./features/control";
import * as scanner from "./features/scanner";
import * as log from "./features/log";

registerSW({ immediate: true });

const mock = new URLSearchParams(location.search).has("mock");
const ctx = createContext(mock ? new MockTransport() : new BleTransport());

const root = document.getElementById("app")!;
root.innerHTML = `<h1>WALPAD ${mock ? "[MOCK]" : ""}</h1>`;
for (const f of [connection, status, control, scanner, log]) f.mount(root, ctx);
