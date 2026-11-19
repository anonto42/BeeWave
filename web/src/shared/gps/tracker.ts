import type { GpsCoordinates } from "./types";

export class GpsTracker {
  private current: GpsCoordinates | null = null;
  private watchId: number | null = null;
  private subs: Array<() => void> = [];
  private errMessage: string | null = null;

  constructor(
    private readonly geo: Geolocation | null = typeof navigator !== "undefined" && "geolocation" in navigator ? navigator.geolocation : null,
    private readonly now: () => number = Date.now
  ) {}

  get isSupported(): boolean {
    return this.geo !== null;
  }

  get isActive(): boolean {
    return this.watchId !== null;
  }

  get coords(): GpsCoordinates | null {
    return this.current;
  }

  get error(): string | null {
    return this.errMessage;
  }

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  start(): void {
    if (!this.geo || this.watchId !== null) return;
    this.errMessage = null;
    try {
      this.watchId = this.geo.watchPosition(
        (pos) => {
          this.current = {
            lat: pos.coords.latitude,
            lon: pos.coords.longitude,
            alt: pos.coords.altitude ?? 0,
            accuracy: pos.coords.accuracy,
            at: pos.timestamp || this.now(),
          };
          this.errMessage = null;
          this.emit();
        },
        (err) => {
          this.errMessage = err.message;
          this.emit();
        },
        { enableHighAccuracy: true, timeout: 10000, maximumAge: 1000 }
      );
    } catch (e) {
      this.errMessage = String(e);
      this.emit();
    }
    this.emit();
  }

  stop(): void {
    if (this.geo && this.watchId !== null) {
      this.geo.clearWatch(this.watchId);
      this.watchId = null;
    }
    this.emit();
  }

  /** Test helper to set location coordinates directly */
  setMockCoords(c: GpsCoordinates | null): void {
    this.current = c;
    this.emit();
  }

  private emit(): void {
    this.subs.forEach((f) => f());
  }
}
