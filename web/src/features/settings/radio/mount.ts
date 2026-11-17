import type { SectionMount } from "../env";
import { channelBoxes, choice, mountFields, select, slider } from "../fields";

const CHANNELS: Array<[number, string]> = [[0, "AUTO (hop)"], ...Array.from({ length: 13 }, (_, i): [number, string] => [i + 1, `Channel ${i + 1}`])];

/** Radio settings (device-side): default channel, hop dwell, hop order and a custom list. */
export const mount: SectionMount = (host, env) => {
  const model = env.ctx.settings.device;
  const channel = select(env, "radio.defaultChannel", "Default channel", CHANNELS,
    "A fixed channel is a permanent pin. Choosing one also pins the radio right away (the same as the Scan tab's channel buttons); Apply makes it permanent.", Number);
  // keep the Scan tab's channel buttons and this list in step: picking here pins the radio now, picking there edits this draft
  channel.el.querySelector("select")!.addEventListener("change", (e) => {
    const value = Number((e.target as HTMLSelectElement).value);
    env.ctx.transport.send({ type: "configureRadio", channel: value }).catch((err) => env.ctx.log(`configureRadio failed: ${err}`));
  });
  const custom = () => model.draft?.radio.hopOrder !== "custom";
  mountFields(host, env, [
    channel,
    slider(env, "radio.hopDwellMs", "Hop dwell", { min: 100, max: 2000, step: 50, show: (v) => `${v} ms` },
      "Time on each channel while hopping. Changing this affects coexistence with the phone link: re-run bench row 2.2 after applying. Takes effect when the next scan starts."),
    choice(env, "radio.hopOrder", "Hop order", { options: [["ascending", "Ascending"], ["ascending1_6_11", "1-6-11 first"], ["custom", "Custom"]] },
      "The order channels are visited while hopping. Takes effect when the next scan starts."),
    channelBoxes(env, "radio.customHopList", "Custom hop list", "list", "Used only for the Custom order. Channels are visited in the order you tick them; at least one is required.", custom, undefined,
      (v) => (v.length ? `Visit order: ${v.join(" → ")}` : "No channels chosen")),
  ]);
};
