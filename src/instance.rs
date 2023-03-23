use std::{
    mem::{zeroed},
    thread, time, sync::Mutex,
};


use windows::Win32::{Foundation::{HWND, COLORREF}, UI::{Input::KeyboardAndMouse::{VK_F6, VK_F3, VK_ESCAPE}, WindowsAndMessaging::{GetWindowTextW, GetClientRect}}, Graphics::Gdi::{GetDC, GetPixel}};

use crate::keyboardutils::{send_keydown, send_keypress, send_keyup};

pub struct Instance {
    hwnd:  HWND,
}
impl Instance {
    pub fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }
    pub fn reset(&self) {
        send_keypress(self.hwnd, VK_F6);
        while !self.has_preview() {
            thread::sleep(time::Duration::from_millis(20));
        }
        while self.has_preview() {
            thread::sleep(time::Duration::from_millis(20));
            
        }
        send_keydown(self.hwnd, VK_F3);
        send_keydown(self.hwnd, VK_ESCAPE);
        send_keyup(self.hwnd, VK_ESCAPE);
        send_keyup(self.hwnd, VK_F3);

        while self.get_title() != "Minecraft* 1.16.1 - Singleplayer" {
            thread::sleep(time::Duration::from_millis(20));
        }

        send_keydown(self.hwnd, VK_F3);
        send_keydown(self.hwnd, VK_ESCAPE);
        send_keyup(self.hwnd, VK_ESCAPE);
        send_keyup(self.hwnd, VK_F3);
    }
    pub fn get_title(&self) -> String {
        let mut v: Vec<u16> = Vec::with_capacity(255);
        unsafe {
            let read_len = GetWindowTextW(self.hwnd, &mut v);
            v.set_len(read_len as usize);
            let title = String::from_utf16_lossy(&v);
            println!("{title}");
            title
        }
    }

    pub fn has_preview(&self) -> bool {
        unsafe {
            let mut my_rect =  zeroed();
            let _client_rect =  GetClientRect(self.hwnd, &mut my_rect) ;

            GetClientRect(self.hwnd, &mut my_rect);

            let y: i32 = my_rect.bottom - 1;
            let dc = GetDC(self.hwnd);
            let pixel = GetPixel(dc, 0, y).0;
            let is_loading_screen_pixel = pixel == 1515822;
            println!("{pixel} {is_loading_screen_pixel}");
            is_loading_screen_pixel
        }
    }
}
