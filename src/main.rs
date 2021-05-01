extern crate regex;
extern crate simplelog;

use std::fs::File;

use simplelog::*;

use crate::hexterm::HexTerm;

mod views;
mod tasks;
mod executable_command;
mod runner;
mod hexterm;

pub type TaskId = String;

fn main() {
    init_logging();
    let config = tasks::load_task_config().unwrap();
    HexTerm::new(config).run();
}

fn init_logging() {
    CombinedLogger::init(
        vec![
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("log/hex.log").unwrap()),
        ]
    ).unwrap();
}


