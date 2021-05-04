use std::cmp::min;
use regex::{Match, Regex};

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