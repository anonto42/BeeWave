/** Counts lost packets from gaps in the 8-bit sequence number (wraps at 256). */
export class SeqTracker {
  lost = 0;
  private last = -1;

  push(seq: number): void {
    if (this.last >= 0) this.lost += (seq - this.last - 1 + 256) & 0xff;
    this.last = seq;
  }

  reset(): void {
    this.last = -1;
  }
}
