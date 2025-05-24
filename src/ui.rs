// src/ui.rs
use std::{
    ffi::OsStr,
    os::windows::prelude::OsStrExt,
    ptr::null_mut,
    sync::mpsc::{Receiver, Sender},
};
use windows::{
    core::{Error, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, HMENU, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::EM_REPLACESEL,
            Shell::{NIF_ICON, NIF_MESSAGE, NIF_TIP, NOTIFYICONDATAW, Shell_NotifyIconW, NIM_ADD, NIM_DELETE},
            WindowsAndMessaging::{
                BS_PUSHBUTTON, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW,
                ES_AUTOVSCROLL, ES_MULTILINE, ES_READONLY, GetDlgItem, GetMessageW, GetSystemMetrics,
                GetWindowRect, IDI_APPLICATION, IsWindowVisible, LoadIconW, MSG, PostQuitMessage,
                RegisterClassW, SendMessageW, SetWindowPos, ShowWindow, SW_HIDE, SW_SHOW, SWP_NOSIZE,
                TranslateMessage, WINDOW_EX_STYLE, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
                WS_VSCROLL, WM_APP, WM_COMMAND, WM_DESTROY, WM_LBUTTONUP, SM_CXSCREEN, SM_CYSCREEN,
            },
        },
    },
};

/// We'll handle tray clicks at `WM_APP+1`
const WM_TRAYICON: u32 = WM_APP + 1;

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

pub fn run_ui(
    log_rx: Receiver<String>,
    _config_tx: Sender<(String, u16)>,
) -> windows::core::Result<()> {
    unsafe {
        // 1) Register window class
        let class_name = to_wide("MyRustTrayClass");
        let hmodule = GetModuleHandleW(None)?;
        let hinstance: HINSTANCE = hmodule.into();

        let wc = WNDCLASSW {
            hInstance: hinstance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            lpfnWndProc: Some(window_proc),
            ..Default::default()
        };
        RegisterClassW(&wc);

        // 2) Create our (initially hidden) main window
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(to_wide("Rust Light Controller").as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            HWND(0),
            HMENU(0),
            hinstance,
            null_mut(),
        )?;

        // 3) Input fields + button + log box
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(to_wide("EDIT").as_ptr()),
            PCWSTR(to_wide("192.168.178.21").as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_BORDER,
            10, 10, 200, 23,
            hwnd,
            HMENU(1 as _),
            hinstance,
            null_mut(),
        )?;

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(to_wide("EDIT").as_ptr()),
            PCWSTR(to_wide("9123").as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_BORDER,
            220, 10, 80, 23,
            hwnd,
            HMENU(2 as _),
            hinstance,
            null_mut(),
        )?;

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(to_wide("BUTTON").as_ptr()),
            PCWSTR(to_wide("Apply").as_ptr()),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            310, 10, 70, 23,
            hwnd,
            HMENU(3 as _),
            hinstance,
            null_mut(),
        )?;

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(to_wide("EDIT").as_ptr()),
            PCWSTR(to_wide("").as_ptr()),
            WS_CHILD
                | WS_VISIBLE
                | ES_MULTILINE
                | ES_AUTOVSCROLL
                | ES_READONLY
                | WS_VSCROLL,
            10, 50, 370, 200,
            hwnd,
            HMENU(4 as _),
            hinstance,
            null_mut(),
        )?;

        // 4) Add tray icon
        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 100;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = LoadIconW(None, IDI_APPLICATION)?;
        let tip = to_wide("Rust Light Controller");
        nid.szTip[..tip.len()].copy_from_slice(&tip);

        if !Shell_NotifyIconW(NIM_ADD, &nid).as_bool() {
            return Err(Error::from_win32());
        }

        // 5) Redirect log lines into the edit box
        std::thread::spawn(move || {
            while let Ok(line) = log_rx.recv() {
                if let Ok(hwnd_log) = GetDlgItem(hwnd, 4) {
                    let text = to_wide(&format!("{}\r\n", line));
                    unsafe {
                        SendMessageW(
                            hwnd_log,
                            EM_REPLACESEL,
                            WPARAM(0),
                            LPARAM(text.as_ptr() as isize),
                        );
                    }
                }
            }
        });

        // 6) Hide on startup (stay in tray)
        ShowWindow(hwnd, SW_HIDE);

        // 7) Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // 8) Remove tray icon on exit
        if !Shell_NotifyIconW(NIM_DELETE, &nid).as_bool() {
            return Err(Error::from_win32());
        }

        Ok(())
    }
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, _wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_COMMAND => LRESULT(0),
            // Clicking the tray icon
            m if m == WM_TRAYICON && lparam.0 as u32 == WM_LBUTTONUP => {
                let visible = IsWindowVisible(hwnd).as_bool();
                ShowWindow(hwnd, if visible { SW_HIDE } else { SW_SHOW });

                if !visible {
                    let mut rc = std::mem::zeroed();
                    let _ = GetWindowRect(hwnd, &mut rc);
                    let sw = GetSystemMetrics(SM_CXSCREEN);
                    let sh = GetSystemMetrics(SM_CYSCREEN);
                    let w = rc.right - rc.left;
                    let h = rc.bottom - rc.top;

                    let _ = SetWindowPos(
                        hwnd,
                        HWND(0),
                        (sw - w) / 2,
                        (sh - h) / 2,
                        0,
                        0,
                        SWP_NOSIZE,
                    );
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, WPARAM(0), LPARAM(0)),
        }
    }
}
