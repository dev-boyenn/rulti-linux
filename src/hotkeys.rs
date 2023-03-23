use std::thread;

use rdev::{listen, Event};
use windows::Win32::{UI::WindowsAndMessaging::GetWindowTextW, Foundation::{HWND, LPARAM, BOOL}};


use crate::instance::Instance;

pub fn setup_listeners(){
    let mut instances:Vec<Instance> = Vec::new();
    let enum_window = extern "system" |hwnd: HWND, _: LPARAM| -> BOOL {
        unsafe {
            let mut text: [u16; 512] = [0; 512];
            let len = GetWindowTextW(hwnd, &mut text);
            let text = String::from_utf16_lossy(&text[..len as usize]);
            if text.contains("Minecraft") {
                println!("found minecraft window");
                instances.push(Instance::new(hwnd));
            }
           
            true.into()
        }
    }

    let callback =  move |event: Event| {
        match event.name {

            Some(string) => match string.as_str().to_lowercase().as_str(){
                "a" => {
                    
                    for instance in instances.iter() {
                        thread::spawn(move || {
                            instance.reset(); // Long running task
                        });
                    }
                    ()
                }
                _ => (),
            },
            None => (),
        }
    };
   
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error)
    }
   
    
}

// pub fn enumerate_windows<F>(mut callback: F)
//     where F: FnMut(HWND) -> bool
// {
//     let mut trait_obj: &mut dyn FnMut(HWND) -> bool = &mut callback;
//     let closure_pointer_pointer: *mut c_void = unsafe { mem::transmute(&mut trait_obj) };

//     let lparam = closure_pointer_pointer as LPARAM;
//     unsafe { EnumWindows(Some(enumerate_callback), lparam) };
// }

// unsafe extern "system" fn enumerate_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
//     let closure: &mut &mut dyn FnMut(HWND) -> bool = mem::transmute(lparam as *mut c_void);
//     if closure(hwnd) { TRUE } else { FALSE }
// }