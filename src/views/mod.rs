use std::cmp::{Ordering, min};
use std::rc::Rc;
use std::cell::RefCell;
use regex::{Match, Regex};
use crate::hexterm::formatting::TaskText;
use crate::tasks::Layout;
use log::{trace, info};
use crate::terminal::{WindowMap, RcView};
use std::slice::IterMut;

mod linear_layout;
mod text_view;

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
    fn id(&self) -> ViewId;
    fn inflate(&mut self, parent_size: &CharDims) -> CharDims;
    fn constraints(&self) -> (Dim, Dim);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn render(&self) -> String;
    fn render_lines(&self) -> Vec<String>;
    fn children(&mut self) -> IterMut<Box<dyn View>>;
    fn replace_content(&mut self, text: String);
}

/***
TextView: A simple text container. The only thing that _displays_ stuff.
 */
pub type ViewId = String;
pub struct TextView {
    id: ViewId,
    dims: Dimensions,
    visible: bool,
    text: String,
    empty_children: Vec<Box<dyn View>> //Just for an empty list we can return.
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
LinearLayout: Prints child View views' contents, stacked horizontally or vertically.
 */
pub struct LinearLayout {
    id: ViewId,
    orientation: Orientation,
    children: Vec<Box<dyn View>>,
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
}