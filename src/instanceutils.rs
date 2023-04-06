use std::process::Command;

use regex::Regex;

pub fn get_instance_dir(pid: u32) -> String {
    let str:String = String::from_utf8_lossy( &Command::new("sh").arg("-c").arg(format!("pwdx {pid}")).output().unwrap().stdout.as_slice()).into();
    // Remove {pid} : from the start of the string
    str.replace(format!("{pid}: ", pid=pid).as_str(), "").trim().to_string()
}

pub fn get_instance_num(instance_dir:String) -> u32{
    let instance_number_regex = Regex::new("RSG (.*?)/").unwrap();
    let instance_number = instance_number_regex.captures(&instance_dir).unwrap().get(1).unwrap().as_str();
    instance_number.parse::<u32>().unwrap()
}