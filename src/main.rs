mod gameloop;
mod map;
mod utils;
mod system;
mod file_generator;

use std::fs::exists;
use std::env::{self, args};
use std::path::Path;

use crate::map::Galaxy;
fn main() {

    let args: Vec<String> = env::args().collect();
    
    if args.len() == 2 {
        if exists(Path::new(&args[1])).unwrap() == true {
            let galaxy = file_generator::load_file(&args[1]);
            let save = gameloop::start_gameloop(galaxy);
            file_generator::save(&args[1], save);
        }
        else {
            let save = gameloop::start_gameloop(None);
            file_generator::save(&args[1], save);
        }
    }
    else {
        let save = gameloop::start_gameloop(None);
        file_generator::save(&("default.rim".to_string()), save);
    }
    
}
