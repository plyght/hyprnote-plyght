// @ts-nocheck

export type LocationEventType = "location_changed" | "trust_status_changed" | "settings_changed"

export type LocationEvent = {
  event_type: LocationEventType
  current_ssid: string | null
  is_trusted: boolean
  should_use_cloud: boolean
}

export type LocationStatus = {
  is_enabled: boolean
  current_ssid: string | null
  is_in_trusted_location: boolean
  trusted_ssids: string[]
  should_use_cloud: boolean
}