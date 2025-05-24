//! main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};
use std::thread::JoinHandle;
use tray_icon::{MouseButtonState, TrayIconBuilder, TrayIconEvent};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOWDEFAULT};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle, Win32WindowHandle};

use winreg::enums::*;
use winreg::RegKey;

mod background;
mod utils;

static VISIBLE: Mutex<bool> = Mutex::new(false);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/TrayIconLit.png");
    let icon: tray_icon::Icon = load_icon(std::path::Path::new(path));
    let _tray_icon = TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("Key Light Control")
        .build()?;

    // Launch eframe
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_visible(false),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Key Light Control",
        options,
        Box::new(|cc| {
            // Hook tray click event
            if let RawWindowHandle::Win32(handle) = cc.window_handle().unwrap().as_raw() {
                TrayIconEvent::set_event_handler(Some(move |event| {
                    if let TrayIconEvent::Click {
                        button_state: MouseButtonState::Down,
                        ..
                    } = event
                    {
                        let mut visible = VISIBLE.lock().unwrap();
                        let hwnd = handle_to_hwnd(handle);
                        unsafe {
                            let _ =
                                ShowWindow(hwnd, if *visible { SW_HIDE } else { SW_SHOWDEFAULT });
                        }
                        *visible = !*visible;
                    }
                }));
            }
            Ok(Box::new(MyApp::default()))
        }),
    )?;
    Ok(())
}

fn handle_to_hwnd(handle: Win32WindowHandle) -> HWND {
    HWND(handle.hwnd.get() as *mut std::ffi::c_void)
}

struct MyApp {
    ip_address: String,
    port: u16,
    check_interval: u32,
    cmd_tx: Sender<background::BackgroundCommand>,
    log_rx: Receiver<String>,
    last_log: Option<String>,
    worker_handle: Option<JoinHandle<()>>,
    first_run: bool,
    auto_start: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        let ip = "192.168.178.21".to_owned();
        let port = 9123;
        let interval = 500;
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (log_tx, log_rx) = std::sync::mpsc::channel();
        let handle = spawn_worker(ip.clone(), port, interval, cmd_rx, log_tx);

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_READ,
        );
        let auto = match run.and_then(|key| key.get_value::<String, _>("KeyLightControl")) {
            Ok(_) => true,
            Err(_) => false,
        };

        Self {
            ip_address: ip,
            port,
            check_interval: interval,
            cmd_tx,
            log_rx,
            last_log: None,
            worker_handle: Some(handle),
            first_run: true,
            auto_start: auto,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_run {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.first_run = false;
        }
        // Poll latest log
        while let Ok(line) = self.log_rx.try_recv() {
            self.last_log = Some(line);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut restart = false;
            ui.horizontal(|ui| {
                ui.label("IP address: ");
                if ui.text_edit_singleline(&mut self.ip_address).changed() {
                    self.ip_address = self.ip_address.trim().to_string();
                    restart = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Port: ");
                let mut s = self.port.to_string();
                if ui.text_edit_singleline(&mut s).changed() {
                    if let Ok(p) = s.parse() {
                        self.port = p;
                        restart = true;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Interval (ms): ");
                let mut s = self.check_interval.to_string();
                if ui.text_edit_singleline(&mut s).changed() {
                    if let Ok(i) = s.parse() {
                        self.check_interval = i;
                        restart = true;
                    }
                }
            });

            ui.separator();

            // Auto-run checkbox
            if ui
                .checkbox(&mut self.auto_start, "Start with Windows")
                .changed()
            {
                set_autostart(self.auto_start)
                    .unwrap_or_else(|e| eprintln!("Registry error: {:?}", e));
            }

            ui.separator();
            ui.label(self.last_log.as_deref().unwrap_or(""));
            if restart {
                let _ = self.cmd_tx.send(background::BackgroundCommand::Stop);
                if let Some(h) = self.worker_handle.take() {
                    let _ = h.join();
                }
                self.last_log = None;
                let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
                let (log_tx, log_rx) = std::sync::mpsc::channel();
                let ip = self.ip_address.clone();
                let port = self.port;
                let interval = self.check_interval;
                let handle = spawn_worker(ip, port, interval, cmd_rx, log_tx);
                self.worker_handle = Some(handle);
                self.cmd_tx = cmd_tx;
                self.log_rx = log_rx;
            }
        });
    }
}

fn spawn_worker(
    ip: String,
    port: u16,
    interval: u32,
    cmd_rx: Receiver<background::BackgroundCommand>,
    log_tx: Sender<String>,
) -> JoinHandle<()> {
    std::thread::spawn(move || background::run(ip, port, cmd_rx, log_tx, interval))
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

fn set_autostart(enable: bool) -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    if enable {
        let exe = std::env::current_exe()?.display().to_string();
        key.set_value("KeyLightControl", &exe)?;
    } else {
        let _ = key.delete_value("KeyLightControl");
    }
    Ok(())
}
