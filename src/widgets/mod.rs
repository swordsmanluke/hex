use std::cmp::{Ordering, min};
use std::rc::Rc;
use std::cell::RefCell;
use regex::{Match, Regex};

mod linear_layout;
mod text_view;

fn find_vt100s(s: &str) -> Vec<Match> {
    let vt100_regex = Regex::new(r"((\u001b\[|\u009b)[\u0030-\u003f]*[\u0020-\u002f]*[\u0040-\u007e])+").unwrap();
    vt100_regex.find_iter(s).collect()
}

/***
Dim: Represents a constraint on layout.
    WrapContent -> Takes its size from the size of its children.
    Fixed(n)    -> Always 'n' characters, until the limits of the container or terminal get in the way.
    UpTo(n)     -> Resizes based on content between 0 and n characters.
 */
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Dim {
    WrapContent,
    Fixed(usize),
    UpTo(usize),
    // Between(usize, usize),
}

impl Dim {
    fn to_ord(&self) -> usize {
        match self {
            Dim::UpTo(x) => *x,
            Dim::Fixed(x) => *x,
            Dim::WrapContent => 1_000_000_000,
        }
    }
}

impl PartialOrd for Dim {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.to_ord().cmp(&other.to_ord()))
    }
}

impl Ord for Dim {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_ord().cmp(&other.to_ord())
    }
}
/***
View: A trait representing a render-able text widget.
 */
pub trait View {
    fn inflate(&mut self, parent_size: &CharDims) -> CharDims;
    fn constraints(&self) -> (Dim, Dim);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn render(&self) -> String;
    fn render_lines(&self) -> Vec<String>;
}

/***
TextView: A simple text container. Throw a String at it.
 */
pub struct TextView {
    raw_text: String,
    dims: Dimensions,
    formatter: Box<dyn TextFormatter>,
    visible: bool
}

/***
TextFormatter: A trait for classes that convert from a raw string into a formatted one.
    Generic in order to allow different Terminal backends to use their own custom
    String-variants.
 */
pub trait TextFormatter {
    fn format(&self, s: String, max_len: usize) -> String;
}

struct DumbFormatter{}

impl TextFormatter for DumbFormatter {
    fn format(&self, s: String, n: usize) -> String {
        let last_len = min(n, s.len());
        s[0..last_len].to_string()
    }
}

struct Vt100Formatter{}

impl TextFormatter for Vt100Formatter {
    fn format(&self, s: String, n: usize) -> String {
        if n >= s.len() { return format!("{:width$}", s, width = n) }

        let vt100s = find_vt100s(s.as_str());
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

/***
Orientation: For a LinearLayout. You know what this does.
 */
#[derive(Copy, Clone, Debug)]
pub enum Orientation {
    HORIZONTAL,
    VERTICAL
}

/***
LinearLayout: Prints child View widgets' contents, stacked horizontally or vertically.
 */
pub struct LinearLayout {
    orientation: Orientation,
    children: Vec<Rc<RefCell<dyn View>>>,
    dims: Dimensions,
    visible: bool
}

/***
Dimensions: An internal struct used to track the constraints and actual size of a View
 */
#[derive(Copy, Clone)]
pub struct Dimensions {
    width_constraint: Dim,
    height_constraint: Dim,
    size: CharDims  // Actual size in character glyphs
}

impl Dimensions {
    pub fn new(width: Dim, height: Dim) -> Dimensions{
        Dimensions {
            width_constraint: width,
            height_constraint: height,
            size: (0, 0), // Will be calculated during 'inflate' later.
        }
    }
}

pub type CharDims = (usize, usize);

pub fn desired_size(constraint: &Dim) -> usize {
    match constraint {
        Dim::WrapContent => 0, // If the constraint at this point is wrap content, we have to inflate children to see
        Dim::Fixed(x) => *x,
        Dim::UpTo(x) => *x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const VT100_TEST: &str = "T\u{1B}[33mE\u{1B}[96mS\u{1B}[39mT\u{1B}[39m"; // "TEST" interspersed with color codes for VT100 terminals

    #[test]
    fn dims_can_be_sorted() {
        assert!(Dim::Fixed(0) < Dim::Fixed(1));
        assert!(Dim::Fixed(1).to_ord() == Dim::UpTo(1).to_ord());
        assert!(Dim::Fixed(1) < Dim::UpTo(2));
        assert!(Dim::Fixed(1000) < Dim::WrapContent);
    }

    #[test]
    fn slicing_vt100_string_works() {
        let fmt = Vt100Formatter{};
        let fmt_str = fmt.format(VT100_TEST.to_string(), 2);
        assert_eq!("T\u{1B}[33mE", fmt_str);
    }
}