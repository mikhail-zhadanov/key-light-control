// src/ui.rs

use crate::background::{self, BackgroundCommand};
use crate::settings::*;
use eframe::egui;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

pub struct MyApp {
    pub settings: MyAppSettings,
    pub cmd_tx: Sender<BackgroundCommand>,
    pub log_rx: Receiver<String>,
    pub last_log: Option<String>,
    pub worker_handle: Option<JoinHandle<()>>,
    pub first_run: bool,
    pub auto_start: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        let settings: MyAppSettings = load_app_settings();
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (log_tx, log_rx) = std::sync::mpsc::channel();
        let handle = spawn_worker(
            settings.ip_address.clone(),
            settings.port,
            settings.check_interval,
            cmd_rx,
            log_tx,
        );

        let auto = is_autostart_enabled();

        Self {
            settings,
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

        while let Ok(line) = self.log_rx.try_recv() {
            self.last_log = Some(line);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut restart = false;
            ui.horizontal(|ui| {
                ui.label("IP address: ");
                if ui.text_edit_singleline(&mut self.settings.ip_address).changed() {
                    self.settings.ip_address = self.settings.ip_address.trim().to_string();
                    restart = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Port: ");
                let mut s = self.settings.port.to_string();
                if ui.text_edit_singleline(&mut s).changed() {
                    if let Ok(p) = s.parse() {
                        self.settings.port = p;
                        restart = true;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Interval (ms): ");
                let mut s = self.settings.check_interval.to_string();
                if ui.text_edit_singleline(&mut s).changed() {
                    if let Ok(i) = s.parse() {
                        self.settings.check_interval = i;
                        restart = true;
                    }
                }
            });

            ui.separator();
            if ui.checkbox(&mut self.auto_start, "Start with Windows").changed() {
                set_autostart(self.auto_start)
                    .unwrap_or_else(|e| eprintln!("Registry error: {:?}", e));
            }

            ui.separator();
            ui.label(self.last_log.as_deref().unwrap_or(""));

            if restart {
                let _ = save_app_settings(MyAppSettings {
                    ip_address: self.settings.ip_address.clone(),
                    port: self.settings.port,
                    check_interval: self.settings.check_interval,
                });

                let _ = self.cmd_tx.send(BackgroundCommand::Stop);
                if let Some(h) = self.worker_handle.take() {
                    let _ = h.join();
                }

                self.last_log = None;

                let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
                let (log_tx, log_rx) = std::sync::mpsc::channel();
                let ip = self.settings.ip_address.clone();
                let port = self.settings.port;
                let interval = self.settings.check_interval;
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
    cmd_rx: Receiver<BackgroundCommand>,
    log_tx: Sender<String>,
) -> JoinHandle<()> {
    std::thread::spawn(move || background::run(ip, port, cmd_rx, log_tx, interval))
}