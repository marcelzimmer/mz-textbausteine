#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Local;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock};

#[cfg(windows)]
use windows::core::{w, PCWSTR};
#[cfg(windows)]
use windows::Win32::{
    Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::{
        CreateFontW, DeleteObject, GetSysColorBrush, HDC, HBRUSH, HFONT, HGDIOBJ,
        SetBkMode, SYS_COLOR_INDEX, TRANSPARENT,
    },
    System::LibraryLoader::GetModuleHandleW,
    System::Threading::CreateMutexW,
    UI::Input::KeyboardAndMouse::{
        GetKeyState, GetKeyboardLayout, SendInput, ToUnicodeEx, INPUT, INPUT_0,
        INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
        VIRTUAL_KEY, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT,
        VK_MENU, VK_RCONTROL, VK_RETURN, VK_RMENU, VK_RSHIFT, VK_SHIFT,
    },
    UI::Shell::{
        ShellExecuteW, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD,
        NIM_DELETE, NIM_SETVERSION, NOTIFYICONDATAW, NOTIFYICONDATAW_0,
        NOTIFYICON_VERSION_4,
    },
    UI::WindowsAndMessaging::{
        AppendMenuW, CallNextHookEx, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
        DestroyMenu, DestroyWindow, DispatchMessageW, GetForegroundWindow,
        GetMessageW, GetWindowThreadProcessId, IsDialogMessageW, LoadIconW, MessageBoxW,
        PostQuitMessage, RegisterClassExW, SendMessageW, SetForegroundWindow,
        SetWindowsHookExW, ShowWindow, TrackPopupMenu, TranslateMessage,
        UnhookWindowsHookEx, UnregisterClassW, CW_USEDEFAULT, HC_ACTION, HMENU,
        HWND_MESSAGE, IDYES, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MB_ICONQUESTION, MB_YESNO,
        MF_STRING, MSG, SW_SHOW, SW_SHOWNORMAL, TPM_BOTTOMALIGN, TPM_RIGHTALIGN,
        TPM_RIGHTBUTTON, WH_KEYBOARD_LL, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP,
        WM_COMMAND, WM_CONTEXTMENU, WM_CREATE, WM_CTLCOLORSTATIC, WM_DESTROY,
        WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_SETFONT, WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSEXW, WS_CAPTION, WS_CHILD,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP,
        WS_VISIBLE,
    },
};

#[derive(Deserialize, Default)]
struct Config {
    #[serde(default)]
    textbaustein: Vec<Textbaustein>,
}

#[derive(Deserialize, Clone)]
struct Textbaustein {
    #[serde(default)]
    ausloeser: Option<String>,
    #[serde(default)]
    tastenfolge: Option<String>,
    ersetzung: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
static BUFFER: OnceLock<Mutex<String>> = OnceLock::new();
static COMPOSE_BUFFER: OnceLock<Mutex<String>> = OnceLock::new();
static COMPOSE_MODE: AtomicBool = AtomicBool::new(false);

static MOD_LSHIFT: AtomicBool = AtomicBool::new(false);
static MOD_RSHIFT: AtomicBool = AtomicBool::new(false);
static MOD_LCTRL: AtomicBool = AtomicBool::new(false);
static MOD_RCTRL: AtomicBool = AtomicBool::new(false);
static MOD_LALT: AtomicBool = AtomicBool::new(false);
static MOD_RALT: AtomicBool = AtomicBool::new(false);
static CAPS_TOGGLED: AtomicBool = AtomicBool::new(false);

static SUPPRESSED_KEYS: OnceLock<Mutex<HashSet<u32>>> = OnceLock::new();

const BUFFER_MAX: usize = 128;

// Magic-Wert in dwExtraInfo, mit dem alle von uns per SendInput erzeugten
// Tastatur-Events markiert werden. Der Hook erkennt sie daran zweifelsfrei
// und schleust sie ohne Verarbeitung weiter, unabhängig von LLKHF_INJECTED.
#[cfg(windows)]
const MZ_INPUT_MAGIC: usize = 0x4D5A_5442;

#[cfg(windows)]
const WM_TRAY: u32 = WM_APP + 1;

// Tray-Notification-Codes [V4-Protokoll]. In windows-rs 0.58 nicht exportiert,
// daher hier definiert. Werte sind seit Jahrzehnten stabil [WM_USER + 0 / + 1].
#[cfg(windows)]
const NIN_SELECT: u32 = 0x0400;

#[cfg(windows)]
const NIN_KEYSELECT: u32 = 0x0401;

#[cfg(windows)]
const TRAY_ID: u32 = 1;

#[cfg(windows)]
static UEBER_HWND: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

#[cfg(windows)]
static FONT_NORMAL: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

#[cfg(windows)]
static FONT_TITEL: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

#[cfg(windows)]
const ID_OK: u16 = 1;

#[cfg(windows)]
const ID_SCHLIESSEN: u16 = 2;

#[cfg(windows)]
const ID_BEENDEN: u16 = 100;

#[cfg(windows)]
const ID_LINK_WEB: u16 = 200;

#[cfg(windows)]
const ID_LINK_X: u16 = 201;

#[cfg(windows)]
const ID_LINK_GITHUB: u16 = 202;

#[cfg(windows)]
const ID_TRAY_BEENDEN: u16 = 1000;

#[cfg(windows)]
const ID_TRAY_UEBER: u16 = 1001;

#[cfg(windows)]
const SS_CENTER: WINDOW_STYLE = WINDOW_STYLE(0x00000001);

#[cfg(windows)]
const BS_DEFPUSHBUTTON: WINDOW_STYLE = WINDOW_STYLE(0x00000001);

#[cfg(windows)]
const BS_PUSHBUTTON: WINDOW_STYLE = WINDOW_STYLE(0x00000000);

fn load_config() -> Config {
    let Ok(exe) = std::env::current_exe() else {
        return Config::default();
    };
    let Some(dir) = exe.parent() else {
        return Config::default();
    };
    let path = dir.join("mz-textbausteine.toml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Config::default();
    };
    toml::from_str(&content).unwrap_or_default()
}

fn expand_placeholders(text: &str) -> String {
    let now = Local::now();
    text.replace("{{datum}}", &now.format("%d.%m.%Y").to_string())
        .replace("{{zeit_sek}}", &now.format("%H:%M:%S").to_string())
        .replace("{{zeit}}", &now.format("%H:%M").to_string())
}

#[cfg(windows)]
unsafe fn fonts_erstellen() {
    let font_normal = unsafe {
        CreateFontW(
            -14, 0, 0, 0, 400, 0, 0, 0,
            1, 0, 0, 0, 0,
            w!("Segoe UI"),
        )
    };
    let font_titel = unsafe {
        CreateFontW(
            -22, 0, 0, 0, 700, 0, 0, 0,
            1, 0, 0, 0, 0,
            w!("Segoe UI"),
        )
    };
    FONT_NORMAL.store(font_normal.0, Ordering::SeqCst);
    FONT_TITEL.store(font_titel.0, Ordering::SeqCst);
}

#[cfg(windows)]
unsafe fn fonts_freigeben() {
    let n = FONT_NORMAL.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !n.is_null() {
        unsafe { let _ = DeleteObject(HGDIOBJ(n)); };
    }
    let t = FONT_TITEL.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !t.is_null() {
        unsafe { let _ = DeleteObject(HGDIOBJ(t)); };
    }
}

#[cfg(windows)]
unsafe fn statischen_text_erstellen(
    eltern: HWND,
    hinstance: HINSTANCE,
    text: PCWSTR,
    x: i32,
    y: i32,
    breite: i32,
    hoehe: i32,
    font: HFONT,
) {
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("STATIC"),
            text,
            WS_CHILD | WS_VISIBLE | SS_CENTER,
            x,
            y,
            breite,
            hoehe,
            eltern,
            HMENU(std::ptr::null_mut()),
            hinstance,
            None,
        )
        .expect("CreateWindowExW STATIC")
    };
    unsafe {
        let _ = SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    };
}

#[cfg(windows)]
unsafe fn button_erstellen(
    eltern: HWND,
    hinstance: HINSTANCE,
    id: u16,
    text: PCWSTR,
    x: i32,
    y: i32,
    breite: i32,
    hoehe: i32,
    font: HFONT,
    default: bool,
) {
    let stil = if default {
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON
    } else {
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON
    };
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            text,
            stil,
            x,
            y,
            breite,
            hoehe,
            eltern,
            HMENU(id as usize as *mut std::ffi::c_void),
            hinstance,
            None,
        )
        .expect("CreateWindowExW BUTTON")
    };
    unsafe {
        let _ = SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    };
}

#[cfg(windows)]
unsafe fn ueberfenster_controls_erstellen(eltern: HWND) {
    let hinstance: HINSTANCE = unsafe {
        GetModuleHandleW(None).expect("GetModuleHandleW").into()
    };
    let font_normal = HFONT(FONT_NORMAL.load(Ordering::SeqCst));
    let font_titel = HFONT(FONT_TITEL.load(Ordering::SeqCst));

    let version_text: Vec<u16> = format!("Version {}", env!("CARGO_PKG_VERSION"))
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        statischen_text_erstellen(eltern, hinstance, w!("MZ-Textbausteine"), 0, 20, 400, 32, font_titel);
        statischen_text_erstellen(eltern, hinstance, PCWSTR(version_text.as_ptr()), 0, 60, 400, 22, font_normal);
        statischen_text_erstellen(eltern, hinstance, w!("Marcel Zimmer"), 0, 88, 400, 22, font_normal);

        button_erstellen(eltern, hinstance, ID_LINK_WEB, w!("www.marcelzimmer.de"), 80, 124, 256, 22, font_normal, false);
        button_erstellen(eltern, hinstance, ID_LINK_X, w!("x.com/marcelzimmer"), 80, 150, 256, 22, font_normal, false);
        button_erstellen(eltern, hinstance, ID_LINK_GITHUB, w!("github.com/marcelzimmer"), 80, 176, 256, 22, font_normal, false);

        button_erstellen(eltern, hinstance, ID_OK, w!("OK"), 20, 240, 100, 30, font_normal, true);
        button_erstellen(eltern, hinstance, ID_SCHLIESSEN, w!("Schliessen"), 150, 240, 100, 30, font_normal, false);
        button_erstellen(eltern, hinstance, ID_BEENDEN, w!("Beenden"), 280, 240, 100, 30, font_normal, false);
    }
}

#[cfg(windows)]
unsafe extern "system" fn ueberfenster_fensterprozedur(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            unsafe { ueberfenster_controls_erstellen(hwnd) };
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as u16;
            match id {
                ID_OK | ID_SCHLIESSEN => {
                    unsafe { let _ = DestroyWindow(hwnd); };
                    LRESULT(0)
                }
                ID_BEENDEN => {
                    unsafe { beenden_mit_abfrage() };
                    LRESULT(0)
                }
                ID_LINK_WEB => {
                    unsafe {
                        let _ = ShellExecuteW(HWND(std::ptr::null_mut()), w!("open"), w!("https://www.marcelzimmer.de"), PCWSTR(std::ptr::null()), PCWSTR(std::ptr::null()), SW_SHOWNORMAL);
                    };
                    LRESULT(0)
                }
                ID_LINK_X => {
                    unsafe {
                        let _ = ShellExecuteW(HWND(std::ptr::null_mut()), w!("open"), w!("https://x.com/marcelzimmer"), PCWSTR(std::ptr::null()), PCWSTR(std::ptr::null()), SW_SHOWNORMAL);
                    };
                    LRESULT(0)
                }
                ID_LINK_GITHUB => {
                    unsafe {
                        let _ = ShellExecuteW(HWND(std::ptr::null_mut()), w!("open"), w!("https://github.com/marcelzimmer"), PCWSTR(std::ptr::null()), PCWSTR(std::ptr::null()), SW_SHOWNORMAL);
                    };
                    LRESULT(0)
                }
                _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
            }
        }
        WM_CTLCOLORSTATIC => {
            let hdc = HDC(wparam.0 as *mut _);
            unsafe { SetBkMode(hdc, TRANSPARENT) };
            LRESULT(unsafe { GetSysColorBrush(SYS_COLOR_INDEX(5)) }.0 as isize)
        }
        WM_DESTROY => {
            UEBER_HWND.store(std::ptr::null_mut(), Ordering::SeqCst);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(windows)]
unsafe fn beenden_mit_abfrage() {
    let antwort = unsafe {
        MessageBoxW(
            HWND(std::ptr::null_mut()),
            w!("Nach dem Beenden stehen die Textbausteine nicht mehr zur Verfügung.\n\nMöchten Sie MZ-Textbausteine wirklich schliessen?"),
            w!("MZ-Textbausteine beenden"),
            MB_YESNO | MB_ICONQUESTION,
        )
    };
    if antwort == IDYES {
        let ueber = UEBER_HWND.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !ueber.is_null() {
            unsafe { let _ = DestroyWindow(HWND(ueber)); };
        }
        unsafe { PostQuitMessage(0) };
    }
}

#[cfg(windows)]
unsafe fn tray_kontextmenue_anzeigen(hwnd: HWND, x: i32, y: i32) {
    let menue = unsafe { CreatePopupMenu().expect("CreatePopupMenu") };
    unsafe {
        let _ = AppendMenuW(menue, MF_STRING, ID_TRAY_UEBER as usize, w!("Über MZ-Textbausteine"));
        let _ = AppendMenuW(menue, MF_STRING, ID_TRAY_BEENDEN as usize, w!("Beenden"));
        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(
            menue,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_RIGHTALIGN,
            x,
            y,
            0,
            hwnd,
            None,
        );
        let _ = DestroyMenu(menue);
    };
}

#[cfg(windows)]
unsafe fn ueberfenster_oeffnen_oder_fokussieren() {
    let aktueller = UEBER_HWND.load(Ordering::SeqCst);
    if !aktueller.is_null() {
        unsafe { let _ = SetForegroundWindow(HWND(aktueller)); };
        return;
    }

    let hinstance: HINSTANCE = unsafe {
        GetModuleHandleW(None).expect("GetModuleHandleW").into()
    };
    let hwnd = match unsafe {
        CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            w!("MzTbUeber"),
            w!("MZ-Textbausteine"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            416,
            320,
            HWND(std::ptr::null_mut()),
            HMENU(std::ptr::null_mut()),
            hinstance,
            None,
        )
    } {
        Ok(h) => h,
        Err(_) => return,
    };
    UEBER_HWND.store(hwnd.0, Ordering::SeqCst);
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    };
}

#[cfg(windows)]
unsafe extern "system" fn nachrichten_fensterprozedur(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_TRAY => {
            // V4-Protokoll: LOWORD[lparam] = Notification-Message,
            // WPARAM = Mauskoordinaten [LOWORD=x, HIWORD=y, signed].
            let event = (lparam.0 as u32) & 0xFFFF;
            let icon_x = (wparam.0 & 0xFFFF) as u16 as i16 as i32;
            let icon_y = ((wparam.0 >> 16) & 0xFFFF) as u16 as i16 as i32;
            let menue_zeigen = event == NIN_SELECT
                || event == NIN_KEYSELECT
                || event == WM_LBUTTONUP
                || event == WM_LBUTTONDBLCLK
                || event == WM_CONTEXTMENU;
            if menue_zeigen {
                unsafe { tray_kontextmenue_anzeigen(hwnd, icon_x, icon_y) };
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as u16;
            if id == ID_TRAY_UEBER {
                unsafe { ueberfenster_oeffnen_oder_fokussieren() };
            } else if id == ID_TRAY_BEENDEN {
                unsafe { beenden_mit_abfrage() };
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(windows)]
unsafe fn tray_icon_erstellen(hwnd: HWND, hinstance: HINSTANCE) -> NOTIFYICONDATAW {
    let hicon = unsafe {
        LoadIconW(hinstance, PCWSTR(1 as *const u16)).expect("LoadIconW")
    };

    let mut sz_tip = [0u16; 128];
    let utf16: Vec<u16> = "MZ-Textbausteine".encode_utf16().collect();
    sz_tip[..utf16.len()].copy_from_slice(&utf16);

    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAY,
        hIcon: hicon,
        szTip: sz_tip,
        Anonymous: NOTIFYICONDATAW_0 {
            uVersion: NOTIFYICON_VERSION_4,
        },
        ..Default::default()
    };

    unsafe {
        Shell_NotifyIconW(NIM_ADD, &nid);
        // Auf das moderne V4-Protokoll umstellen [Vista+], damit die Notification-
        // Messages [NIN_SELECT, NIN_KEYSELECT, WM_CONTEXTMENU] eindeutig geliefert
        // werden und Klicks auf Win10/11 verlässlich ankommen.
        Shell_NotifyIconW(NIM_SETVERSION, &nid);
    };
    nid
}

#[cfg(windows)]
unsafe fn tray_icon_entfernen(nid: &NOTIFYICONDATAW) {
    unsafe { Shell_NotifyIconW(NIM_DELETE, nid) };
}

#[cfg(windows)]
unsafe fn nachrichtenfenster_erstellen(hinstance: HINSTANCE) -> HWND {
    let klassenname = w!("MzTbNachricht");

    let klasse = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(nachrichten_fensterprozedur),
        hInstance: hinstance,
        lpszClassName: klassenname,
        ..Default::default()
    };

    unsafe { RegisterClassExW(&klasse) };

    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            klassenname,
            w!(""),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            HMENU(std::ptr::null_mut()),
            hinstance,
            None,
        )
        .expect("CreateWindowExW")
    }
}

#[cfg(windows)]
unsafe fn ueberfenster_klasse_registrieren(hinstance: HINSTANCE) {
    let klasse = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(ueberfenster_fensterprozedur),
        hInstance: hinstance,
        lpszClassName: w!("MzTbUeber"),
        hIcon: unsafe {
            LoadIconW(hinstance, PCWSTR(1 as *const u16)).unwrap_or_default()
        },
        // COLOR_WINDOW + 1 = 6: Windows-Konvention für Systemfarbe als Hintergrundpinsel
        hbrBackground: HBRUSH(6usize as *mut std::ffi::c_void),
        ..Default::default()
    };
    unsafe { RegisterClassExW(&klasse) };
}

#[cfg(windows)]
fn main() {
    let mutex_result = unsafe { CreateMutexW(None, false, w!("Local\\MzTextbausteine")) };
    let already_running = mutex_result.is_ok() && unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    let _instance_mutex = mutex_result.ok();
    if already_running {
        return;
    }

    let _ = CONFIG.set(load_config());
    let _ = BUFFER.set(Mutex::new(String::new()));
    let _ = COMPOSE_BUFFER.set(Mutex::new(String::new()));
    let _ = SUPPRESSED_KEYS.set(Mutex::new(HashSet::new()));

    unsafe {
        // CapsLock-Toggle-Status beim Start einlesen, damit ToUnicodeEx das
        // korrekte Verhalten zeigt, falls CapsLock vor App-Start aktiv war.
        let caps = GetKeyState(VK_CAPITAL.0 as i32);
        if (caps as u16 & 0x0001) != 0 {
            CAPS_TOGGLED.store(true, Ordering::SeqCst);
        }

        let hmodule = GetModuleHandleW(None).expect("GetModuleHandleW");
        let hinstance: HINSTANCE = hmodule.into();

        fonts_erstellen();

        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_proc),
            hinstance,
            0,
        )
        .expect("SetWindowsHookExW");

        let nachrichten_hwnd = nachrichtenfenster_erstellen(hinstance);
        ueberfenster_klasse_registrieren(hinstance);
        let tray = tray_icon_erstellen(nachrichten_hwnd, hinstance);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
            let ueber = HWND(UEBER_HWND.load(Ordering::SeqCst));
            if !ueber.0.is_null() && IsDialogMessageW(ueber, &mut msg).as_bool() {
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        tray_icon_entfernen(&tray);
        let _ = DestroyWindow(nachrichten_hwnd);
        let _ = UnregisterClassW(w!("MzTbUeber"), hinstance);
        let _ = UnregisterClassW(w!("MzTbNachricht"), hinstance);
        let _ = UnhookWindowsHookEx(hook);
        fonts_freigeben();
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("mz-textbausteine läuft nur unter Windows.");
}

#[cfg(windows)]
unsafe extern "system" fn keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code == HC_ACTION as i32 {
        let info = unsafe { &*(l_param.0 as *const KBDLLHOOKSTRUCT) };
        // Eigene SendInput-Events tragen MZ_INPUT_MAGIC in dwExtraInfo. Diese
        // Events vor jeder weiteren Verarbeitung weiterreichen, damit unsere
        // eigene Ausgabe nicht erneut durch den Trigger-Mechanismus läuft.
        if info.dwExtraInfo == MZ_INPUT_MAGIC {
            return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
        }
        let injected = (info.flags.0 & LLKHF_INJECTED.0) != 0;
        let msg = w_param.0 as u32;
        if !injected {
            let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

            // Modifier-Status pflegen, bevor die Taste verarbeitet wird.
            update_modifier(info.vkCode, is_down, is_up);

            if is_down {
                if handle_key_down(info.vkCode, info.scanCode) {
                    if let Some(set) = SUPPRESSED_KEYS.get() {
                        set.lock().unwrap().insert(info.vkCode);
                    }
                    return LRESULT(1);
                }
            } else if is_up {
                if let Some(set) = SUPPRESSED_KEYS.get() {
                    if set.lock().unwrap().remove(&info.vkCode) {
                        return LRESULT(1);
                    }
                }
            }
        }
    }
    unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
}

#[cfg(windows)]
fn update_modifier(vk: u32, is_down: bool, is_up: bool) {
    if !is_down && !is_up {
        return;
    }
    let value = is_down;
    let target: Option<&AtomicBool> = if vk == VK_LSHIFT.0 as u32 {
        Some(&MOD_LSHIFT)
    } else if vk == VK_RSHIFT.0 as u32 {
        Some(&MOD_RSHIFT)
    } else if vk == VK_LCONTROL.0 as u32 {
        Some(&MOD_LCTRL)
    } else if vk == VK_RCONTROL.0 as u32 {
        Some(&MOD_RCTRL)
    } else if vk == VK_LMENU.0 as u32 {
        Some(&MOD_LALT)
    } else if vk == VK_RMENU.0 as u32 {
        Some(&MOD_RALT)
    } else {
        None
    };
    if let Some(flag) = target {
        flag.store(value, Ordering::SeqCst);
    }
}

// Bildet einen VK-Code auf einen Compose-Sequenz-Token ab, z. B. Space -> "space", A-Z -> "a"-"z"
#[cfg(windows)]
fn vk_to_compose_name(vk: u32) -> Option<String> {
    match vk {
        0x20 => Some("space".to_string()),
        0x41..=0x5A => Some(((b'a' + (vk as u8 - 0x41)) as char).to_string()),
        _ => None,
    }
}

// Verarbeitet einen KeyDown und liefert true, wenn die Taste verschluckt
// werden soll [verhindert Durchreichung an die Anwendung].
#[cfg(windows)]
fn handle_key_down(vk: u32, scan: u32) -> bool {
    // Compose-Modus: CapsLock wurde gedrückt, folgende Tasten werden gesammelt
    if COMPOSE_MODE.load(Ordering::SeqCst) {
        if let Some(name) = vk_to_compose_name(vk) {
            let Some(cfg) = CONFIG.get() else { return true; };
            let Some(cb_lock) = COMPOSE_BUFFER.get() else { return true; };
            let mut cb = cb_lock.lock().unwrap();
            if !cb.is_empty() {
                cb.push(' ');
            }
            cb.push_str(&name);
            let sequence = cb.clone();
            drop(cb);

            for k in &cfg.textbaustein {
                if k.tastenfolge.as_deref() == Some(sequence.as_str()) {
                    COMPOSE_MODE.store(false, Ordering::SeqCst);
                    COMPOSE_BUFFER.get().unwrap().lock().unwrap().clear();
                    let expansion = expand_placeholders(&k.ersetzung);
                    send_replacement(0, &expansion);
                    return true;
                }
            }

            // Compose-Modus verlassen, wenn keine Sequenz mehr passen kann
            let still_reachable = cfg.textbaustein.iter().any(|k| {
                k.tastenfolge.as_ref().map(|c| c.starts_with(&sequence)).unwrap_or(false)
            });
            if !still_reachable {
                COMPOSE_MODE.store(false, Ordering::SeqCst);
                COMPOSE_BUFFER.get().unwrap().lock().unwrap().clear();
            }
            // Compose-Tokens werden immer verschluckt, damit sie nicht in der
            // Anwendung erscheinen.
            return true;
        } else {
            // Nicht-Compose-Taste: Modus beenden, Taste durchreichen und
            // unten als reguläre Eingabe weiterverarbeiten.
            COMPOSE_MODE.store(false, Ordering::SeqCst);
            COMPOSE_BUFFER.get().unwrap().lock().unwrap().clear();
        }
    }

    // CapsLock startet den Compose-Modus und wird verschluckt, damit der
    // CapsLock-Toggle des Systems nicht ausgelöst wird.
    if vk == VK_CAPITAL.0 as u32 {
        COMPOSE_MODE.store(true, Ordering::SeqCst);
        if let Some(cb) = COMPOSE_BUFFER.get() {
            cb.lock().unwrap().clear();
        }
        return true;
    }

    let Some(buf_lock) = BUFFER.get() else {
        return false;
    };
    let mut buf = buf_lock.lock().unwrap();

    if vk == VK_BACK.0 as u32 {
        buf.pop();
        return false;
    }

    let Some(ch) = vk_to_char(vk, scan) else {
        return false;
    };

    if ch.is_control() {
        buf.clear();
        return false;
    }

    buf.push(ch);
    if buf.chars().count() > BUFFER_MAX {
        let skip = buf.chars().count() - BUFFER_MAX;
        *buf = buf.chars().skip(skip).collect();
    }

    let Some(cfg) = CONFIG.get() else {
        return false;
    };

    for k in &cfg.textbaustein {
        if let Some(ausloeser) = &k.ausloeser {
            if buf.ends_with(ausloeser.as_str()) {
                let ausloeser_zeichen = ausloeser.chars().count();
                let expansion = expand_placeholders(&k.ersetzung);
                buf.clear();
                drop(buf);
                // Der auslösende Tastendruck ist zum Zeitpunkt dieses Hooks
                // noch nicht im Zielfenster, die SendInput-Backspaces werden
                // jedoch vorher wirksam. Daher nur die bereits getippten
                // [chars - 1] Trigger-Zeichen löschen und den auslösenden
                // Tastendruck unterdrücken, damit er nicht hinten anhängt.
                send_replacement(ausloeser_zeichen.saturating_sub(1), &expansion);
                return true;
            }
        }
    }

    false
}

#[cfg(windows)]
fn vk_to_char(vk: u32, scan: u32) -> Option<char> {
    unsafe {
        // Tastatur-Status synthetisch aus den selbst getrackten Modifiern
        // aufbauen. GetKeyboardState liefert im Low-Level-Hook nicht den
        // tatsächlichen Modifier-Zustand, daher fälschen wir den State hier.
        let mut state = [0u8; 256];
        let l_shift = MOD_LSHIFT.load(Ordering::SeqCst);
        let r_shift = MOD_RSHIFT.load(Ordering::SeqCst);
        let l_ctrl = MOD_LCTRL.load(Ordering::SeqCst);
        let r_ctrl = MOD_RCTRL.load(Ordering::SeqCst);
        let l_alt = MOD_LALT.load(Ordering::SeqCst);
        let r_alt = MOD_RALT.load(Ordering::SeqCst);

        if l_shift {
            state[VK_LSHIFT.0 as usize] = 0x80;
        }
        if r_shift {
            state[VK_RSHIFT.0 as usize] = 0x80;
        }
        if l_shift || r_shift {
            state[VK_SHIFT.0 as usize] = 0x80;
        }
        if l_ctrl {
            state[VK_LCONTROL.0 as usize] = 0x80;
        }
        if r_ctrl {
            state[VK_RCONTROL.0 as usize] = 0x80;
        }
        if l_ctrl || r_ctrl {
            state[VK_CONTROL.0 as usize] = 0x80;
        }
        if l_alt {
            state[VK_LMENU.0 as usize] = 0x80;
        }
        if r_alt {
            state[VK_RMENU.0 as usize] = 0x80;
            // AltGr: Treiber injiziert einen synthetischen VK_LCONTROL, den der Hook
            // überspringt [LLKHF_INJECTED]. Manuell setzen, damit ToUnicodeEx
            // AltGr-Kombinationen korrekt auflöst [z. B. @ = AltGr+Q auf DE-Tastatur].
            state[VK_LCONTROL.0 as usize] = 0x80;
            state[VK_CONTROL.0 as usize] = 0x80;
        }
        if l_alt || r_alt {
            state[VK_MENU.0 as usize] = 0x80;
        }
        if CAPS_TOGGLED.load(Ordering::SeqCst) {
            state[VK_CAPITAL.0 as usize] = 0x01;
        }

        let hwnd = GetForegroundWindow();
        let tid = if hwnd.0.is_null() {
            0
        } else {
            GetWindowThreadProcessId(hwnd, None)
        };
        let hkl = GetKeyboardLayout(tid);

        let mut buf = [0u16; 8];
        // Flag 4 = KEEP_KEYBOARD_STATE_UNCHANGED [Windows 10 1607+]
        let n = ToUnicodeEx(vk, scan, &state, &mut buf, 4, hkl);
        if n <= 0 {
            return None;
        }
        String::from_utf16_lossy(&buf[..n as usize]).chars().next()
    }
}

#[cfg(windows)]
fn send_replacement(backspaces: usize, text: &str) {
    let mut inputs: Vec<INPUT> = Vec::with_capacity(backspaces * 2 + text.len() * 2);

    for _ in 0..backspaces {
        inputs.push(vk_input(VK_BACK, false));
        inputs.push(vk_input(VK_BACK, true));
    }

    for ch in text.chars() {
        if ch == '\r' {
            continue;
        }
        if ch == '\n' {
            inputs.push(vk_input(VK_RETURN, false));
            inputs.push(vk_input(VK_RETURN, true));
            continue;
        }
        let mut utf16 = [0u16; 2];
        let units = ch.encode_utf16(&mut utf16);
        for u in units.iter() {
            inputs.push(unicode_input(*u, false));
            inputs.push(unicode_input(*u, true));
        }
    }

    // SendInput in einen Worker-Thread auslagern. Wenn wir aus der Hook-
    // Prozedur heraus synchron sendeten, blockiert der Hook die System-Input-
    // Queue, kann die LowLevelHooksTimeout-Grenze reissen und sorgt bei
    // schnellem Tippen für zerhackte oder gedoppelte Ausgaben. Der Hook
    // kehrt jetzt sofort zurück, der Worker liefert die Tasten asynchron.
    std::thread::spawn(move || {
        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    });
}

#[cfg(windows)]
fn vk_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
    let flags = if key_up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: MZ_INPUT_MAGIC,
            },
        },
    }
}

#[cfg(windows)]
fn unicode_input(unit: u16, key_up: bool) -> INPUT {
    let mut flags = KEYEVENTF_UNICODE;
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: unit,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: MZ_INPUT_MAGIC,
            },
        },
    }
}
