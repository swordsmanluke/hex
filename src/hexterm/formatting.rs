use std::cmp::min;
use regex::{Match, Regex, Captures};
use termion::cursor::Goto;
use log::info;
use crate::views::CharDims;

/***
TextFormatter: A trait for classes that convert from a raw string into a formatted one.
    Generic in order to allow different Terminal backends to use their own custom
    String-variants.
 */
pub trait TextFormatter {
    fn format(&self, s: &str, dims: (usize, usize), location: (u16, u16)) -> String;
}
pub struct DumbFormatter{}

impl TextFormatter for DumbFormatter {
    fn format(&self, s: &str, dims: (usize, usize), _: (u16, u16)) -> String {
        let last_len = min(dims.1, s.len());
        s[0..last_len].to_string()
    }
}

pub struct Vt100Formatter{}

fn find_vt100s(s: &str) -> Vec<Match> {
    let vt100_regex = Regex::new(r"((\u001b\[|\u009b)[\u0030-\u003f]*[\u0020-\u002f]*[\u0040-\u007e])+").unwrap();
    vt100_regex.find_iter(s).collect()
}

impl TextFormatter for Vt100Formatter {
    fn format(&self, s: &str, dims: (usize, usize), location: (u16, u16)) -> String {
        let mut final_text = "".to_string();

        for (i, line) in s.split("\n").take(dims.1).enumerate() {
            let (_, sliced) = Vt100Formatter::esc_aware_slice(line, dims.0);
            final_text.push_str(format!("{}{:width$}", Goto(location.0, location.1 + i as u16), sliced, width = dims.0).as_str());
        };

        return handle_clear(&final_text.as_str(), location, dims)
    }
}

impl Vt100Formatter {
    fn esc_aware_slice(s: &str, n: usize) -> (usize, String) {
        if n >= s.len() { return (s.len(), format!("{:width$}", s, width = s.len())) }

        let vt100s = find_vt100s(s);
        if vt100s.last().is_none() { return (n, s[0..n].to_string()); }
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

        let slice_end = min(s.len(), end);
        let sliced_str = s[0..slice_end].to_string();

        (end, sliced_str)
    }
}

fn handle_clear(s: &str, location: (u16, u16), dims: CharDims) -> String {
    // TODO: Move constants somewhere that makes sense
    let CSI = r"\u{1B}\[";
    let cls = Regex::new((CSI.to_string()+"2J").as_str()).unwrap();

    cls.replace(s, clear_screen(s, location, dims).as_str() ).into()
}

fn clear_screen(s: &str, location: (u16, u16), dims: CharDims) -> String{
    let mut clear = String::new();
    for y in 0..dims.1 {
        // Go to each line in the window, then print a bunch of spaces to clear the region.
        clear.push_str(format!("{}{:width$}", Goto(location.0, location.1 + y as u16), " ", width=dims.0).as_str());
    }

    clear
}

#[cfg(test)]
mod tests {
    use super::*;
    const VT100_TEST: &str = "T\u{1B}[33mE\u{1B}[96mS\u{1B}[39mT\u{1B}[39m"; // "TEST" interspersed with color codes for VT100 terminals

    #[test]
    fn slicing_vt100_string_works() {
        let fmt = Vt100Formatter{};
        let fmt_str = fmt.format(VT100_TEST, (2, 1), (1, 1));
        assert_eq!("\u{1b}[1;1HT\u{1B}[33mE", fmt_str);
    }

    #[test]
    fn clearing_screen_clears_section() {
        let mstr = handle_clear("\u{1B}[2JThis is new\nmultiline text", (1, 1), (2, 2));
        assert_eq!(mstr, "\u{1b}[1;1H  \u{1b}[2;1H  This is new\nmultiline text")
    }

    #[test]
    fn format_with_cls_works() {
        let fmt = Vt100Formatter{};
        let fmt_str = fmt.format("\u{1B}[2JThis is new\nmultiline text", (2, 2), (1, 1));
        assert_eq!(fmt_str, "\u{1b}[1;1H\u{1b}[1;1H  \u{1b}[2;1H  Th\u{1b}[2;1Hmu")
    }
}