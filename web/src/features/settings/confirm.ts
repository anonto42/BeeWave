/**
 * A two-step button for destructive actions: the first tap arms it (and shows what will happen), the second
 * tap runs it; it disarms itself after a few seconds. No browser dialogs. With `word`, the second step also
 * needs that word typed (factory reset).
 */
export interface ConfirmOpts {
  label: string;
  confirmLabel: string;
  /** What the action does, shown while armed. */
  warning: string;
  /** Typed confirmation word, if any. */
  word?: string;
  onConfirm: () => void;
}

const ARM_MS = 8000;

export function confirmButton(o: ConfirmOpts): { el: HTMLElement; setDisabled(off: boolean, why?: string): void } {
  const el = document.createElement("div");
  el.className = "confirm";
  el.innerHTML = `<button type="button" class="danger" data-arm>${o.label}</button>
    <div class="armed" hidden>
      <div class="frow-hint warn">${o.warning}</div>
      ${o.word ? `<input type="text" data-word placeholder='type ${o.word}' autocomplete="off" spellcheck="false" />` : ""}
      <div class="row"><button type="button" class="danger" data-go ${o.word ? "disabled" : ""}>${o.confirmLabel}</button><button type="button" data-cancel>CANCEL</button></div>
    </div>`;
  const arm = el.querySelector<HTMLButtonElement>("[data-arm]")!;
  const armed = el.querySelector<HTMLElement>(".armed")!;
  const go = el.querySelector<HTMLButtonElement>("[data-go]")!;
  const word = el.querySelector<HTMLInputElement>("[data-word]");
  let timer: ReturnType<typeof setTimeout> | undefined;
  const disarm = () => {
    clearTimeout(timer);
    armed.hidden = true;
    arm.hidden = false;
    if (word) word.value = "";
    if (o.word) go.disabled = true;
  };
  arm.onclick = () => {
    arm.hidden = true;
    armed.hidden = false;
    timer = setTimeout(disarm, ARM_MS);
  };
  el.querySelector<HTMLButtonElement>("[data-cancel]")!.onclick = disarm;
  if (word) word.oninput = () => (go.disabled = word.value.trim() !== o.word);
  go.onclick = () => {
    disarm();
    o.onConfirm();
  };
  return {
    el,
    setDisabled(off, why) {
      arm.disabled = off;
      arm.title = off ? (why ?? "") : "";
      if (off) disarm();
    },
  };
}
