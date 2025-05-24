use std::net::IpAddr;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

pub enum BackgroundCommand {
    Stop,
}

pub fn run(ip: String, port: u16, cmd_rx: Receiver<BackgroundCommand>, log_tx: Sender<String>, check_interval_ms: u32,) {
    let camera_check_interval = Duration::from_millis(check_interval_ms as u64);
    let mut is_light_on = false;

    if IpAddr::from_str(&ip).is_err() {
        let _ = log_tx.send("Invalid IP address".into());
        return;
    }

    if let Err(e) = crate::utils::light::change(false, &ip, port) {
        let _ = log_tx.send(format!("Failed to change light state: {}", e));
        return;
    }
    let _ = log_tx.send("Light turned off initially".into());

    loop {
        // Check for Stop command
        if let Ok(BackgroundCommand::Stop) = cmd_rx.try_recv() {
            let _ = log_tx.send("Stopped".into());
            break;
        }
        match crate::utils::camera::is_enabled() {
            Ok(is_camera_enabled) => {
                if is_camera_enabled && !is_light_on {
                    if let Err(e) = crate::utils::light::change(true, &ip, port) {
                        let _ = log_tx.send(format!("Failed to turn on the light: {}", e));
                    } else {
                        let _ = log_tx.send("Camera access is enabled".into());
                        is_light_on = true;
                    }
                } else if !is_camera_enabled && is_light_on {
                    if let Err(e) = crate::utils::light::change(false, &ip, port) {
                        let _ = log_tx.send(format!("Failed to turn off the light: {}", e));
                    } else {
                        let _ = log_tx.send("Camera access is disabled".into());
                        is_light_on = false;
                    }
                } else {
                    let _ = log_tx.send(format!(
                        "Camera access is {}",
                        if is_camera_enabled {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    ));
                }
            }
            Err(e) => {
                let _ = log_tx.send(format!("Failed to check camera access: {}", e));
            }
        }
        thread::sleep(camera_check_interval);
    }
}
