export const connectionHtml = `
<div data-state-slot></div>
<div class="frow">
  <div class="frow-head"><span class="frow-label">Device key</span></div>
  <div class="textrow"><input data-key type="password" placeholder="device key (64 hex characters)" autocomplete="off" spellcheck="false" /><button type="button" class="small" data-save>SAVE KEY</button></div>
  <div class="frow-hint" data-key-state></div>
  <div class="row"><button type="button" class="small" data-copy>COPY KEY</button></div>
</div>
<div data-toggles></div>
<div class="frow"><div class="frow-head"><span class="frow-label">Link</span></div><div class="row"><button type="button" data-disconnect>DISCONNECT</button></div>
  <div class="frow-hint">Drops the Bluetooth link; the saved key stays.</div></div>
<div class="frow"><div class="frow-head"><span class="frow-label">Key actions</span></div><div data-forget-slot></div><div data-rotate-slot></div>
  <div class="frow-hint warn" data-rotate-info hidden>1. Build with a new key: <code>make fw-key</code> (delete firmware/.beewave-key first), then <code>make fw-flash FW_KEY=file</code>. 2. Or reset the device so the next phone claims it: <code>make fw-unclaim</code>. Either way, update every other phone.</div></div>
`;
