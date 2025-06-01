use std::{collections::HashMap, sync::Arc, time::Duration};
use tauri::{AppHandle, Manager, WebviewWindow};
use tokio::{sync::RwLock, time::sleep};

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, specta::Type, Clone, Copy)]
pub struct OverlayBound {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Default)]
pub struct OverlayState {
    pub bounds: Arc<RwLock<HashMap<String, HashMap<String, OverlayBound>>>>,
}

pub struct FakeWindowBounds(pub Arc<RwLock<HashMap<String, HashMap<String, OverlayBound>>>>);

impl Default for FakeWindowBounds {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }
}

pub fn spawn_overlay_listener(app: AppHandle, window: WebviewWindow) {
    window.set_ignore_cursor_events(true).ok();

    tokio::spawn(async move {
        let state = app.state::<FakeWindowBounds>();

        loop {
            sleep(Duration::from_millis(1000 / 20)).await;

            let map = state.0.read().await;

            let Some(windows) = map.get(window.label()) else {
                window.set_ignore_cursor_events(true).ok();
                continue;
            };

            if windows.is_empty() {
                window.set_ignore_cursor_events(true).ok();
                continue;
            }

            let (Ok(window_position), Ok(mouse_position), Ok(scale_factor)) = (
                window.outer_position(),
                window.cursor_position(),
                window.scale_factor(),
            ) else {
                let _ = window.set_ignore_cursor_events(true);
                continue;
            };

            let mut ignore = true;

            for (name, bounds) in windows.iter() {
                let x_min = (window_position.x as f64) + bounds.x * scale_factor;
                let x_max = (window_position.x as f64) + (bounds.x + bounds.width) * scale_factor;
                let y_min = (window_position.y as f64) + bounds.y * scale_factor;
                let y_max = (window_position.y as f64) + (bounds.y + bounds.height) * scale_factor;

                // println!("Checking bounds for {}: mouse({}, {}) vs bounds({}-{}, {}-{})", 
                //     name, mouse_position.x, mouse_position.y, x_min, x_max, y_min, y_max);

                if mouse_position.x >= x_min
                    && mouse_position.x <= x_max
                    && mouse_position.y >= y_min
                    && mouse_position.y <= y_max
                {
                    println!("Mouse is inside bounds for {}", name);
                    ignore = false;
                    break;
                }
            }

            window.set_ignore_cursor_events(ignore).ok();

            let focused = window.is_focused().unwrap_or(false);
            if !ignore {
                if !focused {
                    window.set_focus().ok();
                }
            } else if focused {
                window.set_ignore_cursor_events(ignore).ok();
            }
        }
    });
}
