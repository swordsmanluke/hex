use std::cmp::min;
use regex::{Match, Regex};
use crate::views::Dimensions;
use log::info;

pub struct TaskText {
    raw_text: String,
    formatter: Box<dyn TextFormatter>
}

impl TaskText {
    pub fn new(raw_text: String, formatter: Box<dyn TextFormatter>) -> TaskText {
        TaskText { raw_text, formatter }
    }

    pub fn append(&mut self, more_text: String) {
        self.raw_text.push_str(&more_text);
    }

    pub fn replace(&mut self, new_text: String) {
        self.raw_text = new_text;
    }

    pub fn format(&self, width: usize, height: usize) -> String {
        info!("Formatting '{}' to {}x{}", self.raw_text, width, height);
        return self.raw_text.clone();
        // self.raw_text.split("\n").take(height). // First n Lines
        //     map(|c| self.formatter.format(c, width)). // Format them
        //     collect::<Vec<String>>().join("\n")     // Convert back into a single string
    }

    /***
    Return the maximum width of this string
    TODO: Use 'formatter' or something to account for escape sequences
     */
    pub fn width(&self) -> usize {
        self.raw_text.split("\n").map(|c| c.len()).max().unwrap()
    }

    pub fn height(&self) -> usize {
        self.raw_text.split("\n").count()
    }
}

// Conversions to/from strings
impl Into<String> for TaskText {
    fn into(self) -> String { self.raw_text.clone() }
}

impl From<String> for TaskText {
    fn from(text: String) -> Self {
        Self::new(text, Box::new(Vt100Formatter{}))
    }
}

/***
TextFormatter: A trait for classes that convert from a raw string into a formatted one.
    Generic in order to allow different Terminal backends to use their own custom
    String-variants.
 */
pub trait TextFormatter {
    fn format(&self, s: &str, max_len: usize) -> String;
}

pub struct DumbFormatter{}

impl TextFormatter for DumbFormatter {
    fn format(&self, s: &str, n: usize) -> String {
        let last_len = min(n, s.len());
        s[0..last_len].to_string()
    }
}

pub struct Vt100Formatter{}

fn find_vt100s(s: &str) -> Vec<Match> {
    let vt100_regex = Regex::new(r"((\u001b\[|\u009b)[\u0030-\u003f]*[\u0020-\u002f]*[\u0040-\u007e])+").unwrap();
    vt100_regex.find_iter(s).collect()
}

impl TextFormatter for Vt100Formatter {
    fn format(&self, s: &str, n: usize) -> String {
        if n >= s.len() { return format!("{:width$}", s, width = n) }

        let vt100s = find_vt100s(s);
        if vt100s.last().is_none() { return s[0..n].to_string(); }
        let mut captured_chars = 0;
        let mut end = 0;

        for c in vt100s.iter() {
            if (captured_chars + 1) < n {  // cc+1 to avoid subtraction with overflow
                let next_block_of_text_size = c.start() - end;
                let next_incr = if captured_chars + next_block_of_text_size >= n {
                    (captured_chars + next_block_of_text_size) - n
                } else {
                    next_block_of_text_size
                };

                captured_chars += next_incr;
                end = c.end();
            }
        };

        if captured_chars < n {
            end += n - captured_chars; // grab any remaining characters we need
        }

        // info!("Str {} chars\n----\n{}\n----", s.len(), s);
        // info!("Slice:\n-----\n{}\n------", s[0..end].to_string());

        let slice_end = min(s.len(), end);
        format!("{:width$}", s[0..slice_end].to_string(), width = end) // TODO: Width = max width? To erase previous text?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const VT100_TEST: &str = "T\u{1B}[33mE\u{1B}[96mS\u{1B}[39mT\u{1B}[39m"; // "TEST" interspersed with color codes for VT100 terminals

    #[test]
    fn slicing_vt100_string_works() {
        let fmt = Vt100Formatter{};
        let fmt_str = fmt.format(VT100_TEST, 2);
        assert_eq!("T\u{1B}[33mE", fmt_str);
    }
}