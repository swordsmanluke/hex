use std::fs::File;
use std::io::Read;
use std::fmt;
use serde::Deserialize;
use std::ops::Deref;
use fmt::Formatter;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub widgets: Vec<Widget>,
    pub apps: Vec<App>,
    pub layout: Layout,
}

#[derive(Deserialize, Clone)]
pub struct App {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub command: String
}

#[derive(Deserialize, Clone)]
pub struct Widget {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub command: String,
    pub period: String,
}

#[derive(Deserialize, Clone)]
pub struct Layout {
    pub kind: String,
    pub layout_id: Option<String>,
    pub main: Option<bool>,
    pub children: Option<Vec<Layout>>,
    pub orientation: Option<String>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub task_id: Option<String>,
}

impl Layout {

    pub fn to_str(&self, depth: usize) -> Option<String> {
        let mut out = String::from(format!("{:indent$}{}", "", self.kind.clone(), indent=depth*2));
        match self.kind.deref() {
            "linearlayout" => { out += format!(" ({})\n", self.orientation.as_ref().unwrap_or(&String::from("unknown"))).as_ref() },
            "textview" => { out += format!(" ({})\n", self.task_id.as_ref().unwrap_or(&String::from(""))).as_ref() }
            "panel" => { out+= format!(" ({} children)\n", self.children.as_ref().unwrap_or(Vec::new().as_ref()).len()).as_ref() },
            _ => { out += "Unknown" }
        }

        for child in self.children.as_ref().unwrap_or(&Vec::new()) {
            out += child.to_str(depth + 1).unwrap_or(String::from("")).as_ref();
        }

        Some(out)
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let output_str = self.to_str(0);
        match output_str {
            Some(out) => write!(f, "{}", out),
            None => write!(f, "")
        }
    }
}

pub fn load_task_config() -> Option<Config> {
    let mut tasks_file = File::open("config/tasks.toml").unwrap();
    let mut toml_tasks = String::new();
    tasks_file.read_to_string(&mut toml_tasks).unwrap();
    let config = toml::from_str(&toml_tasks);

    match config {
        Ok(conf) => {
            let conf = populate_layout_ids(conf)?;
            match how_many_mains(&conf.layout) {
                0 => { panic!("No 'main' layout! Mark one of your textviews as being 'main'"); },
                1 => { Some(conf) }, // perfect!
                _ => { panic!("More than one 'main' textview in tasks.toml!"); }
            }
        },
        Err(err) => {
            println!("conf err: {}", err);
            None
        }
    }
}

pub fn populate_layout_ids(conf: Config) -> Option<Config> {
    Some(conf)
}

pub fn how_many_mains(l: &Layout) -> usize {
    let main_children = match &l.children {
        Some(children) => { children.iter().map(|c| how_many_mains(c)).sum() },
        None => 0
    };

    let total_mains = if l.main.unwrap_or(false) { 1 } else { 0 } + main_children;

    if l.main.unwrap_or(false) && l.kind != "textview".to_string() { panic!("only textview's can be 'main'"); }

    total_mains
}