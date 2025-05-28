// Helper functions to load and save settings in the registry.
use crate::consts::*;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
use winreg::RegKey;

/// Application settings.
#[derive(Debug)]
pub struct MyAppSettings {
    pub ip_address: String,
    pub port: u16,
    pub check_interval: u32,
}

impl Default for MyAppSettings {
    fn default() -> Self {
        Self {
            ip_address: "192.168.178.21".to_owned(),
            port: 9123,
            check_interval: 500,
        }
    }
}

/// Loads the application settings from the registry.
/// Returns default settings if the registry key or values are missing.
///
/// # Example
/// ```
/// let settings = load_app_settings();
/// ```
pub fn load_app_settings() -> MyAppSettings {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = format!("Software\\{}", APPNAME);
    if let Ok(key) = hkcu.open_subkey_with_flags(key_path, KEY_READ) {
        let default = MyAppSettings::default();
        let ip: String = key.get_value("IP").unwrap_or_else(|_| default.ip_address);
        let port: u16 = key
            .get_value::<u32, _>("Port")
            .unwrap_or(default.port as u32) as u16;
        let interval: u32 = key.get_value("Interval").unwrap_or(default.check_interval);
        MyAppSettings {
            ip_address: ip,
            port,
            check_interval: interval,
        }
    } else {
        MyAppSettings::default()
    }
}

/// Saves the provided application settings to the registry.
///
/// # Errors
///
/// Returns an error if the registry operation fails.
pub fn save_app_settings(settings: MyAppSettings) -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(format!("Software\\{}", APPNAME))?;
    key.set_value("IP", &settings.ip_address)?;
    key.set_value("Port", &(settings.port as u32))?;
    key.set_value("Interval", &settings.check_interval)?;
    Ok(())
}

pub fn is_autostart_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        winreg::enums::KEY_READ,
    );
    run.and_then(|key| key.get_value::<String, _>(APPNAME))
        .is_ok()
}

pub fn set_autostart(enable: bool) -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    if enable {
        let exe = std::env::current_exe()?.display().to_string();
        key.set_value(APPNAME, &exe)?;
    } else {
        let _ = key.delete_value(APPNAME);
    }
    Ok(())
}
