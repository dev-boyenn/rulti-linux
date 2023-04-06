// use windows::Win32::{
//     Foundation::{HWND, LPARAM, WPARAM},
//     UI::{
//         Input::KeyboardAndMouse::{
//             MapVirtualKeyA, VIRTUAL_KEY, VK_DELETE, VK_DIVIDE, VK_DOWN, VK_END, VK_HOME, VK_INSERT,
//             VK_LEFT, VK_NEXT, VK_NUMLOCK, VK_PRIOR, VK_RCONTROL, VK_RIGHT, VK_RMENU, VK_UP,
//         },
//         WindowsAndMessaging::{PostMessageA, WM_KEYDOWN, WM_KEYUP},
//     },
// };

// pub fn send_keypress(hwnd: HWND, key: VIRTUAL_KEY) {
//     send_keydown(hwnd, key);
//     send_keyup(hwnd, key);
// }

// pub fn send_keydown(hwnd: HWND, key: VIRTUAL_KEY) {
//     unsafe {
//         PostMessageA(
//             hwnd,
//             WM_KEYDOWN,
//             WPARAM(key.0 as usize),
//             create_l_param(key, 1, false, false, false),
//         );
//     }
// }
// pub fn send_keyup(hwnd: HWND, key: VIRTUAL_KEY) {
//     unsafe {
//         PostMessageA(
//             hwnd,
//             WM_KEYUP,
//             WPARAM(key.0 as usize),
//             create_l_param(key, 1, true, true, false),
//         );
//     }
// }

// pub fn click_top_left(hwnd: HWND) {
//     unsafe { PostMessageA(hwnd, 0x0201, WPARAM(1), LPARAM(0)) };
// }

// fn virtual_key_to_scan_code(virtual_key: VIRTUAL_KEY) -> (i32, bool) {
//     let scan_code = unsafe { MapVirtualKeyA(virtual_key.0 as u32, 0) as i32 };
//     let mut is_extended = false;
//     match virtual_key {
//         VK_RMENU | VK_RCONTROL | VK_LEFT | VK_UP | VK_RIGHT | VK_DOWN | VK_PRIOR | VK_NEXT
//         | VK_END | VK_HOME | VK_INSERT | VK_DELETE | VK_DIVIDE | VK_NUMLOCK => {
//             is_extended = true;
//         }
//         _ => {}
//     }
//     (scan_code, is_extended)
// }

// fn create_l_param(
//     virtual_key: VIRTUAL_KEY,
//     repeat_count: i32,
//     transition_state: bool,
//     previous_key_state: bool,
//     context_code: bool,
// ) -> LPARAM {
//     let (scan_code, is_extended) = virtual_key_to_scan_code(virtual_key);
//     let l_param = LPARAM(
//         ((transition_state as isize) << 31)
//             | ((previous_key_state as isize) << 30)
//             | ((context_code as isize) << 29)
//             | ((is_extended as isize) << 24)
//             | ((scan_code as isize) << 16)
//             | repeat_count as isize,
//     );
//     l_param
// }
