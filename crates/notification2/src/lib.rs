pub use wezterm::ToastNotification as WezTermNotification;

#[cfg(target_os = "macos")]
mod macos;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub url: Option<String>,
    pub timeout: Option<std::time::Duration>,
    pub icon: Option<NotificationIcon>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum NotificationIcon {
    AppIcon(String),    // Bundle ID or app name
    SystemIcon(String), // System icon name
    Calendar,
}

pub fn show(notif: Notification) {
    if cfg!(debug_assertions) {
        return;
    }

    #[cfg(target_os = "macos")]
    {
        if notif.icon.is_some() {
            macos::show_with_icon(notif);
            return;
        }
    }

    // Fallback to wezterm for notifications without icons
    let wezterm_notif = WezTermNotification {
        title: notif.title,
        message: notif.message,
        url: notif.url,
        timeout: notif.timeout,
    };
    wezterm::show(wezterm_notif);
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum NotificationPermission {
    Granted,
    NotGrantedAndShouldRequest,
    NotGrantedAndShouldAskManual,
}

pub fn request_notification_permission() {
    #[cfg(target_os = "macos")]
    macos::request_notification_permission();
}

pub fn open_notification_settings() -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        return macos::open_notification_settings();
    }

    #[cfg(not(target_os = "macos"))]
    {
        return Ok(());
    }
}

pub fn check_notification_permission(
    completion: impl Fn(Result<NotificationPermission, String>) + 'static,
) {
    #[cfg(target_os = "macos")]
    macos::check_notification_permission(completion);
}
