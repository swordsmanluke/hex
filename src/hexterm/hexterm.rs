use crate::hexterm::HexTerm;
use crate::tasks::Config;
use crate::runner::TaskRunner;
use std::sync::mpsc;
use portable_pty::{CommandBuilder, native_pty_system, PtySize};
use log::{trace, info, warn, error};
use std::collections::HashMap;
use crate::terminal::Terminal;

impl HexTerm {
    pub fn new(config: Config) -> HexTerm {
        // Create channel for widgets/apps to send output back to Hex
        let (output_tx, output_rx) = mpsc::channel();
        let tasks = config.tasks;
        let layout = config.layout;

        let runner = TaskRunner::new(tasks, output_tx);
        let terminal = Terminal::new(&layout);

        return HexTerm { runner, terminal, output_rx }
    }

    pub fn run(&mut self) {
        // TODO: Use crossterm or something similar to manage the display of Widgets.
        // TODO: InteractiveRunners take a child and a master. When active, Input is directed to their master.
        // TODO: TaskRunners just run on their schedule and send data back.
        // TODO: InteractiveRunners direct their output to their Widget on update.

        self.runner.start();

        loop {
            match self.output_rx.recv() {
                Ok(out) => { self.terminal.update(out); },
                Err(e) => { error!("Error receiving from task runner: {}", e); }
            }
        }
    }

    fn start_pty(&mut self, command: &str) {
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
        }).unwrap();

        let cmd = CommandBuilder::new(command);
        let child = pair.slave.spawn_command(cmd).unwrap();

        let mut reader = pair.master.try_clone_reader().unwrap();

        // Send data to the pty by writing to the master
        writeln!(pair.master, "ls -l\r\n").unwrap();

    }

}