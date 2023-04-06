use std::alloc::System;
use std::fmt::format;
use std::mem::zeroed;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use rdev::Key;
use regex::Regex;
use x11rb::connection::Connection;
use x11rb::errors::ReplyOrIdError;
use x11rb::protocol::xproto::{self, *};
use x11rb::protocol::Event;

use crate::instanceutils::{get_instance_dir, get_instance_num};

pub struct InstanceInfo {
    pub window: Window,
    pub pid: u32,
    pub gamedir: String,
    pub instance_num: u32,
}
pub fn set_window_title(conn: &impl Connection, win: u32, title: &str) -> Result<(), ReplyOrIdError> {
    let atom = conn.intern_atom(true, b"WM_NAME")?.reply()?.atom;
    conn.change_property(PropMode::APPEND, win, atom, AtomEnum::STRING, 8, title.len() as u32, title.as_bytes())?;
    println!("Set window title to {} ( but not actually )", title);
    Ok(())
}
fn activate_window(conn: &impl Connection, win: u32) -> Result<(), x11rb::errors::ReplyError> {
    let active_window = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
        .reply()?
        .atom;
    let client_message_data = ClientMessageData::from([1, 0, 0, 0, 0]);

    let evt = ClientMessageEvent {
        response_type: 33, // ClientMessage event
        format: 32,
        sequence: 0,
        window: win,
        type_: active_window,
        data: client_message_data,
    };

    conn.send_event(
        true,
        win,
        EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
        evt,
    )?;
    Ok(())
}
fn send_key(
    conn: &impl Connection,
    code: xproto::Keycode,
    state: bool,
    win: u32,
    timestamp: xproto::Timestamp,
) {
    let evt = KeyPressEvent {
        sequence: 0,
        detail: code,
        time: timestamp,
        root: win,
        event: win,
        child: win,
        root_x: 0,
        root_y: 0,
        event_x: 0,
        event_y: 0,
        same_screen: true,
        response_type: KEY_PRESS_EVENT,
        state: unsafe { zeroed() },
    };

    conn.send_event(
        true,
        win,
        EventMask::KEY_PRESS | EventMask::KEY_RELEASE,
        evt,
    )
    .unwrap();
}

/*
func (c *Client) GrabKey(key Key, win xproto.Window) error {
	return xproto.GrabKeyChecked(
		c.conn,
		true,
		win,
		uint16(key.Mod),
		key.Code,
		xproto.GrabModeAsync,
		xproto.GrabModeAsync,
	).Check()
} */

pub fn grab_key(conn: &impl Connection, key: Keycode, win: u32) -> Result<(), ReplyOrIdError> {
    xproto::grab_key(
        conn,
        true,
        win,
        ModMask::CONTROL,
        key,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?.check()?;
    Ok(())
}

pub fn calculate_offset(
    conn: &impl Connection,
    root: Window,
) -> Result<xproto::Timestamp, ReplyOrIdError> {
    let root = &conn.setup().roots[0].root;
    xproto::change_window_attributes(
        conn,
        *root,
        &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
    )?;
    // Get the current system time as u32

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32;
    let atom = conn.intern_atom(true, b"WM_NAME")?.reply()?.atom;
    println!("current time: {now}");
    conn.change_property(PropMode::APPEND, *root, atom, AtomEnum::STRING, 8, 0, &[])?;
    println!("change property fired");
    // Get the PropertyNotifyEvent
    let event = conn.wait_for_event()?;
    println!("wait_for_event fired");
    // Get the timestamp from the PropertyNotifyEvent
    let timestamp = match event {
        Event::PropertyNotify(event) => event.time,
        _ => panic!("Unexpected event: {:?}", event),
    };
    println!(
        "now: {}, timestamp: {}, subtracted : {}",
        now,
        timestamp,
        now - timestamp
    );
    return Ok(now - timestamp);
}

pub fn print_all_children_names(
    conn: &impl Connection,
    window: Window,
    root: Window,
) -> Result<(), ReplyOrIdError> {
    let tree = conn.query_tree(window)?.reply()?;
    for child in tree.children {
        let name = conn
            .get_property(false, child, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 100)?
            .reply()?;
        let name = String::from_utf8_lossy(&name.value);
        println!("Window: {} ", &name);
        if (name.contains("Minecraft*")) {
            println!("Found Minecraft");
            activate_window(conn, child)?;
            send_key(
                conn,
                72,
                false,
                child,
                (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u32)
                    - (calculate_offset(conn, root)? as u32),
            );
        }
        print_all_children_names(conn, child, root)?;
    }
    Ok(())
}

pub fn find_windows_matching_name(
    conn: &impl Connection,
    window: Window,
    name: &str,
    windows: &mut Vec<Window>,
) -> Result<(), ReplyOrIdError> {
    let tree = conn.query_tree(window)?.reply()?;
    for child in tree.children {
        let window_name = conn
            .get_property(false, child, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 100)?
            .reply()?;
        let window_name = String::from_utf8_lossy(&window_name.value);
        if window_name.contains(name) {
            windows.push(child);
        }
        find_windows_matching_name(conn, child, name, windows)?;
    }
    Ok(())
}
pub fn find_instances(
    conn: &impl Connection,
    window: Window,
) -> Result<Vec<InstanceInfo>, ReplyOrIdError> {
    let mut windows = vec![];
    find_windows_matching_name(conn, window, "Minecraft", &mut windows)?;
    Ok(windows
        .iter()
        .map(|w| {
            let pid = get_window_pid(conn, *w).unwrap();
            let gamedir = get_instance_dir(pid);
            InstanceInfo {
                window: *w,
                pid,
                gamedir: get_instance_dir(pid),
                instance_num: get_instance_num(gamedir)
            }
        })
        .collect())
}


pub fn get_window_pid(conn: &impl Connection, window: Window) -> Result<u32, ReplyOrIdError> {
    let pid = get_property_u32(conn, window, "_NET_WM_PID", AtomEnum::CARDINAL)?;
    Ok(pid)
}

pub fn get_property_u32(
    conn: &impl Connection,
    window: Window,
    name: &str,
    atom_enum: AtomEnum,
) -> Result<u32, ReplyOrIdError> {
    let result = get_property(conn, window, name, atom_enum)?;
    let result = u32::from_le_bytes(result[0..4].try_into().unwrap());
    Ok(result)
}
pub fn get_property(
    conn: &impl Connection,
    window: Window,
    name: &str,
    atom_enum: AtomEnum,
) -> Result<Vec<u8>, ReplyOrIdError> {
    let atom = conn.intern_atom(false, name.as_bytes())?.reply()?.atom;

    Ok(conn
        .get_property(false, window, atom, atom_enum, 0, 1024)?
        .reply()?
        .value)
}
