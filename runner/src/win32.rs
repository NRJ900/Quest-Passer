
#![cfg(target_os = "windows")]

use std::env;
pub use std::ffi::OsStr; // pub use for re-export if needed, or just use
use std::os::windows::ffi::OsStrExt;
use std::time::Instant;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetDC, GetStockObject, GetTextExtentPoint32W, ReleaseDC, SetBkMode, SetTextColor, SetBkColor, 
    CreateFontW, SelectObject, HDC, HFONT, NULL_BRUSH, TRANSPARENT, OPAQUE, FONT_CHARSET, 
    FONT_OUTPUT_PRECISION, FONT_CLIP_PRECISION, FONT_QUALITY, FONT_PITCH, FW_BOLD, FW_NORMAL,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowLongPtrW, PostQuitMessage, RegisterClassW, SetTimer, SetWindowLongPtrW, SetWindowPos,
    SetWindowTextW, ShowWindow, TranslateMessage, CW_USEDEFAULT, GWL_EXSTYLE, MSG, SW_HIDE,
    SW_SHOWNORMAL, SWP_NOZORDER, TIMERPROC, WINDOW_EX_STYLE, WINDOW_STYLE, 
    WM_CTLCOLORSTATIC, WM_DESTROY, WM_SIZE, WM_TIMER, WNDCLASSW, WS_CHILD, WS_EX_APPWINDOW, 
    WM_SETICON, HICON, CreateIcon, ICON_BIG, ICON_SMALL, SW_SHOWNOACTIVATE,
    SendMessageA, WM_SETFONT, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_EX_LAYERED
};
use windows::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_PROGRESS_CLASS, INITCOMMONCONTROLSEX,
    PBM_SETRANGE32, PBM_SETPOS,
};
use std::io::Read;
use std::ffi::c_void;

// SS_CENTER is 1. Defining it locally as WINDOW_STYLE(1)
const SS_CENTER: WINDOW_STYLE = WINDOW_STYLE(1);

use crate::tray;
use tray::create_tray_icon;

// Constants for layout and theme
const WIDTH: i32 = 400;
const HEIGHT: i32 = 300;
const BACKGROUND_COLOR: u32 = 0x001E1E2E; // Dark blue-gray
const TEXT_COLOR: u32 = 0x00CDD6F4; // Light gray/white text

static mut TITLE_LABEL: Option<HWND> = None;
static mut START_TIME: Option<Instant> = None;
static mut DURATION_LABEL: Option<HWND> = None;
static mut PROGRESS_BAR_HWND: Option<HWND> = None;

#[derive(Debug)]
pub struct Config {
    pub title: String,
    pub start_minimized: bool,
    pub icon_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            title: "Quest Passer".to_string(),
            start_minimized: false,
            icon_url: None,
        }
    }
}

pub fn parse_args() -> Config {
    let args: Vec<String> = env::args().collect();
    let mut config = Config::default();

    let mut i = 1; // Skip program name
    while i < args.len() {
        match args[i].as_str() {
            "--title" => {
                if i + 1 < args.len() {
                    config.title = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--icon" => {
                if i + 1 < args.len() {
                    config.icon_url = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--tray" => {
                config.start_minimized = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    config
}

// Helper to convert Rust string to wide string (UTF-16) for Windows APIs
fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn calculate_text_width(hwnd: HWND, text: &str) -> i32 {
    unsafe {
        let hdc = GetDC(Some(hwnd));
        if hdc.is_invalid() {
            return 200; // fallback width
        }

        let wide_text = to_wstring(text);
        let mut size = SIZE::default();

        let result = GetTextExtentPoint32W(hdc, &wide_text, &mut size);

        let _ = ReleaseDC(Some(hwnd), hdc);

        if result.as_bool() {
            size.cx + 20 // Add 20px padding
        } else {
            200 // fallback width
        }
    }
}

fn create_styled_label(
    parent_hwnd: HWND,
    text: &str,
    instance: HINSTANCE,
    y: i32,
    h: i32,
    font_size: i32,
    is_bold: bool,
) -> Option<HWND> {
    unsafe {
        let class_name = to_wstring("STATIC");
        let window_text = to_wstring(text);
        
        // Calculate width to center it (approximate)
        let width = calculate_text_width(parent_hwnd, text) + 40; // Add padding
        let x = (WIDTH - width) / 2; // Center horizontally

        let label_hwnd = CreateWindowExW(
            WS_EX_TRANSPARENT,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | SS_CENTER, // Center text style
            x,
            y,
            width,
            h,
            Some(parent_hwnd),
            None,
            Some(instance),
            None,
        );

        if let Ok(hwnd) = label_hwnd {
            if !hwnd.0.is_null() {
                // Set font
                let weight = if is_bold { FW_BOLD } else { FW_NORMAL };
                let font_name = to_wstring("Segoe UI");
                let hfont = CreateFontW(
                    font_size, 
                    0, 0, 0, 
                    std::mem::transmute(weight), 
                    0, 0, 0, 
                    FONT_CHARSET(0), // ANSI_CHARSET
                    FONT_OUTPUT_PRECISION(0), // OUT_DEFAULT_PRECIS
                    FONT_CLIP_PRECISION(0), // CLIP_DEFAULT_PRECIS
                    FONT_QUALITY(0), // DEFAULT_QUALITY (or PROOF_QUALITY for better rendering?)
                    std::mem::transmute(0u32), // FONT_PITCH | FONT_FAMILY
                    PCWSTR(font_name.as_ptr())
                );
                
                // Send WM_SETFONT message
                let _ = windows::Win32::UI::WindowsAndMessaging::SendMessageA(
                    hwnd,
                    windows::Win32::UI::WindowsAndMessaging::WM_SETFONT,
                    WPARAM(hfont.0 as usize),
                    LPARAM(1),
                );
                
                return Some(hwnd);
            }
        }
        None
    }
}

// Window procedure for handling messages
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_CTLCOLORSTATIC => {
            let hdc = HDC(wparam.0 as *mut _);
            let control_hwnd = HWND(lparam.0 as *mut _);

            // Check if this is the timer label
            let is_timer = unsafe { DURATION_LABEL == Some(control_hwnd) };

            if is_timer {
                SetBkMode(hdc, windows::Win32::Graphics::Gdi::OPAQUE);
                SetBkColor(hdc, COLORREF(BACKGROUND_COLOR));
            } else {
                SetBkMode(hdc, TRANSPARENT);
            }

            SetTextColor(hdc, COLORREF(TEXT_COLOR)); // White text
            
            // Return null brush. For opaque mode, this means the background isn't painted by the system,
            // but the text draws its own background.
            LRESULT(GetStockObject(NULL_BRUSH).0 as isize)
        }
        WM_TIMER => {
            if let Some(start) = START_TIME {
                let elapsed = start.elapsed();
                let secs = elapsed.as_secs();
                let hours = secs / 3600;
                let minutes = (secs % 3600) / 60;
                let seconds = secs % 60;
                let time_str = format!("Running: {:02}:{:02}:{:02}", hours, minutes, seconds);

                if let Some(label_hwnd) = DURATION_LABEL {
                    let wide_text = to_wstring(&time_str);
                    let _ = SetWindowTextW(label_hwnd, PCWSTR(wide_text.as_ptr()));
                }

                if let Some(pb_hwnd) = PROGRESS_BAR_HWND {
                    // Cap at 900
                    let progress = if secs > 900 { 900 } else { secs };
                    let _ = windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                        pb_hwnd,
                        PBM_SETPOS,
                        Some(WPARAM(progress as usize)),
                        Some(LPARAM(0)),
                    );
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn create_native_window(title: &str) -> Result<(HWND, HINSTANCE), Box<dyn std::error::Error>> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = to_wstring("QuestPasser");
        let window_title = to_wstring(title);

        // Create a dark background brush
        let brush = windows::Win32::Graphics::Gdi::CreateSolidBrush(
            COLORREF(BACKGROUND_COLOR) 
        );

        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: HINSTANCE(instance.0),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hbrBackground: brush, // Set background brush
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WIDTH,
            HEIGHT,
            None,
            None,
            Some(HINSTANCE(instance.0)),
            None,
        );
        match hwnd {
            Ok(hwnd) if !hwnd.0.is_null() => Ok((hwnd, HINSTANCE(instance.0))),
            _ => Err("Failed to create window".into()),
        }
    }
}

fn log_debug(msg: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("runner_debug.txt") {
        let _ = writeln!(file, "{}", msg);
    }
}

fn load_icon_from_url(url: &str) -> Option<HICON> {
    log_debug(&format!("Attempting to load icon: {}", url));
    
    // 1. Download
    let resp = ureq::get(url)
        .set("User-Agent", "QuestPasser/1.0 (gzip)")
        .call();
        
    if let Err(e) = &resp {
        log_debug(&format!("Download failed: {}", e));
        return None;
    }
    
    let resp = resp.unwrap();
    let mut bytes = Vec::new();
    if let Err(e) = resp.into_reader().read_to_end(&mut bytes) {
        log_debug(&format!("Read failed: {}", e));
        return None;
    }
    log_debug(&format!("Downloaded {} bytes", bytes.len()));

    // 2. Decode
    let img_result = image::load_from_memory(&bytes);
    if let Err(e) = &img_result {
        log_debug(&format!("Decode failed: {}", e));
        return None;
    }
    
    let img = img_result.unwrap().to_rgba8();
    let width = img.width() as i32;
    let height = img.height() as i32;
    let mut rgba = img.into_raw();
    
    log_debug(&format!("Decoded image: {}x{}", width, height));

    // 3. Convert RGBA to BGRA (for Windows)
    for chunk in rgba.chunks_exact_mut(4) {
        chunk.swap(0, 2); // Swap R and B
    }
    
    // 4. Create Icon using CreateIcon
    let mask_bits = vec![0u8; (width * height / 8) as usize]; // Dummy mask

    unsafe {
        let module_handle = GetModuleHandleW(None).unwrap_or_default();
        let instance: HINSTANCE = std::mem::transmute(module_handle);
        let hicon = CreateIcon(
            Some(instance),
            width,
            height,
            1, // Planes (must be 1 for icons)
            32, // Bits per pixel
            mask_bits.as_ptr(),
            rgba.as_ptr(),
        ).ok();
        
        if hicon.is_some() {
            log_debug("CreateIcon success");
        } else {
            log_debug("CreateIcon failed");
        }
        hicon
    }
}

pub fn run() {
    log_debug("Runner started!");
    let args: Vec<String> = env::args().collect();
    for (i, arg) in args.iter().enumerate() {
        log_debug(&format!("Arg {}: {}", i, arg));
    }

    let config = parse_args();
    log_debug("Args parsed");

    let tray_menu = tray_icon::menu::Menu::new();
    let quit_i = tray_icon::menu::MenuItem::new("Quit", true, None);
    let show_i = tray_icon::menu::MenuItem::new("Show", true, None);
    let hide_i = tray_icon::menu::MenuItem::new("Hide", true, None);

    let _tray_menu = tray_menu.append_items(&[
        &show_i,
        &hide_i,
        &tray_icon::menu::PredefinedMenuItem::separator(),
        &quit_i,
    ]);
    log_debug("Tray menu created");

    let _tray = create_tray_icon(tray_menu, &config.title);
    log_debug("Tray icon created");

    // Create native Windows window
    log_debug("Creating native window...");
    let (hwnd, instance) = match create_native_window(&config.title) {
        Ok(result) => result,
        Err(e) => {
            log_debug(&format!("Failed to create window: {}", e));
            return;
        }
    };
    log_debug("Native window created");

    // Load and set icon if provided
    if let Some(url) = &config.icon_url {
        if let Some(hicon) = load_icon_from_url(url) {
            unsafe {
                let hicon_handle: isize = std::mem::transmute(hicon);
                
                // Use SendMessageW to ensure the icon is set immediately
                let result_big = windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                    hwnd,
                    WM_SETICON,
                    Some(WPARAM(1)), // ICON_BIG
                    Some(LPARAM(hicon_handle)),
                );
                log_debug(&format!("SendMessageW ICON_BIG result: {:?}", result_big));

                let result_small = windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                    hwnd,
                    WM_SETICON,
                    Some(WPARAM(0)), // ICON_SMALL
                    Some(LPARAM(hicon_handle)),
                );
                log_debug(&format!("SendMessageW ICON_SMALL result: {:?}", result_small));
            }
        }
    }

    // 1. Title (Large, Bold)
    let title_label_hwnd = create_styled_label(
        hwnd,
        &config.title,
        instance,
        50, // y position
        40, // height
        24, // font size
        true // bold
    );

    // 2. "Quest Passer" branding
    let _app_label_hwnd = create_styled_label(
        hwnd,
        "Quest Passer",
        instance,
        20,
        30,
        16,
        true
    );

    // 3. Subtitle description
    let _app_label_hwnd = create_styled_label(
        hwnd,
        "Active Game Session",
        instance,
        90,
        25,
        18,
        false
    );

    // 4. Timer (Bottom)
    let duration_label_hwnd = create_styled_label(
        hwnd, 
        "Running: 00:00:00", 
        instance, 
        HEIGHT - 100, 
        30, 
        20, 
        false
    );

    // 5. Progress Bar (Dynamic)
    let progress_bar_hwnd = unsafe {
        // Initialize Common Controls for Progress Bar
        let iccex = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_PROGRESS_CLASS,
        };
        let _ = InitCommonControlsEx(&iccex);

        let progress_class = to_wstring("msctls_progress32");
        let pb = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(progress_class.as_ptr()),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(1), // PBS_SMOOTH
            20, 
            HEIGHT - 65, // Position just above status, below active game text
            WIDTH - 40, 
            6, // Height
            Some(hwnd), // Parent window
            None,
            Some(instance),
            None
        );
        
        if let Ok(pb) = pb {
            // Set Range 0 to 900 (15 minutes in seconds)
            let _ = windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                pb,
                PBM_SETRANGE32,
                Some(WPARAM(0)),
                Some(LPARAM(900)),
            );
            Some(pb)
        } else {
            None
        }
    };

    // Store control references in static variables
    unsafe {
        TITLE_LABEL = title_label_hwnd;
        DURATION_LABEL = duration_label_hwnd;
        PROGRESS_BAR_HWND = progress_bar_hwnd;
        START_TIME = Some(Instant::now());
        // SetTimer expecting: HWND, nIDEvent, uElapse, lpTimerFunc
        let timer: TIMERPROC = None;
        SetTimer(Some(hwnd), 1, 1000, timer);
    }

    unsafe {
        if config.start_minimized {
            // Only modify window styles when starting minimized
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let new_ex_style = (ex_style & !WS_EX_APPWINDOW.0 as isize)
                | WS_EX_TOOLWINDOW.0 as isize
                | WS_EX_TRANSPARENT.0 as isize
                | WS_EX_LAYERED.0 as isize;

            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style);
            let _ = ShowWindow(hwnd, SW_HIDE);
        } else {
            // For normal window, just show it without modifying styles
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }

        // Windows message loop
        let mut msg = MSG::default();
        loop {
            // Handle tray event
            if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                if event.id == quit_i.id() {
                    PostQuitMessage(0);
                }

                if event.id == show_i.id() {
                    // Always restore window styles for normal display when showing from tray
                    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                    let new_ex_style = (ex_style
                        & !(WS_EX_TOOLWINDOW.0 as isize
                            | WS_EX_TRANSPARENT.0 as isize
                            | WS_EX_LAYERED.0 as isize))
                        | WS_EX_APPWINDOW.0 as isize;

                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style);
                    let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
                    let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
                }

                if event.id == hide_i.id() {
                    // Hide window to tray (similar to --tray startup behavior)
                    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                    let new_ex_style = (ex_style & !WS_EX_APPWINDOW.0 as isize)
                        | WS_EX_TOOLWINDOW.0 as isize
                        | WS_EX_TRANSPARENT.0 as isize
                        | WS_EX_LAYERED.0 as isize;

                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style);
                    let _ = ShowWindow(hwnd, SW_HIDE);
                }
            }

            let ret = GetMessageW(&mut msg, None, 0, 0);
            if ret.0 == 0 || ret.0 == -1 {
                break;
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
