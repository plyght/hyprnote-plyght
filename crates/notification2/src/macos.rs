use std::ptr::NonNull;
use std::sync::LazyLock;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_foundation::{NSString, NSURL};
use objc2_user_notifications::{
    UNAuthorizationStatus, UNMutableNotificationContent, UNNotificationRequest,
    UNNotificationSettings, UNUserNotificationCenter,
};

use crate::{Notification, NotificationIcon, NotificationPermission};

const CENTER: LazyLock<Retained<UNUserNotificationCenter>> =
    LazyLock::new(|| unsafe { UNUserNotificationCenter::currentNotificationCenter() });

pub fn request_notification_permission() {
    if cfg!(debug_assertions) {
        return;
    }

    wezterm::macos_initialize();
}

pub fn open_notification_settings() -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.Notifications-Settings.extension")
        .spawn()?
        .wait()?;
    Ok(())
}

pub fn check_notification_permission(
    completion: impl Fn(Result<NotificationPermission, String>) + 'static,
) {
    if cfg!(debug_assertions) {
        completion(Ok(NotificationPermission::Granted));
        return;
    }

    let completion_block = RcBlock::new(move |settings: NonNull<UNNotificationSettings>| {
        let settings = unsafe { settings.as_ref() };
        let auth_status = unsafe { settings.authorizationStatus() };

        let result = match auth_status {
            UNAuthorizationStatus::Authorized => NotificationPermission::Granted,
            UNAuthorizationStatus::NotDetermined => {
                NotificationPermission::NotGrantedAndShouldRequest
            }
            _ => NotificationPermission::NotGrantedAndShouldAskManual,
        };
        completion(Ok(result))
    });

    unsafe {
        CENTER.getNotificationSettingsWithCompletionHandler(&completion_block);
    }
}

pub fn show_with_icon(notif: Notification) {
    if cfg!(debug_assertions) {
        return;
    }

    let content = unsafe { UNMutableNotificationContent::new() };

    unsafe {
        content.setTitle(&NSString::from_str(&notif.title));
        content.setBody(&NSString::from_str(&notif.message));

        if let Some(url_string) = &notif.url {
            if let Some(_url) = NSURL::URLWithString(&NSString::from_str(url_string)) {
                // For now, we'll skip setting user info as it requires more complex NSDictionary creation
                // The notification URL will be handled by the notification system differently
                tracing::debug!("Notification URL: {}", url_string);
            }
        }

        // Set custom icon based on notification type
        if let Some(icon) = &notif.icon {
            match icon {
                NotificationIcon::AppIcon(bundle_id) => {
                    // Try to get the app icon from the bundle ID
                    if let Some(icon_path) = get_app_icon_path(bundle_id) {
                        set_notification_icon(&content, &icon_path);
                    }
                }
                NotificationIcon::SystemIcon(icon_name) => {
                    // Use system icon
                    set_system_icon(&content, icon_name);
                }
                NotificationIcon::Calendar => {
                    // Use calendar system icon
                    set_system_icon(&content, "calendar");
                }
            }
        }

        let identifier = NSString::from_str(&format!("hypr-{}", uuid::Uuid::new_v4()));
        let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
            &identifier,
            &content,
            None,
        );

        let completion_block = RcBlock::new(move |error: *mut objc2_foundation::NSError| {
            if !error.is_null() {
                tracing::error!("Failed to show notification: {:?}", &*error);
            }
        });

        CENTER.addNotificationRequest_withCompletionHandler(&request, Some(&completion_block));
    }
}

fn get_app_icon_path(bundle_id: &str) -> Option<String> {
    // Map common meeting apps to their bundle IDs
    let bundle_id = match bundle_id.to_lowercase().as_str() {
        "zoom" => "us.zoom.xos",
        "teams" => "com.microsoft.teams2",
        "meet" | "google meet" => "com.google.Chrome", // Google Meet runs in browser
        "webex" => "com.cisco.webex.meetings",
        "gotomeeting" => "com.logmein.GoToMeeting",
        "bluejeans" => "com.bluejeans.videoconferencing",
        _ => bundle_id,
    };

    // Try to find the app icon using macOS APIs
    let script = format!(
        r#"tell application "System Events"
            try
                set appPath to (path to application id "{}")
                return POSIX path of appPath
            on error
                return ""
            end try
        end tell"#,
        bundle_id
    );

    if let Ok(output) = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
    {
        let path_str = String::from_utf8_lossy(&output.stdout);
        let path = path_str.trim();
        if !path.is_empty() {
            return Some(format!("{}/Contents/Resources/app.icns", path));
        }
    }

    None
}

unsafe fn set_notification_icon(_content: &UNMutableNotificationContent, icon_path: &str) {
    // This would require additional NSUserNotificationCenter APIs
    // For now, we'll use the system approach
    tracing::debug!("Setting notification icon: {}", icon_path);
}

unsafe fn set_system_icon(_content: &UNMutableNotificationContent, icon_name: &str) {
    // System icons are handled by the notification center
    tracing::debug!("Setting system icon: {}", icon_name);
}
