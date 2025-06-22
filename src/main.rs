//! main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use consts::APPNAME;
use eframe::egui;
use std::sync::Mutex;
use tray_icon::{MouseButtonState, TrayIconBuilder, TrayIconEvent};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOWDEFAULT, SetForegroundWindow};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle, Win32WindowHandle};

mod background;
mod consts;
use crate::consts::*;
mod settings;
mod utils;
use crate::utils::icon::*;
mod ui;
use ui::MyApp;

static VISIBLE: Mutex<bool> = Mutex::new(false);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let icon_image: IconImage = load_icon_raw(std::path::Path::new(TRAY_ICON_LIT_PATH))?;
    let _tray_icon = TrayIconBuilder::new()
        .with_icon(icon_image.to_tray_icon())
        .with_tooltip(APPNAME)
        .build()?;

    // Launch eframe
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_visible(false)
            .with_icon(icon_image.to_egui_icon()),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        APPNAME,
        options,
        Box::new(|cc| {
            // Hook tray click event
            match cc.window_handle().map(|h| h.as_raw()) {
                Ok(RawWindowHandle::Win32(handle)) => setup_tray_icon_click_handler(handle),
                Ok(_) => eprintln!("Tray icon click handler only supports Win32 window handle"),
                Err(e) => eprintln!("Failed to get window handle: {}", e),
            }

            Ok(Box::new(MyApp::default()))
        }),
    )?;
    Ok(())
}

fn handle_to_hwnd(handle: Win32WindowHandle) -> HWND {
    HWND(handle.hwnd.get() as *mut std::ffi::c_void)
}

fn setup_tray_icon_click_handler(handle: Win32WindowHandle) {
    TrayIconEvent::set_event_handler(Some(move |event| {
        if let TrayIconEvent::Click {
            button_state: MouseButtonState::Down,
            ..
        } = event
        {
            let mut visible = VISIBLE.lock().unwrap();
            let hwnd = handle_to_hwnd(handle);
            unsafe {
                let _ = ShowWindow(hwnd, if *visible { SW_HIDE } else { SW_SHOWDEFAULT });
                if !*visible {
                    let _ = SetForegroundWindow(hwnd);
                }
            }
            *visible = !*visible;
        }
    }));
}
