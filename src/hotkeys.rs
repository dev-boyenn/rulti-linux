use tokio::sync::mpsc::Sender;
use tokio::task::spawn_blocking;

use crate::hwndutils::{self, get_wall_hwnd};
use windows_hotkeys::keys::VKey;
use windows_hotkeys::HotkeyManager;
pub async fn setup_listeners(key_pressed: Sender<String>) {
    let returnvalue = spawn_blocking(move || {
        let mut hkm = HotkeyManager::new();

        hkm.register(VKey::Capital, &[], {
            let key_pressed = key_pressed.clone();

            move || {
                print!("F13 pressed");
                if hwndutils::is_active(get_wall_hwnd()) {
                    key_pressed.blocking_send("reset_bag".into()).unwrap()
                }
            }
        })
        .unwrap();

        hkm.register(VKey::Oem3, &[], {
            let key_pressed = key_pressed.clone();
            move || {
                if hwndutils::is_active(get_wall_hwnd()) {
                    key_pressed.blocking_send("lock".into()).unwrap()
                } else {
                    key_pressed.blocking_send("toggle_thin".into()).unwrap()
                }
            }
        })
        .unwrap();
        hkm.register(VKey::U, &[], {
            let key_pressed = key_pressed.clone();
            move || {
                print!("U pressed");
                key_pressed.blocking_send("exit_instance".into()).unwrap()
            }
        })
        .unwrap();
        println!("Registered hotkeys");
        hkm.event_loop()
    })
    .await;
    match returnvalue {
        Ok(_) => (),
        Err(_) => panic!("Error in hotkey listener"),
    }
}
