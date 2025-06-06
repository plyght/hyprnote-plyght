import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type * as bindings from "./bindings.gen";

export type LocationStatus = bindings.LocationStatus;
export type LocationEvent = bindings.LocationEvent;
export type LocationEventType = bindings.LocationEventType;

export const commands = {
  getCurrentSsid: (): Promise<string | null> =>
    invoke("plugin:location-connectivity|get_current_ssid"),

  getTrustedSsids: (): Promise<string[]> =>
    invoke("plugin:location-connectivity|get_trusted_ssids"),

  addTrustedSsid: (ssid: string): Promise<void> =>
    invoke("plugin:location-connectivity|add_trusted_ssid", { ssid }),

  removeTrustedSsid: (ssid: string): Promise<void> =>
    invoke("plugin:location-connectivity|remove_trusted_ssid", { ssid }),

  isLocationBasedEnabled: (): Promise<boolean> =>
    invoke("plugin:location-connectivity|is_location_based_enabled"),

  setLocationBasedEnabled: (enabled: boolean): Promise<void> =>
    invoke("plugin:location-connectivity|set_location_based_enabled", { enabled }),

  isInTrustedLocation: (): Promise<boolean> =>
    invoke("plugin:location-connectivity|is_in_trusted_location"),

  getLocationStatus: (): Promise<LocationStatus> =>
    invoke("plugin:location-connectivity|get_location_status"),
};

export const events = {
  onLocationChanged: (callback: (event: LocationEvent) => void) =>
    listen<LocationEvent>("location-connectivity://location-changed", (event) =>
      callback(event.payload)
    ),
};

export * from "./bindings.gen";