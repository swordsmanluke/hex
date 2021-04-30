mod widgets;

extern crate regex;
extern crate simplelog;

use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use simplelog::*;
use log::info;

use crate::runner::TaskRunner;
use crate::tasks::Layout;
use std::thread::JoinHandle;

use portable_pty::{CommandBuilder, PtySize, native_pty_system, PtySystem};
use anyhow::Error;

mod tasks;
mod executable_command;
mod runner;

pub type TaskId = String;

pub struct Channel<T> {
    pub tx: Sender<T>,
    pub rx: Receiver<T>
}

impl<T> Channel<T> {
    pub fn new(tx: Sender<T>, rx: Receiver<T>) -> Channel<T> {
        Channel { tx, rx }
    }

    pub fn from(tuple: (Sender<T>, Receiver<T>)) -> Channel<T> {
        Channel::new(tuple.0, tuple.1)
    }
}

fn main() {
    init_logging();

    let config = tasks::load_task_config().unwrap();
    let layout = config.layout;

    let system_command_channel = Channel::from(mpsc::channel());
    let task_running_channel = Channel::from(mpsc::channel());

    let mut runner = TaskRunner::new(config.tasks, system_command_channel.tx.clone(), task_running_channel.rx);

    thread::spawn( move || { runner.run(); });

    launch(layout,
           system_command_channel.rx,
           system_command_channel.tx,
           task_running_channel.tx.clone()).join().unwrap_or({});
}

fn launch(layout: Layout,
          command_receiver: Receiver<HashMap<String, String>>,
          command_sender: Sender<HashMap<String, String>>,
          task_sender: Sender<String>) -> JoinHandle<()> {
    thread::spawn(move || {
        // Launch a PTY session
        let pty_system = native_pty_system();
        let mut pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            // Not all systems support pixel_width, pixel_height,
            // but it is good practice to set it to something
            // that matches the size of the selected font.  That
            // is more complex than can be shown here in this
            // brief example though!
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = CommandBuilder::new("bash");
        let child = pair.slave.spawn_command(cmd)?;

        let mut reader = pair.master.try_clone_reader()?;

        // Send data to the pty by writing to the master
        writeln!(pair.master, "ls -l\r\n")?;

        // TODO: Use crossterm or something similar to manage the display of Widgets.
        // TODO: InteractiveRunners take a child and a master. When active, Input is directed to their master.
        // TODO: TaskRunners just run on their schedule and send data back.
        // TODO: InteractiveRunners direct their output to their Widget on update.

    })
}

fn init_logging() {
    CombinedLogger::init(
        vec![
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("log/flux.log").unwrap()),
        ]
    ).unwrap();
}


