use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use crate::runner::TaskRunner;
use crate::terminal::Terminal;

mod hexterm;
pub(crate) mod formatting;

pub type TaskId = String;
pub struct HexTerm {
    pub running: bool,
    runner: TaskRunner,
    terminal: Terminal,
    output_rx: Receiver<HashMap<TaskId, String>>,
    command: String,
}