use crate::views::{View, TextView, Dim, Dimensions, desired_size, CharDims, ViewId};
use std::cmp::min;
use uuid::Uuid;
use std::slice::IterMut;
use log::info;
use crate::hexterm::formatting::TextFormatter;

impl TextView {
    pub fn new(width: Dim, height: Dim, formatter: Box<dyn TextFormatter>) -> TextView {
        TextView {
            id: Uuid::new_v4().to_string(),
            dims: Dimensions {
                width_constraint: width,
                height_constraint: height,
                size: (0, 0)
            },
            visible: true,
            text: "".to_string(),
            formatter: formatter,
            empty_children: Vec::new()
        }
    }
}

impl View for TextView {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn inflate(&mut self, parent_dimensions: &CharDims) -> CharDims {
        if !self.visible || self.text.is_empty() {
            self.dims.size = (0, 0);
            return self.dims.size;
        }

        match &self.text.len() {
            0 => { return self.dims.size },
            _ => {
                let lines = self.text.split("\n").collect::<Vec<&str>>();
                let height = lines.iter().count();
                let width = lines.iter().map(|l| l.len()).max().unwrap();
                let desired_width_constraint = Dim::UpTo(width);
                let desired_height_constraint  = Dim::UpTo(height);

                let most_restrictive_width = min(desired_width_constraint, min(self.dims.width_constraint, Dim::Fixed(parent_dimensions.0)));
                let most_restrictive_height= min(desired_height_constraint,  min(self.dims.height_constraint, Dim::Fixed(parent_dimensions.1)));

                self.dims.size = (desired_size(&most_restrictive_width),
                                  desired_size(&most_restrictive_height));

                self.dims.size.clone()
            }
        }
    }

    fn constraints(&self) -> (Dim, Dim) {
        (self.dims.width_constraint.clone(), self.dims.height_constraint.clone())
    }

    fn width(&self) -> usize { self.dims.size.0 }

    fn height(&self) -> usize { self.dims.size.1 }

    fn render(&self) -> String {
        self.render_lines().join("\n")
    }

    fn render_lines(&self) -> Vec<String> {
        self.text
            .split("\n")
            .take(self.height())
            .map(|s| self.formatter.format(s, self.width()))
            .collect()
    }

    fn children(&mut self) -> IterMut<Box<dyn View>> {
        self.empty_children.iter_mut()
    }

    fn replace_content(&mut self, text: String) {
        info!("Setting view text to {}", text);
        self.text = text;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hexterm::formatting::DumbFormatter;

    fn fixed_size_text_widget() -> TextView {
        TextView::new(Dim::Fixed(10), Dim::Fixed(2), Box::new(DumbFormatter{}))
    }

    fn wrap_content_text_widget() -> TextView {
        TextView::new(Dim::WrapContent, Dim::WrapContent, Box::new(DumbFormatter{}))
    }

    #[test]
    fn retrieves_constraints() {
        assert_eq!(fixed_size_text_widget().constraints(), (Dim::Fixed(10), Dim::Fixed(2)));
    }

    #[test]
    fn inflation_of_fixed_width_works_with_wrap_content_parent() {
        let mut tw = fixed_size_text_widget();
        tw.text = "line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.".to_string();
        tw.inflate(&(100, 100));
        assert_eq!(10, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_fixed_width_works_shrinks_to_fit_parent() {
        let mut tw = fixed_size_text_widget();
        tw.text = "line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.".to_string();
        tw.inflate(&(5, 100));
        assert_eq!(5, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_wrap_content_width_expands_to_line_length() {
        let mut tw = wrap_content_text_widget();
        tw.text = "line 1 is pretty long\nline 2 is shorter.".to_string();
        tw.inflate(&(100, 100));
        assert_eq!("line 1 is pretty long".len(), tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_wrap_content_width_shrinks_to_fixed_parent_dims() {
        let mut tw = wrap_content_text_widget();
        tw.text = "line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.".to_string();
        tw.inflate(&(3, 2));
        assert_eq!(3, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn renders_all_text_within_wrap_content() {
        let mut tw = wrap_content_text_widget();
        tw.text = "some\ntext".to_string();
        tw.inflate(&(100, 100));
        assert_eq!(String::from("some\ntext"), tw.render());
    }

    #[test]
    fn renders_partial_text_within_fixed_size() {
        let mut tw = fixed_size_text_widget();
        tw.text = "some really long text\nand another really long line\nthis line doesn't show up at all".to_string();
        tw.inflate(&(100, 100));
        assert_eq!(String::from("some reall\nand anothe"), tw.render());
    }

    #[test]
    fn when_invisible_renders_nothing() {
        let mut tw = fixed_size_text_widget();
        tw.text = "some really long text\nand another really long line\nthis line doesn't show up at all".to_string();
        tw.visible = false;
        tw.inflate(&(100, 100));
        assert_eq!(String::from(""), tw.render());
        assert_eq!(0, tw.render_lines().len());
    }

    #[test]
    fn when_invisible_dims_are_0() {
        let mut tw = fixed_size_text_widget();
        tw.text = "some really long text\nand another really long line\nthis line doesn't show up at all".to_string();
        tw.visible = false;
        tw.inflate(&(100, 100));
        assert_eq!(tw.dims.size, (0, 0));
    }
}