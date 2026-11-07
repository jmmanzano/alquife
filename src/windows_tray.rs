//! Windows notification-area integration for an unshared classic console.

use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};
use std::sync::mpsc;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::Console::{GetConsoleProcessList, GetConsoleWindow};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const TRAY_MESSAGE: u32 = WM_APP + 1;
const RESTORE: usize = 1;
const EXIT: usize = 2;
static CONSOLE: AtomicIsize = AtomicIsize::new(0);
static ACTIVE: AtomicBool = AtomicBool::new(false);
static QUIT: AtomicBool = AtomicBool::new(false);
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn quit_requested() -> bool {
    QUIT.load(Ordering::Acquire)
}

pub fn hide() -> Result<(), &'static str> {
    if std::env::var_os("WT_SESSION").is_some() {
        return Err("Tray requires a dedicated classic Windows console, not Windows Terminal");
    }
    unsafe {
        let console = GetConsoleWindow();
        let mut processes = [0u32; 2];
        let mut class = [0u16; 64];
        let len = GetClassNameW(console, class.as_mut_ptr(), class.len() as i32);
        if console.is_null()
            || len <= 0
            || String::from_utf16_lossy(&class[..len as usize]) != "ConsoleWindowClass"
            || GetConsoleProcessList(processes.as_mut_ptr(), processes.len() as u32) != 1
        {
            return Err("Tray requires a dedicated, unshared classic Windows console");
        }
        if ACTIVE.swap(true, Ordering::AcqRel) {
            return Err("The tray icon is already active");
        }
        CONSOLE.store(console as isize, Ordering::Release);
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || tray_thread(tx));
        match rx.recv() {
            Ok(true) => Ok(()),
            _ => {
                ACTIVE.store(false, Ordering::Release);
                Err("Could not create the Windows tray icon")
            }
        }
    }
}

fn tray_thread(ready: mpsc::SyncSender<bool>) {
    unsafe {
        let instance = GetModuleHandleW(null());
        TASKBAR_CREATED.store(
            RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()),
            Ordering::Release,
        );
        let class_name = wide("AlquifeTrayWindow");
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: class_name.as_ptr(),
            ..Default::default()
        };
        // The class may already be registered when hiding a second time.
        RegisterClassW(&class);
        let window = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if window.is_null() {
            let _ = ready.send(false);
            return;
        }
        let mut icon = NOTIFYICONDATAW::default();
        icon.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        icon.hWnd = window;
        icon.uID = 1;
        icon.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        icon.uCallbackMessage = TRAY_MESSAGE;
        icon.hIcon = LoadIconW(null_mut(), IDI_APPLICATION);
        let tip = wide("Alquife");
        icon.szTip[..tip.len()].copy_from_slice(&tip);
        if icon.hIcon.is_null() || Shell_NotifyIconW(NIM_ADD, &icon) == 0 {
            DestroyWindow(window);
            let _ = ready.send(false);
            return;
        }
        ShowWindow(CONSOLE.load(Ordering::Acquire) as HWND, SW_HIDE);
        let _ = ready.send(true);
        let mut message = MSG::default();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        Shell_NotifyIconW(NIM_DELETE, &icon);
        // Also restore if the message loop fails unexpectedly.
        ShowWindow(CONSOLE.load(Ordering::Acquire) as HWND, SW_RESTORE);
        ACTIVE.store(false, Ordering::Release);
    }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Explorer restarting removes the icon; restore the console rather than
    // leaving the running application inaccessible.
    if message != 0 && message == TASKBAR_CREATED.load(Ordering::Acquire) {
        DestroyWindow(window);
        return 0;
    }
    match message {
        TRAY_MESSAGE if lparam as u32 == WM_LBUTTONDBLCLK => {
            DestroyWindow(window);
            0
        }
        TRAY_MESSAGE if lparam as u32 == WM_RBUTTONUP => {
            let menu = CreatePopupMenu();
            if !menu.is_null() {
                AppendMenuW(menu, MF_STRING, RESTORE, wide("Restore").as_ptr());
                AppendMenuW(menu, MF_STRING, EXIT, wide("Exit").as_ptr());
                let mut point = POINT::default();
                GetCursorPos(&mut point);
                SetForegroundWindow(window);
                TrackPopupMenu(menu, TPM_RIGHTBUTTON, point.x, point.y, 0, window, null());
                DestroyMenu(menu);
            }
            0
        }
        WM_COMMAND if (wparam & 0xffff) == RESTORE || (wparam & 0xffff) == EXIT => {
            if (wparam & 0xffff) == EXIT {
                QUIT.store(true, Ordering::Release);
            }
            DestroyWindow(window);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}
