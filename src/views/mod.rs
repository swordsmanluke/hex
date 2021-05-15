use std::cmp::Ordering;
use crate::hexterm::formatting::TextFormatter;
use std::slice::IterMut;

mod linear_layout;
mod widget;
mod window;
mod interactive_widget;
mod input_processor;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct TermLocation{ pub(crate) x: u16, pub(crate) y: u16 }

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct CharDims { pub(crate) width: usize, pub(crate) height: usize }

impl TermLocation { pub fn new(x: u16, y: u16) -> TermLocation { TermLocation{ x, y } } }
impl CharDims { pub fn new(width: usize, height: usize) -> CharDims { CharDims{ width, height } } }

/***
DimConstraint: Represents a constraint on layout.
    WrapContent -> Takes its size from the size of its children.
    Fixed(n)    -> Always 'n' characters, until the limits of the container or terminal get in the way.
    UpTo(n)     -> Resizes based on content between 0 and n characters.
 */
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum DimConstraint {
    WrapContent,
    Fixed(usize),
    UpTo(usize),
    // Between(usize, usize),
}

impl DimConstraint {
    fn to_ord(&self) -> usize {
        match self {
            DimConstraint::UpTo(x) => *x,
            DimConstraint::Fixed(x) => *x,
            DimConstraint::WrapContent => 1_000_000_000,
        }
    }
}

impl PartialOrd for DimConstraint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.to_ord().cmp(&other.to_ord()))
    }
}

impl Ord for DimConstraint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_ord().cmp(&other.to_ord())
    }
}
/***
View: A trait representing a render-able text widget.
 */
pub trait View {
    fn id(&self) -> ViewId;
    fn dirty(&self) -> bool;
    fn wash(&mut self);
    fn inflate(&mut self, parent_size: &CharDims, location: &TermLocation) -> CharDims;
    fn constraints(&self) -> (DimConstraint, DimConstraint);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn render(&self) -> String;
    fn children(&mut self) -> IterMut<Box<dyn View>>;
    fn update_content(&mut self, text: String);
}

/***
Widget: A simple text container. The thing that displays non-interactive stuff.
 */
pub type ViewId = String;
pub struct Widget {
    id: ViewId,
    location: TermLocation,
    dims: Dimensions,
    visible: bool,
    text: String,
    formatter: Box<dyn TextFormatter>,
    dirty: bool,
    empty_children: Vec<Box<dyn View>> //Just for an empty list we can return.
}

/***
InteractiveWidget: An even simpler text container. The thing that prints interactive stuff.
 */
pub struct InteractiveWidget {
    id: ViewId,
    location: TermLocation,
    dims: Dimensions,
    visible: bool,
    dirty: bool,
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
    location: TermLocation,
    visible: bool
}

/***
Dimensions: An internal struct used to track the constraints and actual size of a View
 */
#[derive(Copy, Clone)]
pub struct Dimensions {
    width_constraint: DimConstraint,
    height_constraint: DimConstraint,
    size: CharDims  // Actual size in character glyphs
}

impl Dimensions {
    pub fn new(width: DimConstraint, height: DimConstraint) -> Dimensions{
        Dimensions {
            width_constraint: width,
            height_constraint: height,
            size: CharDims::new(0,  0), // Will be updated during 'inflate' later.
        }
    }
}

pub fn desired_size(constraint: &DimConstraint) -> usize {
    match constraint {
        DimConstraint::WrapContent => 0, // If the constraint at this point is wrap content, we have to inflate children to see
        DimConstraint::Fixed(x) => *x,
        DimConstraint::UpTo(x) => *x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dims_can_be_sorted() {
        assert!(DimConstraint::Fixed(0) < DimConstraint::Fixed(1));
        assert!(DimConstraint::Fixed(1).to_ord() == DimConstraint::UpTo(1).to_ord());
        assert!(DimConstraint::Fixed(1) < DimConstraint::UpTo(2));
        assert!(DimConstraint::Fixed(1000) < DimConstraint::WrapContent);
    }
}