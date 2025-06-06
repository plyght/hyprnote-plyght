use crate::{LocationConnectivityError, LocationEvent, LocationEventType, LocationStatus};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;
use tokio::time::interval;

#[cfg(target_os = "macos")]
use crate::wifi_macos::get_wifi_ssid;

#[cfg(not(target_os = "macos"))]
fn get_wifi_ssid() -> Result<Option<String>, LocationConnectivityError> {
    Err(LocationConnectivityError::PlatformNotSupported)
}

pub struct LocationConnectivityState<R: tauri::Runtime> {
    app_handle: tauri::AppHandle<R>,
    current_status: Arc<RwLock<LocationStatus>>,
    monitoring_active: Arc<RwLock<bool>>,
}

impl<R: tauri::Runtime> LocationConnectivityState<R> {
    pub fn new(app_handle: &tauri::AppHandle<R>) -> Self {
        let state = Self {
            app_handle: app_handle.clone(),
            current_status: Arc::new(RwLock::new(LocationStatus::default())),
            monitoring_active: Arc::new(RwLock::new(false)),
        };
        
        // Start background monitoring
        state.start_monitoring();
        
        state
    }
    
    pub async fn get_current_ssid(&self) -> Result<Option<String>, LocationConnectivityError> {
        get_wifi_ssid()
    }
    
    pub async fn get_location_status(&self) -> Result<LocationStatus, LocationConnectivityError> {
        let status = self.current_status.read().await;
        Ok(status.clone())
    }
    
    pub async fn update_location_status(&self) -> Result<(), LocationConnectivityError> {
        let current_ssid = self.get_current_ssid().await?;
        
        let is_enabled = crate::store::get_location_based_enabled(self.app_handle.clone()).await?;
        let trusted_ssids = crate::store::get_trusted_ssids(self.app_handle.clone()).await?;
        
        let is_in_trusted_location = if let Some(ref ssid) = current_ssid {
            trusted_ssids.contains(ssid)
        } else {
            false
        };
        
        let should_use_cloud = is_enabled && is_in_trusted_location;
        
        let new_status = LocationStatus {
            is_enabled,
            current_ssid: current_ssid.clone(),
            is_in_trusted_location,
            trusted_ssids,
            should_use_cloud,
        };
        
        let mut current_status = self.current_status.write().await;
        let status_changed = new_status.current_ssid != current_status.current_ssid 
            || new_status.is_in_trusted_location != current_status.is_in_trusted_location
            || new_status.should_use_cloud != current_status.should_use_cloud;
        
        *current_status = new_status.clone();
        drop(current_status);
        
        if status_changed {
            self.emit_location_event(LocationEventType::LocationChanged, &new_status).await;
        }
        
        Ok(())
    }
    
    pub async fn emit_location_event(&self, event_type: LocationEventType, status: &LocationStatus) {
        let event = LocationEvent {
            event_type,
            current_ssid: status.current_ssid.clone(),
            is_trusted: status.is_in_trusted_location,
            should_use_cloud: status.should_use_cloud,
        };
        
        if let Err(e) = self.app_handle.emit("location-connectivity://location-changed", &event) {
            tracing::error!("Failed to emit location event: {}", e);
        }
    }
    
    pub fn start_monitoring(&self) {
        let app_handle = self.app_handle.clone();
        let monitoring_active = self.monitoring_active.clone();
        
        tokio::spawn(async move {
            let mut is_active = monitoring_active.write().await;
            if *is_active {
                return; // Already monitoring
            }
            *is_active = true;
            drop(is_active);
            
            let mut interval = interval(Duration::from_secs(5)); // Check every 5 seconds
            
            loop {
                interval.tick().await;
                
                if let Some(state) = app_handle.try_state::<LocationConnectivityState<R>>() {
                    if let Err(e) = state.update_location_status().await {
                        tracing::warn!("Failed to update location status: {}", e);
                    }
                } else {
                    break; // App is shutting down
                }
            }
            
            let mut is_active = monitoring_active.write().await;
            *is_active = false;
        });
    }
}