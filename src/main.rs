use std::{
    fs::{self, File},
    io::Write,
    thread,
};

use instancemanager::WallFileInstance;
use tokio::{select, sync::mpsc::channel};

use crate::{
    hwndutils::{activate_hwnd, get_wall_hwnd, is_full_screen},
    instancemanager::{set_affinity, write_wall_queue_to_json_file},
};

mod hotkeys;
mod hwndutils;
mod instance;
mod instancemanager;
mod keyboardutils;
use livesplit_hotkey::{Hook, Hotkey};
#[tokio::main]
async fn main() {
    let mut preview_becomes_ready_channel = channel(1);
    let mut hotkeys_channel = channel(100);
    let mut wall_instances: Vec<WallFileInstance> = Vec::new();
    let mut instance_manager =
        instancemanager::InstanceManager::initialize(preview_becomes_ready_channel.0);

        hotkeys::setup_listeners(hotkeys_channel.0);
    loop {
        select! {
        // event = GlobalHotKeyEvent::receiver().recv() => {
        //     // println!("{:?}", event);
        // },
        instance_num = preview_becomes_ready_channel.1.recv() =>{
                let instance_arc = instance_manager.get_instance_by_instance_num(instance_num.unwrap());
                match instance_arc {
                    Some(instance_arc) => {
                        instance_manager.preview_unlocked_wall_queue.push(instance_arc.clone());

                    }
                    None => {
                        panic!("Received a preview_becomes_ready_channel message for an instance that doesn't exist");
                    }
                }
            },
            Some(hotkey) = hotkeys_channel.1.recv() => {
                    println!("Received hotkey: {}", hotkey);
                    match hotkey.as_str() {
                        "reset_bag" => {
                            println!("Resetting all instances");
                            if instance_manager.preview_unlocked_wall_queue.can_pop() {
                                instance_manager.reset_wall_bag();
                            }
                            if !instance_manager.preview_unlocked_wall_queue.can_pop() {
                                match instance_manager.get_first_idle_locked_instance(){
                                    Some(instance_arc) => {
                                        instance_manager.locked_instances.retain(|instance| instance.instance_num != instance_arc.instance_num);
                                        set_affinity(instance_arc.hwnd, 30);
                                        let mut file = File::create(r#"C:\Users\Boyen\sleepbg.lock"#).unwrap();
                                        file.write_all(b"").unwrap();
                                        instance_arc.play();
                                    }
                                    None => {
                                        println!("No idle instances to play");
                                    }
                                }
                            }

                        },
                        "exit_instance"=>{
                            // Exit the currently playing instance
                            match instance_manager.get_playing_instance(){
                                Some(instance_arc) => {
                                    fs::remove_file(r#"C:\Users\Boyen\sleepbg.lock"#).unwrap();
                                    let clone = instance_arc.clone();
                                    clone.state.store(instance::InstanceState::Idle,std::sync::atomic::Ordering::SeqCst);
                                    wall_instances = write_wall_queue_to_json_file(
                                        &instance_manager.preview_unlocked_wall_queue,
                                        &instance_manager.instances,
                                    );
                                    println!("Exiting instance: {}", clone.instance_num);
                                    clone.exit();
                                    loop{
                                        if !is_full_screen(clone.hwnd) {
                                            break;
                                        }
                                        thread::sleep(std::time::Duration::from_millis(5));

                                    }
                                    activate_hwnd(get_wall_hwnd());
                                    instance_manager.reset_instance(instance_arc.clone());

                                }
                                None => {
                                    println!("No playing instances to exit");
                                }
                            }
                        }
                        "toggle_thin"=>{
                            // Thin macro
                            match instance_manager.get_playing_instance(){
                                Some(instance_arc) => {
                                    let clone = instance_arc.clone();
                                    clone.thin();
                                },
                                None => {
                                    println!("No playing instances to make thin");
                                }
                            }
                        },
                        _ => {}
                }
            },
        }
    }
    println!("Updating affinities");
    instance_manager.update_affinities();
    println!(
        "preview_unlocked_wall_queue: {}",
        instance_manager.preview_unlocked_wall_queue.len()
    );
    wall_instances = write_wall_queue_to_json_file(
        &instance_manager.preview_unlocked_wall_queue,
        &instance_manager.instances,
    );
}
