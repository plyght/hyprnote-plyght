use crate::{LocationConnectivityState, LocationEventType, LocationStatus};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn get_current_ssid<R: tauri::Runtime>(
    state: State<'_, LocationConnectivityState<R>>,
) -> Result<Option<String>, String> {
    state.get_current_ssid().await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_trusted_ssids<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<String>, String> {
    crate::store::get_trusted_ssids(app).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn add_trusted_ssid<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: State<'_, LocationConnectivityState<R>>,
    ssid: String,
) -> Result<(), String> {
    if ssid.trim().is_empty() {
        return Err("SSID cannot be empty".to_string());
    }
    
    crate::store::add_trusted_ssid(app, ssid).await.map_err(|e| e.to_string())?;
    
    // Update location status after adding trusted SSID
    state.update_location_status().await.map_err(|e| e.to_string())?;
    
    // Emit settings changed event
    let status = state.get_location_status().await.map_err(|e| e.to_string())?;
    state.emit_location_event(LocationEventType::SettingsChanged, &status).await;
    
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn remove_trusted_ssid<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: State<'_, LocationConnectivityState<R>>,
    ssid: String,
) -> Result<(), String> {
    crate::store::remove_trusted_ssid(app, ssid).await.map_err(|e| e.to_string())?;
    
    // Update location status after removing trusted SSID
    state.update_location_status().await.map_err(|e| e.to_string())?;
    
    // Emit settings changed event
    let status = state.get_location_status().await.map_err(|e| e.to_string())?;
    state.emit_location_event(LocationEventType::SettingsChanged, &status).await;
    
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn is_location_based_enabled<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<bool, String> {
    crate::store::get_location_based_enabled(app).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn set_location_based_enabled<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: State<'_, LocationConnectivityState<R>>,
    enabled: bool,
) -> Result<(), String> {
    crate::store::set_location_based_enabled(app, enabled).await.map_err(|e| e.to_string())?;
    
    // Update location status after changing setting
    state.update_location_status().await.map_err(|e| e.to_string())?;
    
    // Emit settings changed event
    let status = state.get_location_status().await.map_err(|e| e.to_string())?;
    state.emit_location_event(LocationEventType::SettingsChanged, &status).await;
    
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn is_in_trusted_location<R: tauri::Runtime>(
    state: State<'_, LocationConnectivityState<R>>,
) -> Result<bool, String> {
    let status = state.get_location_status().await.map_err(|e| e.to_string())?;
    Ok(status.is_in_trusted_location)
}

#[tauri::command]
#[specta::specta]
pub async fn get_location_status<R: tauri::Runtime>(
    state: State<'_, LocationConnectivityState<R>>,
) -> Result<LocationStatus, String> {
    state.get_location_status().await.map_err(|e| e.to_string())
}