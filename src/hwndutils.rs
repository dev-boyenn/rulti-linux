use std::mem::zeroed;

use windows::{Win32::{Foundation::{HWND, BOOL, LPARAM, POINT, RECT}, UI::WindowsAndMessaging::*, System::Threading::{GetCurrentThreadId, AttachThreadInput}}, core::PCWSTR};

pub fn enum_windows<F: FnMut(HWND) -> BOOL>(mut f: F) -> BOOL {
    unsafe extern "system" fn proc<F: FnMut(HWND) -> BOOL>(window: HWND, userdata: LPARAM) -> BOOL {
        let f = userdata.0 as *mut F;
        (*f)(window)
    }

    let userdata = LPARAM(&mut f as *mut _ as isize);
    let func: WNDENUMPROC = Some(proc::<F>);
    unsafe { EnumWindows(func, userdata) }
}
pub fn get_wall_hwnd()->HWND{
    let mut wall_hwnd = HWND::default();
    enum_windows(|hwnd| {
        let text = get_hwnd_title(hwnd);
        if text.contains("Fullscreen Projector") {
            wall_hwnd = hwnd;
            return false.into();
        }
        true.into()
    });
    wall_hwnd
}

pub fn is_full_screen(hwnd:HWND)->bool{
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) };
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    
    let is_full_screen = width == 1920 && height == 1080;
    is_full_screen
}
pub fn get_hwnd_pid(hwnd:HWND)-> u32{

    let mut p_id:u32= 0;
    let _thread_id = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut p_id)) };
    p_id
}

pub fn set_borderless(hwnd: HWND) {
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) as u32};
    let new_style = style & !WS_BORDER.0  & !WS_CAPTION.0 & !WS_THICKFRAME.0;
    unsafe { SetWindowLongPtrW(hwnd, GWL_STYLE, new_style as isize) };
}

pub fn set_hwnd_title(hwnd:HWND, title:String){
    assert!(title.ends_with("\0"));
    let title = title.encode_utf16().collect::<Vec<u16>>();
    unsafe { SetWindowTextW(hwnd, PCWSTR::from_raw(title.as_ptr())) };
}

pub fn get_hwnd_title(hwnd:HWND) -> String {
    let mut text: [u16; 512] = [0; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut text) };
    String::from_utf16_lossy(&text[..len as usize])
}

pub fn is_active(hwnd:HWND) -> bool {
    let active_hwnd = unsafe { GetForegroundWindow() };
    active_hwnd == hwnd
}
pub fn get_mouse_pos()->(i32,i32){
    let mut pos = POINT::default();
    unsafe { GetCursorPos(&mut pos) };
    (pos.x, pos.y)
}

pub fn activate_hwnd(hwnd:HWND){
    let foreground_window = unsafe { GetForegroundWindow() };
    let window_thread_process_id =
        unsafe { GetWindowThreadProcessId(foreground_window, zeroed()) };
    let current_thread = unsafe { GetCurrentThreadId() };
    if window_thread_process_id != current_thread {
        unsafe { AttachThreadInput(window_thread_process_id, current_thread, true) };
    }

    // unsafe { ShowWindow(self.hwnd, SW_SHOW) };
    unsafe { SetForegroundWindow(hwnd) };
    unsafe { BringWindowToTop(hwnd) };
    unsafe { AttachThreadInput(window_thread_process_id, current_thread, false) };
}