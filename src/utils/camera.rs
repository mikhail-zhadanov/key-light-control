use std::io;
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

pub fn is_enabled() -> io::Result<bool> {
    let key: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\webcam\NonPackaged";
    let hkcu: RegKey = RegKey::predef(HKEY_CURRENT_USER);

    // Try to open the main key; if it doesn't exist, assume the camera is disabled
    let res: RegKey = match hkcu.open_subkey(key) {
        Ok(k) => k,
        Err(_) => return Ok(false),
    };

    for subkey_name in res.enum_keys().flatten() {
        let sub_path = format!(r"{}\{}", key, subkey_name);
        if let Ok(subkey) = hkcu.open_subkey(&sub_path) {
            if let Ok(time_stop) = subkey.get_value::<u64, _>("LastUsedTimeStop") {
                if time_stop == 0 {
                    println!("Camera access is enabled for {}", sub_path);
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
