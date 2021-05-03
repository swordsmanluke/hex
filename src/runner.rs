use std::collections::HashMap;
use std::process::{Command, Output};
use std::str;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use log::{info, trace, warn};

use crate::executable_command::ExecutableCommand;
use crate::tasks::Task;

pub struct TaskRunner {
    pub commands: Vec<ExecutableCommand>,
    system_command_sender: Sender<HashMap<String, String>>,
    running: bool,
}

impl TaskRunner {
    pub fn new(tasks: Vec<Task>,
               output_tx: Sender<HashMap<String, String>>) -> TaskRunner {
        let commands = tasks.iter().
            map(|t| task_to_command(t)).
            collect();

        TaskRunner { commands, system_command_sender: output_tx, running: true }
    }

    pub fn start(&mut self) {
        self.commands.iter().for_each(|cmd| self.start_task_loop(&cmd) );
    }

    pub fn run_command(&self, task_id: String, command: String) {
        match self.commands.iter().find(|cmd| cmd.id == task_id) {
            Some(cmd) => {
                let mut mutcmd = cmd.clone();
                let mut parts = command.split_whitespace();
                parts.next(); // pop the initial command off
                mutcmd.command += " ";
                mutcmd.command += parts.collect::<Vec<&str>>().join(" ").as_str();

                self.run_task_once(&mutcmd);
            }
            None => { warn!("Could not find command '{}'", task_id) }
        }
    }

    fn start_task_loop(&self, command: &ExecutableCommand) {
        let trx = self.system_command_sender.clone();
        let cmd = command.clone();
        info!("spawn {} thread", cmd.id);

        thread::Builder::new().name(cmd.id.clone()).spawn(move ||
            {
                loop {
                    let last_run = SystemTime::now();

                    let mut h = HashMap::new();
                    h.insert(cmd.id.clone(), convert_output(exec_command(cmd.command.clone(), cmd.working_dir.clone())));
                    trx.send(h).unwrap();

                    let nap_millis = cmd.millis_until_next_run(last_run.elapsed().unwrap().as_millis() as u64);
                    let naptime = Duration::from_millis(nap_millis);
                    info!("{} ran for {:.2?}", cmd.id, last_run.elapsed().unwrap());
                    trace!("{} sleeping for {}ms", cmd.id, nap_millis);
                    sleep(naptime);
                }
            }).unwrap();
    }

    fn run_task_once(&self, command: &ExecutableCommand) -> () {
        let trx = self.system_command_sender.clone();
        let cmd = command.clone();
        info!("Running manual '{}' command", cmd.id);

        let mut h = HashMap::new();
        h.insert(cmd.id.clone(), convert_output(exec_command(cmd.command.clone(), cmd.working_dir.clone())));
        trx.send(h).unwrap();
    }
}

fn convert_output(output: Output) -> String {
    let std_text = match str::from_utf8(&output.stdout) {
        Ok(t) => t.to_owned(),
        Err(_) => String::from("")
    };

    let err_text = match str::from_utf8(&output.stderr) {
        Ok(t) => t.to_owned(),
        Err(_) => String::from("")
    };

    // If we got an error, show that.
    match err_text.len() {
        0 => std_text,
        _ => err_text
    }
}

fn exec_command(command: String, working_dir: String) -> Output {
    let mut parts = command.trim().split_whitespace();
    let cmd = parts.next().unwrap();
    let args = parts;

    info!("Running {}/{} {}", working_dir, cmd, args.clone().map(|s| s.to_string()).collect::<Vec<String>>().join(" "));

    Command::new(vec!(working_dir.clone(), cmd.to_string()).join("/"))
        .current_dir(working_dir.clone())
        .args(args)
        .output()
        .expect("failed to execute process")
}

fn task_to_command(t: &Task) -> ExecutableCommand {
    ExecutableCommand::new(t.id.clone(),
                           t.command.clone(),
                           t.path.clone(),
                           t.period.clone())
}
