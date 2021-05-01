use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use crate::runner::TaskRunner;
use crate::tasks::Layout;

mod hexterm;

pub struct HexTerm {
    runner: TaskRunner,
    layout: Layout,
    output_rx: Receiver<HashMap<String, String>>,
}

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

