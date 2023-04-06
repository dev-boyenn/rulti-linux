use tokio::sync::mpsc::channel;
use x11::find_instances;
use x11rb::{connection::Connection, protocol::{xproto::{self, ChangeWindowAttributesAux, EventMask, Keycode}, Event}};

use crate::x11::grab_key;

mod instance;
// mod instancemanager;
// mod keyboardutils;
mod x11;
mod instanceutils;
mod instancemanager;
#[tokio::main]
async fn main() {
    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];
    let instances = find_instances(&conn, screen.root).unwrap();
    xproto::change_window_attributes(
        &conn,
        screen.root,
        &ChangeWindowAttributesAux::new().event_mask(EventMask::KEY_PRESS),
    ).unwrap();

    grab_key(&conn, 25, screen.root).unwrap();
    println!("Found {} instances", instances.len());

    // for instance in instances {
    //     // print instance pid and window
    //     println!("Instance: {} {} {} {}", instance.pid, instance.window, instance.gamedir, instance.instance_num);
    // }

    let mut preview_becomes_ready_channel = channel(100);
    let mut percent_sender = channel(100);
    // let mut hotkeys_channel = channel(100);
    // let mut wall_instances: Vec<WallFileInstance> = Vec::new();
    let mut instance_manager =
        instancemanager::InstanceManager::initialize(preview_becomes_ready_channel.0,percent_sender.0, instances, &conn);
    loop{
        if let Ok(event) = conn.poll_for_event(){
            if let Some(event) = event{
                match event {
                    Event::KeyPress(event) => println!("Key pressed: {}", event.detail),
                    e => {println!("Event: {:?}", e)},
                    // other => panic!("Unexpected event {:?}", other),
                };
            }
        }
    }
    // tokio::spawn(async move {
    //     hotkeys::setup_listeners(hotkeys_channel.0).await;
    // });
    // loop {
    //     select! {
    //         Some(percent) = percent_sender.1.recv() => {
    //             println!("Percent: {}", percent);
    //         },
    //         instance_num = preview_becomes_ready_channel.1.recv() =>{
    //             let instance_arc = instance_manager.get_instance_by_instance_num(instance_num.unwrap());
    //             match instance_arc {
    //                 Some(instance_arc) => {
    //                     instance_manager.preview_unlocked_wall_queue.push(instance_arc.clone());
    //                 }
    //                 None => {
    //                     panic!("Received a preview_becomes_ready_channel message for an instance that doesn't exist");
    //                 }
    //             }
    //         },
    //         Some(hotkey) = hotkeys_channel.1.recv() => {
    //                 println!("Received hotkey: {}", hotkey);
    //                 match hotkey.as_str() {
    //                     "lock" => {
    //                     let left_click = hwndutils::get_mouse_pos();
    //                     wall_instances.iter().for_each(|wall_instance|{

    //                    if (wall_instance.x < left_click.0 as usize && wall_instance.x + wall_instance.width > left_click.0 as usize) && (wall_instance.y < left_click.1 as usize && wall_instance.y + wall_instance.height > left_click.1 as usize){
    //                        println!("Found instance: {}", wall_instance.instance_num);
    //                        let instance_arc = instance_manager.get_instance_by_instance_num(wall_instance.instance_num);
    //                        match instance_arc {
    //                            Some(instance_arc) => {
    //                                instance_manager.preview_unlocked_wall_queue.remove_by_instance_num(instance_arc.instance_num);
    //                                instance_manager.lock(instance_arc.instance_num);
    //                            }
    //                            None => {
    //                                println!("Received a lock message for an instance that doesn't exist");
    //                            }
    //                        }
    //                    }
    //                });
    //                     },

    //                     "reset_bag" => {
    //                         println!("Resetting all instances");
    //                         if instance_manager.preview_unlocked_wall_queue.can_pop() {
    //                             instance_manager.reset_wall_bag();
    //                         }
    //                         if !instance_manager.preview_unlocked_wall_queue.can_pop() {
    //                             match instance_manager.get_first_idle_locked_instance(){
    //                                 Some(instance_arc) => {
    //                                     instance_manager.locked_instances.retain(|instance| instance.instance_num != instance_arc.instance_num);
    //                                     instance_arc.set_affinity(((1 << 28) - 1) << 4);

    //                                     let mut file = File::create(r#"C:\Users\Boyen\sleepbg.lock"#).unwrap();
    //                                     file.write_all(b"").unwrap();
    //                                     instance_arc.play();
    //                                 }
    //                                 None => {
    //                                     println!("No idle instances to play");
    //                                 }
    //                             }
    //                         }

    //                     },
    //                     "exit_instance"=>{
    //                         // Exit the currently playing instance
    //                         match instance_manager.get_playing_instance(){
    //                             Some(instance_arc) => {
    //                                 fs::remove_file(r#"C:\Users\Boyen\sleepbg.lock"#).unwrap();
    //                                 let clone = instance_arc.clone();
    //                                 clone.state.store(instance::InstanceState::Idle,std::sync::atomic::Ordering::SeqCst);
    //                                 wall_instances = write_wall_queue_to_json_file(
    //                                     &instance_manager.preview_unlocked_wall_queue,
    //                                     &instance_manager.instances,
    //                                 );
    //                                 println!("Exiting instance: {}", clone.instance_num);
    //                                 clone.exit();
    //                                 loop{
    //                                     if !is_full_screen(clone.hwnd) {
    //                                         break;
    //                                     }
    //                                     thread::sleep(std::time::Duration::from_millis(5));

    //                                 }
    //                                 activate_hwnd(get_wall_hwnd());
    //                                 instance_manager.reset_instance(instance_arc.clone());

    //                             }
    //                             None => {
    //                                 println!("No playing instances to exit");
    //                             }
    //                         }
    //                     }
    //                     "toggle_thin"=>{
    //                         // Thin macro
    //                         match instance_manager.get_playing_instance(){
    //                             Some(instance_arc) => {
    //                                 let clone = instance_arc.clone();
    //                                 clone.thin();
    //                             },
    //                             None => {
    //                                 println!("No playing instances to make thin");
    //                             }
    //                         }
    //                     },
    //                     _
    //         "preview_unlocked_wall_queue: {}",
    //         instance_manager.preview_unlocked_wall_queue.len()
    //     );
    //     wall_instances = write_wall_queue_to_json_file(
    //         &instance_manager.preview_unlocked_wall_queue,
    //         &instance_manager.instances,
    //     );
    // }
}

// #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
// async fn it_works() {
//     let mut preview_becomes_ready_channel = channel(100);
//     let mut percent_sender = channel(100);

//     let mut instance_manager = instancemanager::InstanceManager::initialize(
//         preview_becomes_ready_channel.0,
//         percent_sender.0,
//     );

//     for _ in 1..100 {
//         let start_time = std::time::Instant::now();
//         let mut successful_bag_resets: u32 = 0;

//         loop {
//             match preview_becomes_ready_channel.1.try_recv().map(|instance_num|{
//             let instance_arc = instance_manager.get_instance_by_instance_num(instance_num);
//             match instance_arc {
//                 Some(instance_arc) => {
//                     instance_manager.preview_unlocked_wall_queue.push(instance_arc.clone());
//                 }
//                 None => {
//                     panic!("Received a preview_becomes_ready_channel message for an instance that doesn't exist");
//                 }
//             }
//         }){
//             Ok(_) => {},
//             Err(_) => {}
//         }

//             if instance_manager.preview_unlocked_wall_queue.can_pop() {
//                 instance_manager.reset_wall_bag();
//                 successful_bag_resets += 1;
//             }
//             if (successful_bag_resets == 100) {
//                 break;
//             }
//             instance_manager.update_affinities();
//         }
//         let end_time = std::time::Instant::now();
//         println!("Time taken: {:?}", end_time.duration_since(start_time));
//     }
// }
