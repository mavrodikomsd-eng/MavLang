// notify_tray — «Я пришёл» (чистый Win32 API, без внешних крейтов)
// Компилируется: rustc src/main.rs  (или cargo)
// При клике кнопки пишет came_back.txt в корне проекта и показывает трей-уведомление.

#![windows_subsystem = "windows"]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::ptr;

use winapi::um::winuser::*;
use winapi::um::shellapi::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winnt::*;
use winapi::um::commctrl::*;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::*;

// --- constants ---
const ID_TRAY: u32 = 1001;
const WM_TRAYICON: u32 = WM_USER + 100;
const IDM_EXIT: u32 = 1002;
const IDM_SHOW: u32 = 1003;
const BTN_BACK: u32 = 2001;

static mut HINST: HINSTANCE = ptr::null_mut();
static mut HWND_MAIN: HWND = ptr::null_mut();
static mut HWND_BTN: HWND = ptr::null_mut();

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn now_str() -> String {
    let now = SystemTime::now();
    let dur = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs();
    let secs_day = secs % 86400;
    let h = secs_day / 3600;
    let m = (secs_day % 3600) / 60;
    let s = secs_day % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn write_came_back() {
    let path = r"C:\1\аап\came_back.txt";
    let content = format!("Я пришёл: {}\n", now_str());
    let _ = fs::write(path, content);
}

fn tray_add(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = ID_TRAY;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    let tip = to_wide("MavLang — я пришёл");
    for (i, c) in tip.iter().enumerate() {
                    nid.szTip[i] = *c;
    }
    nid.hIcon = unsafe { LoadIconW(HINST, MAKEINTRESOURCEW(1)) };
    unsafe { Shell_NotifyIconW(NIM_ADD, &mut nid); }
}

fn tray_remove(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = ID_TRAY;
    unsafe { Shell_NotifyIconW(NIM_DELETE, &mut nid); }
}

fn tray_balloon(hwnd: HWND, title: &str, msg: &str) {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = ID_TRAY;
    nid.uFlags = NIF_INFO;
    nid.dwInfoFlags = NIIF_INFO;
    let t = to_wide(title);
    let m = to_wide(msg);
    for (i, c) in t.iter().enumerate() {
        nid.szInfoTitle[i] = *c;
    }
    for (i, c) in m.iter().enumerate() {
        nid.szInfo[i] = *c;
    }
    unsafe { Shell_NotifyIconW(NIM_MODIFY, &mut nid); }
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            // кнопка
            HWND_BTN = CreateWindowExW(
                0,
                to_wide("BUTTON").as_ptr(),
                to_wide("Я пришёл").as_ptr(),
                WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON,
                40, 40, 200, 40,
                hwnd,
                BTN_BACK as HMENU,
                HINST,
                ptr::null_mut(),
            );
            tray_add(hwnd);
            0
        }
        WM_COMMAND => {
            let id = LOWORD(wparam as u32);
            if u32::from(id) == BTN_BACK {
                write_came_back();
                tray_balloon(hwnd, "Уведомление", "Помечено: я пришёл!");
                ShowWindow(hwnd, SW_HIDE);
            }
            0
        }
        WM_TRAYICON => {
            if lparam as u32 == WM_LBUTTONDBLCLK || lparam as u32 == WM_RBUTTONUP {
                // просто покажем окно
                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);
            }
            0
        }
        WM_SIZE => {
            if wparam == SIZE_MINIMIZED {
                ShowWindow(hwnd, SW_HIDE);
            }
            0
        }
        WM_CLOSE => {
            ShowWindow(hwnd, SW_HIDE);
            0
        }
        WM_DESTROY => {
            tray_remove(hwnd);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn main() {
    unsafe {
        HINST = GetModuleHandleW(ptr::null());

        // инитим common controls (для кнопок, хотя BUTTON работает и без)
        let mut icc: INITCOMMONCONTROLSEX = std::mem::zeroed();
        icc.dwSize = std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32;
        icc.dwICC = ICC_WIN95_CLASSES;
        InitCommonControlsEx(&mut icc);

        let class_name = to_wide("MavLangNotifyClass");
        let wnd_name = to_wide("Я пришёл");

        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(wndproc);
        wc.hInstance = HINST;
        wc.hCursor = LoadCursorW(ptr::null_mut(), IDC_ARROW);
        wc.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
        wc.lpszClassName = class_name.as_ptr();
        RegisterClassExW(&mut wc);

        HWND_MAIN = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            wnd_name.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT, CW_USEDEFAULT, 320, 160,
            ptr::null_mut(),
            ptr::null_mut(),
            HINST,
            ptr::null_mut(),
        );

        if HWND_MAIN.is_null() {
            return;
        }

        ShowWindow(HWND_MAIN, SW_SHOW);
        UpdateWindow(HWND_MAIN);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&mut msg);
            DispatchMessageW(&mut msg);
        }
    }
}
