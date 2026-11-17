import { DeviceState } from "./device-state";
import { DeviceSettingsModel } from "./model";
import { PhoneSettingsStore } from "./phone-settings";

/** Device-side settings (edited, then applied), what the device reported, and phone-side settings (saved at once). */
export interface SettingsKernel {
  readonly device: DeviceSettingsModel;
  readonly state: DeviceState;
  readonly phone: PhoneSettingsStore;
}

export function createSettingsKernel(storage?: ConstructorParameters<typeof PhoneSettingsStore>[0]): SettingsKernel {
  return { device: new DeviceSettingsModel(), state: new DeviceState(), phone: new PhoneSettingsStore(storage) };
}
