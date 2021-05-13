use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use crate::runner::WidgetUpdater;
use crate::terminal::Terminal;

mod hexterm;
pub(crate) mod formatting;

pub type TaskId = String;
pub struct HexTerm {
    pub running: bool,
    widget_runner: WidgetUpdater,
    terminal: Terminal,
    output_rx: Receiver<HashMap<TaskId, String>>,
    command: String,
}