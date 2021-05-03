use crate::tasks::Layout;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::views::{View, TextView, Dim, Orientation, LinearLayout, ViewId};
use std::collections::HashMap;
use std::thread;
use std::sync::mpsc;
use log::{trace, info, error};

extern crate termion;

use termion::{clear, color, cursor};
use std::io::{Write, stdout, Stdout, Stdin, stdin};
use self::termion::raw::{IntoRawMode, RawTerminal};
use self::termion::{style, terminal_size};
use std::sync::mpsc::{Sender, Receiver};
use self::termion::event::Key;
use self::termion::input::TermRead;
use crate::hexterm::formatting::{TaskText, Vt100Formatter};
use std::time::Duration;
use crate::hexterm::TaskId;

pub type RcView = Rc<RefCell<dyn View>>;
pub type WindowMap = HashMap<TaskId, ViewId>;
type TaskStore = HashMap<TaskId, TaskText>;

pub struct Terminal {
    pub windows: WindowMap,
    root: Box<dyn View>,
    tasks: TaskStore,
    running: bool,
    stdout: RawTerminal<Stdout>
}

impl Terminal {

    pub fn new(layout: &Layout) -> Terminal {
        // TODO: TaskText is referenced mutably out here and needs to be injected into
        //       Windows.... Might be better to go back to just Strings. :/
        //       Except that TaskText has formatters.
        let mut windows = WindowMap::new();
        let mut tasks = TaskStore::new();
        let root = construct_layout(layout, &mut windows);
        let mut stdout = stdout().into_raw_mode().unwrap();

        Terminal {  windows, tasks, root, stdout, running: false }
    }

    // TODO: Alter the task ID assigned to a View. This is altering
    //       the Layout... might need to keep it after all?
    //  pub fn assign_task_to_view(task_id, view)

    pub fn update(&mut self, output: HashMap<String, String>) {
        // TODO: The rendering here will differ a bit for Interactive processes.
        //       I guess we'll need to know if we're interactive or not in here.
        output.iter().for_each(|(task_id, text)| {
            match self.tasks.get_mut(task_id) {
                None => {
                    let text = TaskText::new(text.clone(), Box::new(Vt100Formatter {}));
                    self.tasks.insert(task_id.clone(), text);
                },
                Some(taskContents) => {
                    taskContents.replace(text.clone());
                }
            }

            match self.windows.get(task_id) {
                None => {},
                Some(view_id) => {
                    // If there's a TextView with this ID, set its contents to this value.
                    match self.tasks.get(task_id) {
                        Some(task_text) => { set_view_content(view_id, &mut self.root, &task_text); },
                        None => {}
                    }
                }
            }
        } );

        self.update_screen();
    }

    fn update_screen(&mut self) {
        let (width, height) = terminal_size().unwrap();
        self.root.inflate(&(width as usize, height as usize));
        let output: String = self.root.render_lines().join("\n");
        info!("Screen output: {}", output);
        // TODO: Does clearing the screen and reprinting everything cause flicker?
        let rst_header = format!("{clear}{goto}",
                                 clear = clear::All,
                                 goto = cursor::Goto(1, 1));

        writeln!(self.stdout, "{}{}{}",
                 rst_header,
                 output,
                 style::Reset).unwrap();

        self.stdout.flush().unwrap();
    }
}

fn set_view_content<'a>(id: &ViewId, mut view: &'a mut Box<dyn View>, text: &TaskText) -> bool {
    if view.id().eq(id) {
        view.replace_content(text.format(view.width(), view.height()));
        return true;
    }

    view.children().any(|c|
        set_view_content(id, c, text)
    )
}

/***
 * Converts Layout to View
 * Pass in a Layout description at the top and it'll build the concrete View objects.
 */
pub fn construct_layout(layout: &Layout, windows: &mut WindowMap) -> Box<dyn View> {
    info!("Building {}:{}", layout.kind, layout.task_id.clone().unwrap_or("".to_string()));

    let constructed: Box<dyn View> = match layout.kind.as_ref() {
        "linearlayout" => build_linear_layout(&layout, windows),
        "textview" => build_text_view(&layout, windows),
        _ => panic!("Unknown layout {}", layout.kind)
    };

    return constructed;
}

fn build_text_view(layout: &Layout, windows: &mut WindowMap) -> Box<dyn View> {
    let h_const = match layout.height {
        Some(h) => Dim::Fixed(h),
        None => Dim::WrapContent
    };
    let w_const = match layout.width {
        Some(w) => Dim::Fixed(w),
        None => Dim::WrapContent
    };

    let task_id = layout.task_id.clone().unwrap_or(String::from("unknown"));
    trace!("Creating text view for {}", task_id);
    let tv = TextView::new(w_const, h_const);
    windows.insert(task_id.clone(), tv.id());

    Box::new(tv)
}

fn build_linear_layout(layout: &Layout, windows: &mut WindowMap) -> Box<dyn View> {
    let orientation = match layout.orientation.as_ref().unwrap().as_ref() {
        "vertical" => Orientation::VERTICAL,
        _ => Orientation::HORIZONTAL
    };

    let h_const = match layout.height {
        Some(h) => Dim::Fixed(h),
        None => Dim::WrapContent
    };

    let w_const = match layout.width {
        Some(w) => Dim::Fixed(w),
        None => Dim::WrapContent
    };

    let mut ll: LinearLayout = LinearLayout::new(orientation, w_const, h_const);

    for child in layout.children.as_ref().unwrap_or(&Vec::new()) {
        let child= construct_layout(&child, windows);
        ll.add_child(child);
    }

    Box::new(ll)
}
