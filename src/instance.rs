use std::{
    mem::zeroed,
    sync::atomic::{
        AtomicBool,
        Ordering::{SeqCst},
    },
    thread,
    time::{self, Duration},
};

use crate::{
    hwndutils,
    keyboardutils::{send_keydown, send_keypress, send_keyup},
};
use atomic_enum::atomic_enum;
use tokio::sync::mpsc::{error::TryRecvError, Receiver, Sender};
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{GetDC, GetPixel, ReleaseDC},
    System::Threading::{AttachThreadInput, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::{VK_ESCAPE, VK_F11, VK_F3, VK_F6},
        WindowsAndMessaging::{
            BringWindowToTop, GetClientRect, GetForegroundWindow, GetWindowTextW,
            GetWindowThreadProcessId, MoveWindow, SetForegroundWindow,
        },
    },
};

pub struct Instance {
    pub hwnd: HWND,
    pub state: AtomicInstanceState,
    pub instance_num: u32,
    pub locked: AtomicBool,
    pub thin: AtomicBool,
}
#[derive(strum_macros::Display)]
#[atomic_enum]
#[derive(PartialEq)]
pub enum InstanceState {
    Idle,
    Resetting,
    LoadingScreen,
    Preview,
    Playing,
}
const READY: &str = "Minecraft* 1.16.1 - Singleplayer";
impl Instance {
    pub fn new(hwnd: HWND, instance_num: u32) -> Self {
        Self {
            hwnd,
            state: AtomicInstanceState::new(InstanceState::Idle),
            instance_num,
            locked: AtomicBool::new(false),
            thin: AtomicBool::new(false),
        }
    }
    fn send_f3_esc(&self) {
        send_keydown(self.hwnd, VK_F3);
        send_keydown(self.hwnd, VK_ESCAPE);
        send_keyup(self.hwnd, VK_ESCAPE);
        send_keyup(self.hwnd, VK_F3);
    }

    pub fn thin(&self) {
        self.thin.store(!self.thin.load(SeqCst), SeqCst);

        match self.thin.load(SeqCst) {
            true => {
                unsafe { MoveWindow(self.hwnd, 1920 / 2 - (300 / 2), 0, 400, 1080, true) };
            }
            false => {
                unsafe { MoveWindow(self.hwnd, 0, 0, 1920, 1080, true) };
            }
        }
    }
    pub async fn reset(
        &self,
        mut cancel_receiver: Receiver<()>,
        on_preview_ready_sender: Sender<u32>,
    ) {
        if self.state.load(SeqCst) == InstanceState::Resetting
            || self.state.load(SeqCst) == InstanceState::LoadingScreen
        {
            println!("Trigger reset during reset, taking over");
        } else {
            // Start resetting
            send_keypress(self.hwnd, VK_F6);
            self.state.store(InstanceState::Resetting, SeqCst);
        }

        loop {
            thread::sleep(Duration::from_millis(50));

            match cancel_receiver.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    println!("Reset cancelled");
                    return;
                }
                _ => {}
            }
            match self.state.load(SeqCst) {
                InstanceState::Resetting => {
                    if self.is_in_loading_screen() {
                        self.state.store(InstanceState::LoadingScreen, SeqCst);
                        continue;
                    }
                }
                InstanceState::LoadingScreen => {
                    if !self.is_in_loading_screen() {
                        self.state.store(InstanceState::Preview, SeqCst);
                        // Hide the menu
                        self.send_f3_esc();
                        match on_preview_ready_sender.send(self.instance_num).await {
                            Ok(_) => {}
                            Err(_) => {
                                panic!("Failed to send preview ready signal");
                            }
                        }
                        continue;
                    }
                }
                InstanceState::Preview => {
                    if self.get_title() == READY {
                        self.state.store(InstanceState::Idle, SeqCst);
                        // Pause the game
                        self.send_f3_esc();
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    pub fn play(&self) {
        match self.state.load(SeqCst) {
            InstanceState::Idle => {
                println!("Playing");
                // print current time in miliseconds
                println!(
                    "Current time: {}",
                    time::SystemTime::now()
                        .duration_since(time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
                let foreground_window = unsafe { GetForegroundWindow() };
                let window_thread_process_id =
                    unsafe { GetWindowThreadProcessId(foreground_window, zeroed()) };
                let current_thread = unsafe { GetCurrentThreadId() };
                if window_thread_process_id != current_thread {
                    unsafe { AttachThreadInput(window_thread_process_id, current_thread, true) };
                }

                // unsafe { ShowWindow(self.hwnd, SW_SHOW) };
                unsafe { SetForegroundWindow(self.hwnd) };
                unsafe { BringWindowToTop(self.hwnd) };
                unsafe { AttachThreadInput(window_thread_process_id, current_thread, false) };
                while (unsafe { GetForegroundWindow() } != self.hwnd) {
                    thread::sleep(time::Duration::from_millis(10));
                }
                // print current time in miliseconds
                println!(
                    "Current time: {}",
                    time::SystemTime::now()
                        .duration_since(time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
                println!("Making fullscreen");

                send_keydown(self.hwnd, VK_F11);
                send_keyup(self.hwnd, VK_F11);

                send_keydown(self.hwnd, VK_ESCAPE);
                send_keyup(self.hwnd, VK_ESCAPE);
                thread::sleep(time::Duration::from_millis(10));
                send_keydown(self.hwnd, VK_ESCAPE);
                send_keyup(self.hwnd, VK_ESCAPE);
                thread::sleep(time::Duration::from_millis(10));
                send_keydown(self.hwnd, VK_ESCAPE);
                send_keyup(self.hwnd, VK_ESCAPE);
                println!("Setting state to playing");
                self.state.store(InstanceState::Playing, SeqCst);
                println!("Done with playing");
            }
            _ => {}
        }
    }

    pub fn exit(&self) {
        println!("Exiting");
        if self.thin.load(SeqCst) {
            self.thin();
        }
        send_keydown(self.hwnd, VK_F11);
        send_keyup(self.hwnd, VK_F11);
    }

    pub fn lock(&self) {
        self.locked.store(true, SeqCst);
    }

    pub fn unlock(&self) {
        self.locked.store(false, SeqCst);
    }

    pub fn get_title(&self) -> String {
        let mut text: [u16; 512] = [0; 512];
        let len = unsafe { GetWindowTextW(self.hwnd, &mut text) };
        let text = String::from_utf16_lossy(&text[..len as usize]);
        text
    }

    pub fn is_in_loading_screen(&self) -> bool {
        let mut my_rect = unsafe { zeroed() };
        let _client_rect = unsafe { GetClientRect(self.hwnd, &mut my_rect) };
        let y: i32 = my_rect.bottom - 1;
        let dc = unsafe { GetDC(self.hwnd) };
        let pixel = unsafe { GetPixel(dc, 0, y).0 };
        let is_loading_screen_pixel = pixel == 1515822;
        unsafe { ReleaseDC(self.hwnd, dc) };
        is_loading_screen_pixel
    }

    pub fn set_instance_title(&self) {
        let title = format!("Minecraft* - Instance {}\0", self.instance_num);
        hwndutils::set_hwnd_title(self.hwnd, title);
    }
}
