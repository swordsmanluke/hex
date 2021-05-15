use crate::tasks::Layout;
use crate::views::{View, Widget, DimConstraint, Orientation, LinearLayout, ViewId, CharDims, TermLocation};
use std::collections::HashMap;
use log::{trace, info};

extern crate termion;

use std::io::{Write, stdout, Stdout};
use self::termion::raw::{IntoRawMode, RawTerminal};
use self::termion::{style, terminal_size};
use crate::hexterm::formatting::{Vt100Formatter, TextFormatter};
use crate::hexterm::TaskId;

pub type WindowMap = HashMap<TaskId, ViewId>;
type TaskStore = HashMap<TaskId, String>;

pub struct Terminal {
    pub windows: WindowMap,
    formatter: Box<dyn TextFormatter>,
    root: Box<dyn View>,
    tasks: TaskStore,
    stdout: RawTerminal<Stdout>
}

impl Terminal {

    pub fn new(layout: &Layout) -> Terminal {
        let mut windows = WindowMap::new();
        let tasks = TaskStore::new();
        let root = construct_layout(layout, &mut windows, TermLocation::new(1, 1));
        let stdout = stdout().into_raw_mode().unwrap();
        let formatter = Box::new(Vt100Formatter {});

        Terminal {  windows, tasks, root, stdout, formatter }
    }

    // TODO: Alter the task ID assigned to a View. This is altering
    //       the Layout... might need to keep it after all?
    //  pub fn assign_task_to_view(task_id, view)

    /***
     * Print the current display to the screen
     *
     * TODO: The rendering here will differ a bit for Interactive processes.
     *        I guess we'll need to know if we're interactive or not in here.
     ***/
    pub fn update(&mut self, output: HashMap<String, String>) {
        self.store_output(output);
        self.update_screen();
    }

    fn store_output(&mut self, output: HashMap<String, String>) {
        output.iter().for_each(|(task_id, text)| {
            // Store the output for later swapping into/out of a Window
            self.tasks.insert(task_id.clone(), text.clone());

            // Check - if a Window is displaying this task, update its associated View.
            match self.windows.get(task_id) {
                None => {},
                Some(view_id) => {
                    // If there's a View with this ID, set its contents to this value.
                    match self.tasks.get(task_id) {
                        None => {},
                        Some(task_text) => {
                            set_view_content(view_id, &mut self.root, &task_text, &self.formatter);
                        }
                    }
                }
            }
        });
    }

    fn update_screen(&mut self) {
        let (width, height) = terminal_size().unwrap();
        self.root.inflate(&CharDims::new(width as usize, height as usize), &TermLocation::new(1, 1));
        writeln!(self.stdout, "{}", self.root.render()).unwrap();
        self.root.wash();

        // and reset our style back to standard. JIC.
        writeln!(self.stdout, "{}", style::Reset).unwrap();

        self.stdout.flush().unwrap();
    }
}

fn set_view_content<'a>(id: &ViewId, view: &'a mut Box<dyn View>, text: &String, formatter: &Box<dyn TextFormatter>) -> bool {
    if view.id().eq(id) {
        view.update_content(text.clone());
        return true;
    }

    view.children().any(|c|
        set_view_content(id, c, text, formatter)
    )
}

/***
 * Converts Layout to View
 * Pass in a Layout description at the top and it'll build the concrete View objects.
 */
pub fn construct_layout(layout: &Layout, windows: &mut WindowMap, location: TermLocation) -> Box<dyn View> {
    info!("Building {}:{}", layout.kind, layout.task_id.clone().unwrap_or("".to_string()));

    let constructed: Box<dyn View> = match layout.kind.as_ref() {
        "linearlayout" => build_linear_layout(&layout, windows, location),
        "textview" => build_text_view(&layout, windows, location),
        _ => panic!("Unknown layout {}", layout.kind)
    };

    return constructed;
}

fn build_text_view(layout: &Layout, windows: &mut WindowMap, location: TermLocation) -> Box<dyn View> {
    let h_const = match layout.height {
        Some(h) => DimConstraint::Fixed(h),
        None => DimConstraint::WrapContent
    };
    let w_const = match layout.width {
        Some(w) => DimConstraint::Fixed(w),
        None => DimConstraint::WrapContent
    };

    let task_id = layout.task_id.clone().unwrap_or(String::from("unknown"));
    trace!("Creating text view for {}", task_id);
    let tv = Widget::new(w_const, h_const, Box::new(Vt100Formatter{}), location);
    windows.insert(task_id.clone(), tv.id());

    Box::new(tv)
}

fn build_linear_layout(layout: &Layout, windows: &mut WindowMap, location: TermLocation) -> Box<dyn View> {
    let orientation = match layout.orientation.as_ref().unwrap().as_ref() {
        "vertical" => Orientation::VERTICAL,
        _ => Orientation::HORIZONTAL
    };

    let h_const = match layout.height {
        Some(h) => DimConstraint::Fixed(h),
        None => DimConstraint::WrapContent
    };

    let w_const = match layout.width {
        Some(w) => DimConstraint::Fixed(w),
        None => DimConstraint::WrapContent
    };

    let mut ll: LinearLayout = LinearLayout::new(orientation, w_const, h_const, location);

    let mut next_child_loc = location;
    for child in layout.children.as_ref().unwrap_or(&Vec::new()) {
        let child= construct_layout(&child, windows, next_child_loc);
        next_child_loc = match orientation {
            Orientation::HORIZONTAL => { TermLocation::new(next_child_loc.x + child.width() as u16, next_child_loc.y) }
            Orientation::VERTICAL => { TermLocation::new(next_child_loc.x, next_child_loc.y + child.height() as u16) }
        };
        ll.add_child(child);
    }

    Box::new(ll)
}
