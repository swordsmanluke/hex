use crate::hexterm::HexTerm;
use crate::tasks::Config;
use crate::runner::WidgetUpdater;
use std::sync::mpsc;
// use portable_pty::{CommandBuilder, native_pty_system, PtySize};
use log::{error};
use crate::terminal::Terminal;
use std::io::{stdin, stdout, Write};
use termion::input::TermRead;
use std::sync::mpsc::Receiver;
use termion::event::Key;
use std::thread;
use termion::{terminal_size, clear};

impl HexTerm {
    pub fn new(config: Config) -> HexTerm {
        // Create channel for widgets/apps to send output back to Hex
        let (output_tx, output_rx) = mpsc::channel();
        let widgets = config.widgets;
        let layout = config.layout;

        let widget_runner = WidgetUpdater::new(widgets, output_tx);
        let terminal = Terminal::new(&layout);
        let command = "".to_owned();

        return HexTerm { widget_runner, terminal, output_rx, command, running: false }
    }

    pub fn run(&mut self) {
        // TODO: InteractiveRunners take a child and a master. When active, Input is directed to their master.
        // TODO: InteractiveRunners direct their output to their Widget on update.

        let key_rx = self.run_input_loop();
        self.running = true;
        self.widget_runner.start();

        // Empty the screen!
        writeln!(stdout(), "{}{}", termion::cursor::Hide, clear::All).unwrap();

        while self.running {
            match self.output_rx.try_recv() {
                Ok(out) => { self.terminal.update(out); },
                Err(_) => {} // error!("Error receiving from task runner: {}", e); }
            }
            self.process_input(&key_rx);
            self.print_prompt();
            stdout().flush().unwrap();
        }

        writeln!(stdout(), "{}So long!{}", clear::All, termion::cursor::Show).unwrap();
    }

    fn print_prompt(&self) {
        let (_, bottom) = terminal_size().unwrap();
        writeln!(stdout(), "{}{}> {}_{}{}",
                 termion::cursor::Goto(1, bottom - 1),
                 termion::color::Green.fg_str(),
                 self.command,
                 termion::style::Reset,
                 termion::clear::AfterCursor).unwrap();
    }

    fn process_input(&mut self, key_rx: &Receiver<Key>) {
        match key_rx.try_recv() {
            Err(_) => {}
            Ok(key) => {
                // TODO: If in passthrough mode, forward input to child proc
                match key {
                    // TODO: Add more interesting key combo support
                    Key::Backspace => {
                        if self.command.len() > 0 {
                            self.command.truncate(self.command.len() - 1);
                        }
                    }
                    Key::Char(c) => {
                        if c == '\n' {
                            self.execute_command();
                        } else {
                            self.command.push(c);
                        }
                    }
                    Key::Ctrl(ctl_c) => {
                        match ctl_c {
                            'c' => { self.running = false; }
                            _   => {} // No other Ctrl codes
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn execute_command(&mut self) {
        let mut parts = self.command.trim().split_whitespace();
        let task_id = parts.next().unwrap().to_owned();
        self.widget_runner.run_command(task_id, parts.collect::<Vec<&str>>().join(" "));
        self.command.truncate(0);
    }

    pub fn run_input_loop(&mut self) -> Receiver<Key>{
        // Create our channel
        let (tx, rx) = mpsc::channel();

        // Kick off the input handler
        thread::spawn( move || {
            for k in stdin().keys() {
                match k {
                    Ok(key) => { tx.send(key).unwrap(); },
                    Err(e) => { error!("Error reading keys: {}", e); }
                }
            }
        });

        // ...and return the receiver for the main loop to deal with.
        rx
    }

    // fn start_pty(&mut self, command: &str) {
    //     // Launch a PTY session
    //     let pty_system = native_pty_system();
    //     let mut pair = pty_system.openpty(PtySize {
    //         rows: 24,
    //         cols: 80,
    //         // Not all systems support pixel_width, pixel_height,
    //         // but it is good practice to set it to something
    //         // that matches the size of the selected font.  That
    //         // is more complex than can be shown here in this
    //         // brief example though!
    //         pixel_width: 0,
    //         pixel_height: 0,
    //     }).unwrap();
    //
    //     let cmd = CommandBuilder::new(command);
    //     let child = pair.slave.spawn_command(cmd).unwrap();
    //
    //     let mut reader = pair.master.try_clone_reader().unwrap();
    //
    //     // Send data to the pty by writing to the master
    //     writeln!(pair.master, "ls -l\r\n").unwrap();
    // }

}