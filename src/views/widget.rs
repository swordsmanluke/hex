use crate::views::{View, Widget, Dim, Dimensions, desired_size, CharDims, ViewId};
use std::cmp::min;
use uuid::Uuid;
use std::slice::IterMut;
use log::info;
use crate::hexterm::formatting::TextFormatter;
use termion::cursor::Goto;
use std::io::{stdout, Write};

impl Widget {
    pub fn new(width: Dim, height: Dim, formatter: Box<dyn TextFormatter>, location: (u16, u16)) -> Widget {
        Widget {
            id: Uuid::new_v4().to_string(),
            dims: Dimensions {
                width_constraint: width,
                height_constraint: height,
                size: (0, 0)
            },
            location: location,
            visible: true,
            text: "".to_string(),
            formatter: formatter,
            empty_children: Vec::new(),
            dirty: true
        }
    }
}

impl View for Widget {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn dirty(&self) -> bool {
        return self.dirty
    }

    fn wash(&mut self) {
        self.dirty = false
    }

    fn inflate(&mut self, parent_dimensions: &CharDims, location: (u16, u16)) -> CharDims {
        if self.location != location {
            self.location = location;
            self.dirty = true;
        }

        if !self.visible || self.text.is_empty() {
            self.dims.size = (0, 0);
            return self.dims.size;
        }

        let new_size = self.update_dims(parent_dimensions);
        if new_size != self.dims.size {
            clear_area(self.location, self.dims.size);
            self.dims.size = new_size;
        }

        self.dims.size.clone()
    }

    fn constraints(&self) -> (Dim, Dim) {
        (self.dims.width_constraint.clone(), self.dims.height_constraint.clone())
    }

    fn width(&self) -> usize { self.dims.size.0 }

    fn height(&self) -> usize { self.dims.size.1 }

    fn render(&self) -> String {
        if !self.dirty { return String::new() }
        self.formatter.format(self.text.as_str(), (self.width(), self.height()), self.location)
    }

    fn children(&mut self) -> IterMut<Box<dyn View>> {
        self.empty_children.iter_mut()
    }

    fn update_content(&mut self, text: String) {
        self.text = text;
        self.dirty = true; // gotta be updated!
    }
}

impl Widget {
    fn update_dims(&mut self, parent_dimensions: &(usize, usize)) -> CharDims {
        match &self.text.len() {
            0 => { self.dims.size.clone() },
            _ => {
                let lines = self.text.split("\n").collect::<Vec<&str>>();
                let height = lines.iter().count();
                let width = lines.iter().map(|l| l.len()).max().unwrap();
                let desired_width_constraint = Dim::UpTo(width);
                let desired_height_constraint = Dim::UpTo(height);

                let most_restrictive_width = min(desired_width_constraint, min(self.dims.width_constraint, Dim::Fixed(parent_dimensions.0)));
                let most_restrictive_height = min(desired_height_constraint, min(self.dims.height_constraint, Dim::Fixed(parent_dimensions.1)));

                (desired_size(&most_restrictive_width), desired_size(&most_restrictive_height))
            }
        }
    }
}

fn clear_area(location: (u16, u16), dims: CharDims) {
    for i in 0..dims.1 as u16 {
        writeln!(stdout(), "{}{:width$}", Goto(location.0, location.1 + i), " ", width=dims.0).unwrap();
        stdout().flush().unwrap();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::hexterm::formatting::Vt100Formatter;

    fn fixed_size_text_widget() -> Widget {
        Widget::new(Dim::Fixed(10), Dim::Fixed(2), Box::new(Vt100Formatter{}), (1, 1))
    }

    fn wrap_content_text_widget() -> Widget {
        Widget::new(Dim::WrapContent, Dim::WrapContent, Box::new(Vt100Formatter{}), (1, 1))
    }

    #[test]
    fn retrieves_constraints() {
        assert_eq!(fixed_size_text_widget().constraints(), (Dim::Fixed(10), Dim::Fixed(2)));
    }

    #[test]
    fn inflation_of_fixed_width_works_with_wrap_content_parent() {
        let mut tw = fixed_size_text_widget();
        tw.text = "line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.".to_string();
        tw.inflate(&(100, 100), (1, 1));
        assert_eq!(10, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_fixed_width_works_shrinks_to_fit_parent() {
        let mut tw = fixed_size_text_widget();
        tw.text = "line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.".to_string();
        tw.inflate(&(5, 100), (1, 1));
        assert_eq!(5, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_wrap_content_width_expands_to_line_length() {
        let mut tw = wrap_content_text_widget();
        tw.text = "line 1 is pretty long\nline 2 is shorter.".to_string();
        tw.inflate(&(100, 100), (1, 1));
        assert_eq!("line 1 is pretty long".len(), tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_wrap_content_width_shrinks_to_fixed_parent_dims() {
        let mut tw = wrap_content_text_widget();
        tw.text = "line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.".to_string();
        tw.inflate(&(3, 2), (1, 1));
        assert_eq!(3, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn renders_all_text_within_wrap_content() {
        let mut tw = wrap_content_text_widget();
        tw.text = "some\ntext".to_string();
        tw.inflate(&(100, 100), (1, 1));
        assert_eq!(String::from("\u{1b}[1;1Hsome\u{1b}[2;1Htext"), tw.render());
    }

    #[test]
    fn renders_partial_text_within_fixed_size() {
        let mut tw = fixed_size_text_widget();
        tw.text = "some really long text\nand another really long line\nthis line doesn't show up at all".to_string();
        tw.inflate(&(100, 100), (1, 1));
        assert_eq!(String::from("\u{1b}[1;1Hsome reall\u{1b}[2;1Hand anothe"), tw.render());
    }

    #[test]
    fn when_invisible_renders_nothing() {
        let mut tw = fixed_size_text_widget();
        tw.text = "some really long text\nand another really long line\nthis line doesn't show up at all".to_string();
        tw.visible = false;
        tw.inflate(&(100, 100), (1, 1));
        assert_eq!(String::from(""), tw.render());
    }

    #[test]
    fn when_invisible_dims_are_0() {
        let mut tw = fixed_size_text_widget();
        tw.text = "some really long text\nand another really long line\nthis line doesn't show up at all".to_string();
        tw.visible = false;
        tw.inflate(&(100, 100), (1, 1));
        assert_eq!(tw.dims.size, (0, 0));
    }
}