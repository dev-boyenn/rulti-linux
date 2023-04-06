use std::{
    sync::{atomic::{AtomicBool, Ordering::SeqCst, AtomicU32, AtomicUsize}, Arc, Mutex},
    thread,
    time::{self, Duration, SystemTime}, fs::File, io::Read,
};

use crate::{
    x11::InstanceInfo,
};
use atomic_enum::atomic_enum;
use tokio::sync::mpsc::{error::TryRecvError, Receiver, Sender};

pub struct Instance {
    pub instance_info: InstanceInfo,
    pub state: AtomicInstanceState,
    pub locked: AtomicBool,
    pub thin: AtomicBool,
    pub thread_count:AtomicU32,
    pub affinity_mask : AtomicUsize,
    pub last_world_preview_modification : Arc<Mutex<SystemTime>>,
    pub last_world_preview_state : Arc<Mutex<String>>,
    pub preview_percent: AtomicUsize,
    pub has_sent_percent: AtomicBool,
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
    pub fn new(instance_info:InstanceInfo) -> Self {
        Self {
            instance_info: instance_info,
            state: AtomicInstanceState::new(InstanceState::Idle),
            locked: AtomicBool::new(false),
            thin: AtomicBool::new(false),
            thread_count:AtomicU32::new(0),
            affinity_mask:AtomicUsize::new(0),
            last_world_preview_modification : Arc::new(Mutex::new(SystemTime::now())),
            last_world_preview_state : Arc::new(Mutex::new(String::new())),
            preview_percent:AtomicUsize::new(0),
            has_sent_percent:AtomicBool::new(false),
        }
    }
    fn send_f3_esc(&self) {
        todo!("Implement f3 esc");
        
        // send_keydown(self.hwnd, VK_F3);
        // send_keydown(self.hwnd, VK_ESCAPE);
        // send_keyup(self.hwnd, VK_ESCAPE);
        // send_keyup(self.hwnd, VK_F3);
    }

    pub fn thin(&self) {
        self.thin.store(!self.thin.load(SeqCst), SeqCst);

        match self.thin.load(SeqCst) {
            true => {
                todo!("Implement thinning");
                // unsafe { MoveWindow(self.hwnd, 1920 / 2 - (300 / 2), 0, 400, 1080, true) };
            }
            false => {
                todo!("Implement thinning");
                // unsafe { MoveWindow(self.hwnd, 0, 0, 1920, 1080, true) };
            }
        }
    }

    pub fn getWorldPreviewState(&self)->String{
        todo!("Implement getWorldPreviewState");

        let i = self.instance_info.instance_num;
        let file_path = format!("C:\\Users\\Boyen\\Downloads\\MultiMC\\instances\\RSG {i}\\.minecraft\\wpstateout.txt");
        let mut file = File::open(&file_path).unwrap();
        let time = file.metadata().unwrap().modified().unwrap();
        if time > self.last_world_preview_modification.lock().unwrap().clone(){
            let mut contents = String::new();
            let mut stored_time = self.last_world_preview_modification.lock().unwrap();
            *stored_time = time;
            match file.read_to_string(&mut contents){
                Ok(_) => {}
                Err(err) => {
                    println!("Failed to read world preview state {file_path}");
                    println!("Error: {err}");
                    return String::new();
                }
            }

            let mut stored_state = self.last_world_preview_state.lock().unwrap();
            *stored_state = contents.clone();
            // println!("World preview state: {contents} (from file)");
            return contents;
        }
        return self.last_world_preview_state.lock().unwrap().clone();
    }
    pub async fn reset(
        &self,
        mut cancel_receiver: Receiver<()>,
        on_preview_ready_sender: Sender<u32>,
        on_preview_percent_sender: Sender<u32>,
    ) {
        if self.state.load(SeqCst) == InstanceState::Resetting
            || self.state.load(SeqCst) == InstanceState::LoadingScreen
        {
            println!("Trigger reset during reset, taking over");
        } else {
            // Start resetting
            todo!("Implement reset");
            // send_keypress(self.hwnd, VK_F6);
            self.state.store(InstanceState::Resetting, SeqCst);
        }
        self.has_sent_percent.store(false,SeqCst);

        loop {
            thread::sleep(Duration::from_millis(50));

            match cancel_receiver.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    // println!("Reset cancelled");
                    return;
                }
                _ => {}
            }
            match self.state.load(SeqCst) {
                InstanceState::Resetting => {


                    
                    if self.getWorldPreviewState().contains("generating") {
                        self.state.store(InstanceState::LoadingScreen, SeqCst);
                        continue;
                    }
                }
                InstanceState::LoadingScreen => {
                    if self.getWorldPreviewState().contains("previewing")  {
                        self.state.store(InstanceState::Preview, SeqCst);
                        // Hide the menu
                        self.send_f3_esc();
                        match on_preview_ready_sender.send(self.instance_info.instance_num).await {
                            Ok(_) => {}
                            Err(_) => {
                                panic!("Failed to send preview ready signal");
                            }
                        }
                        continue;
                    }
                }
                InstanceState::Preview => {
                    let state = self.getWorldPreviewState();
                    if state.contains("previewing") && !self.has_sent_percent.load(SeqCst) {
                        let percent = state.split(",").collect::<Vec<&str>>()[1]
                            .parse::<usize>()
                            .unwrap();
                        // println!("Preview percent: {percent}%");
                        self.preview_percent.store(percent, SeqCst);
                        if percent>80 {
                            on_preview_percent_sender.send(percent as u32).await.unwrap();
                            self.has_sent_percent.store(true, SeqCst);

                        }
                        continue;
                    }
                    if self.getWorldPreviewState().contains("inworld") {
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

                todo!("Implement play");

                // let foreground_window = unsafe { GetForegroundWindow() };
                // let window_thread_process_id =
                //     unsafe { GetWindowThreadProcessId(foreground_window, zeroed()) };
                // let current_thread = unsafe { GetCurrentThreadId() };
                // if window_thread_process_id != current_thread {
                //     unsafe { AttachThreadInput(window_thread_process_id, current_thread, true) };
                // }

                // // unsafe { ShowWindow(self.hwnd, SW_SHOW) };
                // unsafe { SetForegroundWindow(self.hwnd) };
                // unsafe { BringWindowToTop(self.hwnd) };
                // unsafe { AttachThreadInput(window_thread_process_id, current_thread, false) };
                // while (unsafe { GetForegroundWindow() } != self.hwnd) {
                //     thread::sleep(time::Duration::from_millis(2));
                // }
                // // print current time in miliseconds
                // println!(
                //     "Current time: {}",
                //     time::SystemTime::now()
                //         .duration_since(time::UNIX_EPOCH)
                //         .unwrap()
                //         .as_millis()
                // );
                // println!("Making fullscreen");

                // send_keydown(self.hwnd, VK_F11);
                // send_keyup(self.hwnd, VK_F11);

                // send_keydown(self.hwnd, VK_ESCAPE);
                // send_keyup(self.hwnd, VK_ESCAPE);
                // thread::sleep(time::Duration::from_millis(2));
                // send_keydown(self.hwnd, VK_ESCAPE);
                // send_keyup(self.hwnd, VK_ESCAPE);
                // thread::sleep(time::Duration::from_millis(2));
                // send_keydown(self.hwnd, VK_ESCAPE);
                // send_keyup(self.hwnd, VK_ESCAPE);
                // println!("Setting state to playing");
                // self.state.store(InstanceState::Playing, SeqCst);
                // println!("Done with playing");
            }
            _ => {}
        }
    }

    pub fn exit(&self) {
        println!("Exiting");
        todo!("Implement Exit");

        if self.thin.load(SeqCst) {
            self.thin();
        }
        // send_keydown(self.hwnd, VK_F11);
        // send_keyup(self.hwnd, VK_F11);
    }

    pub fn lock(&self) {
        self.locked.store(true, SeqCst);
    }

    pub fn unlock(&self) {
        self.locked.store(false, SeqCst);
    }

    pub fn set_threadcount(&self, thread_count: u32){
        if self.thread_count.load(SeqCst) ==  thread_count{
            return;
        }
        self.thread_count.store(thread_count, SeqCst);

        let affinity = (1 << thread_count) - 1;
        let mask = if thread_count == 2 { affinity << (self.instance_info.instance_num * 2) } else { affinity };
        self.set_affinity(mask);
    }

    pub fn set_affinity(&self, affinity_mask : usize){
        // if self.affinity_mask.load(SeqCst) ==  affinity_mask{
        //     return;
        // }
        // self.affinity_mask.store(affinity_mask, SeqCst);

        // let process_handle = unsafe { OpenProcess(PROCESS_ACCESS_RIGHTS(0x0200), false, self.process_id) };
        
        // match process_handle {
        //     Ok(handle) => {
        //         unsafe { SetProcessAffinityMask(handle, affinity_mask) };
        //         unsafe { CloseHandle(handle) };
        //     }
        //     Err(e) => {
        //         println!("Error: {}", e);
        //     }
        // }
    }
}
