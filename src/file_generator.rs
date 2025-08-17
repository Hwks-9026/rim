use crate::map::Galaxy;
use crate::system::StarSystemData;

use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::fs;



pub(crate) fn load_file(path: &String) -> Option<Galaxy> {
    let data: Vec<u8>  = fs::read(&path).ok()?;
    serde_json::from_slice(&data).ok()
}

pub(crate) fn save(path: &String, save: Galaxy) {
    let data = serde_json::to_vec(&save).unwrap();
    fs::write(path, data);
}

pub(crate) fn generate_system_data() -> StarSystemData {
    loop {
        match try_generate_system() {
            None => {}
            Some(data) => {return data}
        }
    }
}

fn try_generate_system() -> Option<StarSystemData> {

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = StarSystemData::new();
        let _ = tx.send(result);
    });
    match rx.recv_timeout(Duration::from_micros(100)) {
        Ok(data) => return Some(data),
        Err(_) => {}
    } 
    return None;
}
