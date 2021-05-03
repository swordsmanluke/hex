use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use crate::runner::TaskRunner;
use crate::tasks::Layout;
use crate::terminal::Terminal;

mod hexterm;
pub(crate) mod formatting;

pub type TaskId = String;
pub struct HexTerm {

    runner: TaskRunner,
    terminal: Terminal,
    output_rx: Receiver<HashMap<TaskId, String>>,
}