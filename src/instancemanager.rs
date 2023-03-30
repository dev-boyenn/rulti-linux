use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    sync::{atomic::Ordering::SeqCst, Arc},
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{channel, Sender};
use windows::Win32::{
    Foundation::{CloseHandle, HWND},
    System::Threading::{OpenProcess, SetProcessAffinityMask, PROCESS_ACCESS_RIGHTS},
    UI::WindowsAndMessaging::MoveWindow,
};

use crate::{
    hwndutils::{self, get_hwnd_pid},
    instance::{Instance, InstanceState},
    keyboardutils::click_top_left,
};
const GAME_TITLE: &str = "Minecraft*";

pub struct InstanceManager {
    pub instances: Vec<Arc<Instance>>,
    reset_cancel_channels: HashMap<u32, Sender<()>>,
    pub preview_unlocked_wall_queue: WallQueue,
    pub locked_instances: Vec<Arc<Instance>>,
    instance_becomes_preview_sender: Sender<u32>,
}
pub fn set_affinity(hwnd: HWND, thread_count: u32) {
    let process_id = get_hwnd_pid(hwnd);
    let process_handle = unsafe { OpenProcess(PROCESS_ACCESS_RIGHTS(0x0200), false, process_id) };
    let affinity = (1 << thread_count) - 1;
    match process_handle {
        Ok(handle) => {
            unsafe { SetProcessAffinityMask(handle, affinity as usize) };
            unsafe { CloseHandle(handle) };
            // println!(
            //     "Set affinity of process {} to {:#018b} with wanted thread count {}",
            //     process_id, affinity, thread_count
            // );
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

impl InstanceManager {
    fn new(preview_becomes_ready_sender: Sender<u32>) -> Self {
        Self {
            instances: Vec::new(),
            reset_cancel_channels: HashMap::new(),
            locked_instances: Vec::new(),
            preview_unlocked_wall_queue: WallQueue::new(),
            instance_becomes_preview_sender: preview_becomes_ready_sender,
        }
    }
    pub fn update_affinities(&self) {
        let playing_instance = self.get_playing_instance();

        match playing_instance {
            Some(instance) => {
                set_affinity(instance.hwnd, 30);
                self.instances
                    .iter()
                    .for_each(|instance| match instance.state.load(SeqCst) {
                        InstanceState::Idle | InstanceState::Preview => {
                            set_affinity(instance.hwnd, 4)
                        }
                        InstanceState::Resetting | InstanceState::LoadingScreen => {
                            set_affinity(instance.hwnd, 8)
                        }
                        InstanceState::Playing => (),
                    });
            }
            None => {
                self.instances
                    .iter()
                    .for_each(|instance| match instance.state.load(SeqCst) {
                        InstanceState::Idle => set_affinity(instance.hwnd, 4),
                        InstanceState::Preview => match instance.locked.load(SeqCst) {
                            true => set_affinity(instance.hwnd, 30),
                            false => set_affinity(instance.hwnd, 16),
                        },
                        InstanceState::Resetting | InstanceState::LoadingScreen => {
                            set_affinity(instance.hwnd, 30)
                        }
                        InstanceState::Playing => (),
                    });

                match self.locked_instances.get(0){
                    Some(instance) => {
                        set_affinity(instance.hwnd, 30);
                    }
                    None => ()
                }
            }
        }
    }

    pub fn initialize(preview_becomes_ready_sender: Sender<u32>) -> Self {
        let mut instance_manager = Self::new(preview_becomes_ready_sender);

        // Enumerate all windows and find the ones that match the game title, creating instances for them
        hwndutils::enum_windows(|hwnd| unsafe {
            let text = hwndutils::get_hwnd_title(hwnd);
            if text.contains(GAME_TITLE) {
                set_affinity(hwnd, 30);

                println!("found instance window");
                let process_id = get_hwnd_pid(hwnd);
                let instance = Instance::new(hwnd, get_instance_num(process_id));

                instance.set_instance_title();
                hwndutils::set_borderless(hwnd);
                MoveWindow(hwnd, 0, 680, 1920, 400, true);
                click_top_left(hwnd);
                let instance_arc = Arc::new(instance);

                instance_manager.instances.push(instance_arc.clone());
                instance_manager
                    .preview_unlocked_wall_queue
                    .push(instance_arc.clone());
            }

            true.into()
        });
        instance_manager
    }

    pub fn reset_all_instances(&mut self) {
        let cloned_instances = self.instances.iter().cloned().collect::<Vec<_>>();
        self.preview_unlocked_wall_queue.clear();
        for instance in cloned_instances {
            self.reset_instance(instance.clone());
        }
    }

    pub fn reset_wall_bag(&mut self) {
        let cloned_instances = self
            .preview_unlocked_wall_queue
            .pop()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        for instance in cloned_instances {
            println!("Resetting instance: {}", instance.instance_num);
            self.reset_instance(instance.clone());
        }
    }

    pub fn reset_instance(&mut self, instance: Arc<Instance>) {
        match self.reset_cancel_channels.get(&instance.instance_num) {
            Some(sender) => match sender.send(()) {
                _ => (),
            },
            _ => (),
        }

        let cancel_channel = channel(1); // TODO: Figure out bound size
        self.reset_cancel_channels
            .insert(instance.instance_num, cancel_channel.0);
        let sender = self.instance_becomes_preview_sender.clone();
        tokio::spawn(async move {
            instance.reset(cancel_channel.1, sender).await;
        });
    }
    pub fn lock(&mut self, instance_num: u32) {
        let instance = self.get_instance_by_instance_num(instance_num).unwrap();
        instance.lock();
        self.locked_instances.push(instance.clone());
    }
    pub fn unlock(&mut self, instance_num: u32) {
        let instance = self.get_instance_by_instance_num(instance_num).unwrap();
        instance.unlock();
        self.locked_instances
            .retain(|locked_instance| locked_instance.instance_num != instance.instance_num);
    }
    pub fn get_instance_by_instance_num(&self, instance_num: u32) -> Option<Arc<Instance>> {
        self.instances
            .iter()
            .find(|instance| instance.instance_num == instance_num)
            .map(Arc::clone)
    }
    pub fn get_unlocked_idle_instances(&self) -> Vec<Arc<Instance>> {
        self.instances
            .iter()
            .filter(|instance| {
                instance.state.load(SeqCst) == InstanceState::Idle
                    && instance.locked.load(SeqCst) == false
            })
            .map(Arc::clone)
            .collect()
    }
    pub fn get_playing_instance(&self) -> Option<Arc<Instance>> {
        self.instances
            .iter()
            .find(|instance| instance.state.load(SeqCst) == InstanceState::Playing)
            .map(Arc::clone)
    }
    pub fn get_first_idle_locked_instance(&self) -> Option<Arc<Instance>> {
        self.locked_instances
            .iter()
            .find(|instance| instance.state.load(SeqCst) == InstanceState::Idle)
            .map(Arc::clone)
    }
}

fn get_instance_num(process_id: u32) -> u32 {
    let instance_number_regex = Regex::new("RSG (.*?)/").unwrap();

    let command = format!(
        r#"$x= Get-WmiObject Win32_Process -Filter "ProcessId = {}"; $x.CommandLine"#,
        process_id
    );
    println!("Command: {}", command);
    match powershell_script::run(command.as_str()) {
        Ok(command_line) => {
            match instance_number_regex.captures(command_line.to_string().as_str()) {
                Some(matched_numbers) =>{
                    matched_numbers.get(1).unwrap().as_str().parse::<u32>().unwrap()
                },
                None => {
                    panic!("Could not find instance number in command line: {}", command_line)
                }
            }
        }
        Err(e) => {
            panic!("Error: {}", e)
        }
    }
}

pub struct WallQueue {
    queue: Vec<Arc<Instance>>,
    bag_size: usize,
}

impl WallQueue {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            bag_size: 4,
        }
    }
    pub fn pop(&mut self) -> Vec<Arc<Instance>> {
        self.queue.drain(0..self.bag_size).collect()
    }
    pub fn can_pop(&self) -> bool {
        self.queue.len() >= self.bag_size
    }

    pub fn push(&mut self, instance: Arc<Instance>) {
        self.queue.push(instance);
    }
    pub fn clear(&mut self) {
        self.queue.clear();
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn remove_by_instance_num(&mut self, instance_num: u32) {
        self.queue
            .retain(|instance| instance.instance_num != instance_num);
    }
}

#[derive(Serialize, Deserialize)]
pub struct WallFileInstance {
    pub instance_num: u32,
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
    playing: bool,
    freeze: bool,
}

pub fn write_wall_queue_to_json_file(
    wall_queue: &WallQueue,
    all_instances: &Vec<Arc<Instance>>,
) -> Vec<WallFileInstance> {
    let mut file = File::create("wall_queue.json").unwrap();

    let mut index = 0;

    let screen_width = 1920;
    let screen_height = 1080;

    let instance_width = screen_width / wall_queue.bag_size;
    let instance_height = screen_height / wall_queue.bag_size;
    let bag_size = wall_queue.bag_size;
    let bags_horizontal = 2;
    let bags_vertical = 2;
    let bag_width = screen_width / bags_horizontal;
    let bag_height = screen_height / bags_vertical;
    let bag_cols = 2;
    // Create an empty vector to store the instances in
    let mut instances: Vec<WallFileInstance> = Vec::new();
    let mut already_written_instances = Vec::new();
    let in_play_mode = all_instances.iter().any(|instance| instance.state.load(SeqCst) == InstanceState::Playing);
    for instance in &wall_queue.queue {
        if index >= wall_queue.bag_size * wall_queue.bag_size {
            break;
        }

        // Calculate which bag we're in
        let bag_index = index / bag_size;
        let bag_x_pos = bag_index % bags_horizontal;
        let bag_y_pos = bag_index / bags_vertical;

        let bag_x = screen_width - bag_x_pos * bag_width - bag_width;
        let bag_y = screen_height - bag_y_pos * bag_height - bag_height;
        // Calculate which position in the bag we should be in
        let instance_x = bag_x + (index % bag_cols) * instance_width;
        let instance_y = bag_y + ((index - bag_index * bag_size) / bag_cols) * instance_height;

        let _row = index / wall_queue.bag_size;
        let _col = index % wall_queue.bag_size;
        let instance_json: WallFileInstance = WallFileInstance {
            instance_num: instance.instance_num,
            width: instance_width,
            height: instance_height,
            x: if in_play_mode {screen_width} else {instance_x},
            y: instance_y,
            playing: instance.state.load(SeqCst) == InstanceState::Playing,
            freeze: false,
        };

        // println!("Index {}, Bag {} is at ({},{}), instance is at ({},{}), instance is {}",index, bag_index, bag_x, bag_y, instance_x, instance_y, instance.instance_num);
        // println!(
        //     "Index {}, instance is at ({},{}), instance is {}",
        //     index, instance_json.x, instance_json.y, instance.instance_num
        // );

        instances.push(instance_json);
        already_written_instances.push(instance.clone());
        index += 1;
    }

    for instance in all_instances {
        if !already_written_instances
            .iter()
            .find(|inst| inst.instance_num == instance.instance_num)
            .is_some()
        {
            let instance_json: WallFileInstance = WallFileInstance {
                instance_num: instance.instance_num,
                width: 1,
                height: 1,
                x: screen_width,
                y: 0,
                playing: instance.state.load(SeqCst) == InstanceState::Playing,
                freeze: false,
            };
            instances.push(instance_json);
        }
        println!("Is in play mode: {}", in_play_mode);

    }

    // write the instances to a string
    let json_string = serde_json::to_string(&instances);
    match json_string {
        Ok(json_string) => {
            file.write_all(json_string.as_bytes()).unwrap();
        }
        Err(e) => {
            println!("error writing json to file: {}", e);
        }
    }
    println!("Instances len: {}", instances.len());
    instances
}
