mod commands;
mod error;
mod ext;
mod store;
mod types;

#[cfg(target_os = "macos")]
mod wifi_macos;

pub use error::*;
pub use ext::*;
pub use store::*;
pub use types::*;

use tauri::Manager;

const PLUGIN_NAME: &str = "location-connectivity";

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(PLUGIN_NAME)
        .invoke_handler(tauri::generate_handler![
            commands::get_current_ssid,
            commands::get_trusted_ssids,
            commands::add_trusted_ssid,
            commands::remove_trusted_ssid,
            commands::is_location_based_enabled,
            commands::set_location_based_enabled,
            commands::is_in_trusted_location,
            commands::get_location_status,
        ])
        .setup(|app_handle, _api| {
            let state = LocationConnectivityState::new(app_handle);
            app_handle.manage(state);
            
            Ok(())
        })
        .build()
}

#[cfg(test)]
mod test {
    use super::*;


    fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
        builder
            .plugin(init())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap()
    }

    #[test]
    fn test_location_connectivity() {
        let _app = create_app(tauri::test::mock_builder());
    }
}