use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::Arc;
use crate::views::{InteractiveWidget};
use std::{thread, io};
use regex::Regex;
use crate::views::input_processor::InputProcessor;
use std::io::{stdout, Stdout};

pub struct Window {
    stdin_rx: Receiver<String>,
    view: Option<Arc<InteractiveWidget>>,
    input_proc: InputProcessor
}

impl Window {
    pub fn new(stdin_rx: Receiver<String>) -> Window {
        Window { stdin_rx, input_proc: InputProcessor::new(), view: None }
    }

    pub fn assign_view(&mut self, view: Arc<InteractiveWidget>) {
        self.view = Some(view.clone());
        self.input_proc = InputProcessor::new()
    }

    pub fn run(&mut self) {
        loop {
            match self.stdin_rx.try_recv() {
                Ok(input) => {
                    match self.view.as_ref() {
                        None => {},
                        Some(v) => {
                            self.input_proc.push(input, &v.location, &v.dims.size);
                        }
                    }
                }
                Err(_) => {}
            }

            self.input_proc.print(&mut io::stdout());
        }
    }

    fn print(&self, input: String) {
        match self.my_view() {
            Some(v) => v.print(input),
            None => {}
        }
    }

    fn my_view(&self) -> Option<Arc<InteractiveWidget>> {
        match self.view.clone() {
            Some(v) => Some(v.clone()),
            None => None
        }
    }
}

