use crate::widgets::{View, TextView, Dim, Dimensions, desired_size, Vt100Formatter, CharDims};
use std::cmp::min;

impl TextView {
    pub fn new(width: Dim, height: Dim) -> TextView {
        TextView {
            raw_text: "".to_string(),
            dims: Dimensions {
                width_constraint: width,
                height_constraint: height,
                size: (0, 0)
            },
            formatter: Box::new(Vt100Formatter{}),
            visible: true
        }
    }

    pub fn update_content(&mut self, s: String) -> () {
        self.raw_text = s;
    }
}

impl View for TextView {
    fn inflate(&mut self, parent_dimensions: &CharDims) -> CharDims {
        if !self.visible {
            self.dims.size = (0, 0);
            return self.dims.size;
        }

        let text_size = self.raw_text.split("\n").map(|c| c.len()).max().unwrap();
        let desired_width_constraint = Dim::UpTo(text_size);
        let desired_height_constraint  = Dim::UpTo(self.raw_text.split("\n").count());

        let most_restrictive_width = min(desired_width_constraint, min(self.dims.width_constraint, Dim::Fixed(parent_dimensions.0)));
        let most_restrictive_height= min(desired_height_constraint,  min(self.dims.height_constraint, Dim::Fixed(parent_dimensions.1)));

        self.dims.size = (desired_size(&most_restrictive_width),
                          desired_size(&most_restrictive_height));

        self.dims.size.clone()
    }

    fn constraints(&self) -> (Dim, Dim) {
        (self.dims.width_constraint.clone(), self.dims.height_constraint.clone())
    }

    fn width(&self) -> usize { self.dims.size.0 }

    fn height(&self) -> usize { self.dims.size.1 }

    fn render(&self) -> String {
        self.raw_text.
            split("\n").take(self.height()). // First n Lines
            map(|c| self.formatter.format(c.to_string(), self.width())). // Format them
            collect::<Vec<String>>().join("\n")     // Convert back into a single string
    }

    fn render_lines(&self) -> Vec<String> {
        self.render()
            .split("\n")
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_size_text_widget() -> TextView {
        TextView::new(Dim::Fixed(10), Dim::Fixed(2))
    }

    fn wrap_content_text_widget() -> TextView {
        TextView::new(Dim::WrapContent, Dim::WrapContent)
    }

    #[test]
    fn retrieves_constraints() {
        assert_eq!(fixed_size_text_widget().constraints(), (Dim::Fixed(10), Dim::Fixed(2)));
    }

    #[test]
    fn inflation_of_fixed_width_works_with_wrap_content_parent() {
        let mut tw = fixed_size_text_widget();
        tw.raw_text = String::from("line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.");
        tw.inflate(&(100, 100));
        assert_eq!(10, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_fixed_width_works_shrinks_to_fit_parent() {
        let mut tw = fixed_size_text_widget();
        tw.raw_text = String::from("line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.");
        tw.inflate(&(5, 100));
        assert_eq!(5, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_wrap_content_width_expands_to_line_length() {
        let mut tw = wrap_content_text_widget();
        tw.raw_text = String::from("line 1 is pretty long\nline 2 is shorter.");
        tw.inflate(&(100, 100));
        assert_eq!("line 1 is pretty long".len(), tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_wrap_content_width_shrinks_to_fixed_parent_dims() {
        let mut tw = wrap_content_text_widget();
        tw.raw_text = String::from("line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.");
        tw.inflate(&(3, 2));
        assert_eq!(3, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn renders_all_text_within_wrap_content() {
        let mut tw = wrap_content_text_widget();
        tw.raw_text = String::from("some\ntext");
        tw.inflate(&(100, 100));
        assert_eq!(String::from("some\ntext"), tw.render());
    }

    #[test]
    fn renders_partial_text_within_fixed_size() {
        let mut tw = fixed_size_text_widget();
        tw.raw_text = String::from("some really long text\nand another really long line\nthis line doesn't show up at all");
        tw.inflate(&(100, 100));
        assert_eq!(String::from("some reall\nand anothe"), tw.render());
    }

    #[test]
    fn when_invisible_renders_nothing() {
        let mut tw = fixed_size_text_widget();
        tw.raw_text = String::from("some really long text\nand another really long line\nthis line doesn't show up at all");
        tw.visible = false;
        tw.inflate(&(100, 100));
        assert_eq!(String::from(""), tw.render());
        assert_eq!(vec![""], tw.render_lines());
    }

    #[test]
    fn when_invisible_dims_are_0() {
        let mut tw = fixed_size_text_widget();
        tw.raw_text = String::from("some really long text\nand another really long line\nthis line doesn't show up at all");
        tw.visible = false;
        tw.inflate(&(100, 100));
        assert_eq!(tw.dims.size, (0, 0));
    }
}