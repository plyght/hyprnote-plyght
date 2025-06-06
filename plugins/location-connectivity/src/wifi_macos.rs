#[cfg(target_os = "macos")]
use crate::LocationConnectivityError;
use std::process::Command;

// WiFi SSID detection for macOS using multiple methods
pub fn get_wifi_ssid() -> Result<Option<String>, LocationConnectivityError> {
    // Method 1: Try networksetup command
    if let Ok(ssid) = get_ssid_via_networksetup() {
        return Ok(ssid);
    }
    
    // Method 2: Try airport command (if available)
    if let Ok(ssid) = get_ssid_via_airport() {
        return Ok(ssid);
    }
    
    // Method 3: Try system_profiler (slower but comprehensive)
    if let Ok(ssid) = get_ssid_via_system_profiler() {
        return Ok(ssid);
    }
    
    Ok(None)
}

// Primary method: Use networksetup to get current airport network
fn get_ssid_via_networksetup() -> Result<Option<String>, LocationConnectivityError> {
    // Try common interface names
    let interfaces = ["en0", "en1", "en2"];
    
    for interface in &interfaces {
        let output = Command::new("networksetup")
            .arg("-getairportnetwork")
            .arg(interface)
            .output()
            .map_err(|e| LocationConnectivityError::WifiDetection(
                format!("Failed to execute networksetup: {}", e)
            ))?;
        
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Parse output like "Current Wi-Fi Network: NetworkName"
            if let Some(ssid) = parse_networksetup_output(&output_str) {
                return Ok(Some(ssid));
            }
        }
    }
    
    Ok(None)
}

// Secondary method: Use airport command (if available)
fn get_ssid_via_airport() -> Result<Option<String>, LocationConnectivityError> {
    // Try the airport command which might be available
    let airport_paths = [
        "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport",
        "/usr/local/bin/airport",
    ];
    
    for airport_path in &airport_paths {
        let output = Command::new(airport_path)
            .arg("-I")
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Parse output for SSID line
                if let Some(ssid) = parse_airport_output(&output_str) {
                    return Ok(Some(ssid));
                }
            }
        }
    }
    
    Ok(None)
}

// Tertiary method: Use system_profiler (slower but comprehensive)
fn get_ssid_via_system_profiler() -> Result<Option<String>, LocationConnectivityError> {
    let output = Command::new("system_profiler")
        .arg("SPAirPortDataType")
        .arg("-json")
        .output()
        .map_err(|e| LocationConnectivityError::WifiDetection(
            format!("Failed to execute system_profiler: {}", e)
        ))?;
    
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Try to parse JSON output
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
            if let Some(ssid) = parse_system_profiler_json(&json) {
                return Ok(Some(ssid));
            }
        }
    }
    
    Ok(None)
}

// Parse networksetup output
fn parse_networksetup_output(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.starts_with("Current Wi-Fi Network:") {
            let ssid = line.replace("Current Wi-Fi Network:", "").trim().to_string();
            if !ssid.is_empty() && ssid != "You are not associated with an AirPort network." {
                return Some(ssid);
            }
        }
    }
    None
}

// Parse airport command output
fn parse_airport_output(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.trim().starts_with("SSID:") {
            let ssid = line.replace("SSID:", "").trim().to_string();
            if !ssid.is_empty() {
                return Some(ssid);
            }
        }
    }
    None
}

// Parse system_profiler JSON output
fn parse_system_profiler_json(json: &serde_json::Value) -> Option<String> {
    // Navigate through the JSON structure to find current network SSID
    if let Some(airport_data) = json.get("SPAirPortDataType") {
        if let Some(interfaces) = airport_data.as_array() {
            for interface in interfaces {
                if let Some(networks) = interface.get("spairport_airport_other") {
                    if let Some(networks_array) = networks.as_array() {
                        for network in networks_array {
                            if let Some(current) = network.get("_name") {
                                if let Some(ssid_str) = current.as_str() {
                                    // Check if this is the current network
                                    if network.get("spairport_network_cc").is_some() {
                                        return Some(ssid_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}