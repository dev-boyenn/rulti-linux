use std::{rc::Rc, sync::Arc};

use livesplit_hotkey::{Hook, Hotkey, KeyCode};
use rdev::{listen, Event};
use tokio::{sync::mpsc::Sender, task::spawn_blocking};

use crate::hwndutils::{self, get_mouse_pos, get_wall_hwnd};

pub fn setup_listeners(key_pressed: Sender<String>) {
    let hook = Hook::new().unwrap();
    hook.register(Hotkey::from(KeyCode::F13), {
        let key_pressed = key_pressed.clone();

        move || {
            if hwndutils::is_active(get_wall_hwnd()) {
                key_pressed.blocking_send("reset_bag".into()).unwrap()
            }
        }
    })
    .unwrap();
    hook.register(Hotkey::from(KeyCode::Backquote), {
        let key_pressed = key_pressed.clone();
        move || key_pressed.blocking_send("toggle_thin".into()).unwrap()
    })
    .unwrap();
    hook.register(Hotkey::from(KeyCode::KeyU), {
        let key_pressed = key_pressed.clone();
        move || key_pressed.blocking_send("exit_instance".into()).unwrap()
    })
    .unwrap();
}
