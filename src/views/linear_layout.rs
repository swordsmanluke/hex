use crate::views::{LinearLayout, Orientation, View, Dim, Dimensions, desired_size, CharDims, ViewId};
use std::cmp::{min, max};
use log::info;
use uuid::Uuid;
use std::slice::IterMut;


impl LinearLayout {
    pub fn new(orientation: Orientation, width: Dim, height: Dim, location: (u16, u16)) -> LinearLayout {
        LinearLayout {
            id: Uuid::new_v4().to_string(),
            orientation: orientation,
            dims: Dimensions::new(width, height),
            location: location,
            children: vec![],
            visible: true,
        }
    }

    pub fn add_child(&mut self, child: Box<dyn View>) {
        self.children.push(child);
    }

    fn update_child_dims(orientation: Orientation, childrens_desired_dims: CharDims, child_dims: CharDims) -> CharDims {
        // Sum our children in the direction we are stacking them.
        // Capture the maximum in the direction we are stretching.
        // e.g. for Vertical, we stack by height, so sum those.
        //      ...then stretch sideways to the max child width.
        match orientation {
            Orientation::HORIZONTAL => {
                (childrens_desired_dims.0 + child_dims.0,
                 max(childrens_desired_dims.1, child_dims.1))
            }
            Orientation::VERTICAL => {
                (max(childrens_desired_dims.0, child_dims.0),
                 childrens_desired_dims.1 + child_dims.1)
            }
        }
    }

    fn update_parent_dims(orientation: Orientation, remaining_parent_dims: CharDims, child_dims: CharDims) -> CharDims {
        // Subtract remaining size in the direction we are stacking children.
        // Ignore in the direction we are stretching.
        // e.g. for Vertical, we stack by height, so subtract each child from that.
        match orientation {
            Orientation::VERTICAL => {
                (remaining_parent_dims.0,
                 if remaining_parent_dims.1 >= child_dims.1 { remaining_parent_dims.1 - child_dims.1 } else { 0 })
            }
            Orientation::HORIZONTAL => {
                (if remaining_parent_dims.0 >= child_dims.0 { remaining_parent_dims.0 - child_dims.0 } else { 0 },
                 remaining_parent_dims.1)
            }
        }
    }
}

impl View for LinearLayout {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn dirty(&self) -> bool {
        self.children.iter().any(|c| c.dirty())
    }

    fn wash(&mut self) {
        self.children.iter_mut().for_each(|c| c.wash());
    }

    fn inflate(&mut self, parent_dimensions: &CharDims, location: (u16, u16)) -> CharDims {
        self.location = location;
        if !self.visible {
            self.dims.size = (0, 0);
            return self.dims.size;
        }

        let mut childrens_desired_dims = (0, 0);
        let most_restrictive_width = min(self.dims.width_constraint, Dim::Fixed(parent_dimensions.0));
        let most_restrictive_height = min(self.dims.height_constraint, Dim::Fixed(parent_dimensions.1));

        self.dims.size = (desired_size(&most_restrictive_width),
                          desired_size(&most_restrictive_height));

        let mut remaining_parent_dims = self.dims.size.clone();

        let mut next_child_loc = location;
        for child in &mut self.children {
            let child_dims = child.inflate(&remaining_parent_dims, next_child_loc);
            childrens_desired_dims = LinearLayout::update_child_dims(self.orientation, childrens_desired_dims, child_dims);
            remaining_parent_dims = LinearLayout::update_parent_dims(self.orientation, remaining_parent_dims, child_dims);

            next_child_loc = match self.orientation {
                Orientation::HORIZONTAL =>  { (next_child_loc.0 + child.width() as u16, next_child_loc.1) },
                Orientation::VERTICAL =>    { (next_child_loc.0, next_child_loc.1 + child.height() as u16) }
            };
        }

        let new_most_restrictive_width = min(Dim::Fixed(childrens_desired_dims.0), most_restrictive_width);
        let new_most_restrictive_height = min(Dim::Fixed(childrens_desired_dims.1), most_restrictive_height);

        self.dims.size.0 = match new_most_restrictive_width {
            Dim::Fixed(n) => n,
            Dim::UpTo(n) => n,
            Dim::WrapContent => 0 // Only happens if we're a "WrapContent" and have 0 or empty children
        };

        self.dims.size.1 = match new_most_restrictive_height {
            Dim::Fixed(n) => n,
            Dim::UpTo(n) => n,
            Dim::WrapContent => 0 // Only happens if we're a "WrapContent" and have 0 or empty children
        };

        if self.height() == 0 {
            info!("LL {:?} Dimensions: {}x{}", self.orientation, self.width(), self.height());
            info!("LL zero height child dims height: {}; constraint: {:?}", childrens_desired_dims.1, self.dims.height_constraint);
        }

        self.dims.size.clone()
    }

    fn constraints(&self) -> (Dim, Dim) { (self.dims.width_constraint, self.dims.height_constraint) }

    fn width(&self) -> usize { self.dims.size.0 }

    fn height(&self) -> usize { self.dims.size.1 }

    fn render(&self) -> String {
        if !self.visible || !self.dirty() { return String::new() }
        self.children.iter().map(|c| c.render()).collect::<Vec<String>>().join("")
    }

    fn children(&mut self) -> IterMut<Box<dyn View>> {
        self.children.iter_mut()
    }

    fn replace_content(&mut self, _: String) {
        // No-op - you can't replace text in a LL.
        // I know this breaks Liscov substitution and I'm not much happier about it.
        return;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::TextView;
    use crate::hexterm::formatting::{DumbFormatter, Vt100Formatter};

    fn fixed_size_text_widget() -> TextView {
        let mut tw = TextView::new(Dim::Fixed(10), Dim::Fixed(2), Box::new(Vt100Formatter{}), (1, 1));
        tw.text = "This is some raw text\nwith multiple lines\nand then another line.".to_owned();
        tw
    }

    fn wrap_content_text_widget() -> TextView {
        let mut tw = TextView::new(Dim::WrapContent, Dim::WrapContent, Box::new(Vt100Formatter{}), (1, 1));
        tw.text = "This is some raw text\nwith multiple lines\nand then another line.".to_owned();
        tw
    }

    fn vert_ll_with_wrap_content() -> LinearLayout {
        LinearLayout::new(Orientation::VERTICAL, Dim::WrapContent, Dim::WrapContent, (1, 1))
    }

    fn vert_ll_with_fixed_size() -> LinearLayout {
        LinearLayout::new(Orientation::VERTICAL, Dim::Fixed(5), Dim::Fixed(2), (1, 1))
    }

    fn horz_ll_with_wrap_content() -> LinearLayout {
        LinearLayout::new(Orientation::HORIZONTAL, Dim::WrapContent, Dim::WrapContent, (1, 1))
    }

    #[test]
    fn retrieves_constraints() {
        assert_eq!(vert_ll_with_fixed_size().constraints(), (Dim::Fixed(5), Dim::Fixed(2)));
    }

    #[test]
    fn when_wrapping_content_takes_size_from_children() {
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.inflate(&(100, 100), (1, 1));
        assert_eq!(10, ll.width());
        assert_eq!(2, ll.height());
    }

    #[test]
    fn when_vert_wrapping_content_takes_horz_size_from_largest_child() {
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.add_child(Box::new(wrap_content_text_widget()));
        ll.inflate(&(100, 100), (1, 1));
        assert_eq!(ll.width(), 22);
    }

    #[test]
    fn when_vert_wrapping_content_takes_vert_size_from_summed_children() {
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.inflate(&(100, 100), (1, 1));
        assert_eq!(ll.height(), 4);
    }

    #[test]
    fn when_horz_wrapping_content_takes_horz_size_from_summed_children() {
        let mut ll = horz_ll_with_wrap_content();
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.inflate(&(100, 100), (1, 1));
        assert_eq!(ll.width(), 20);
    }

    #[test]
    fn when_horz_wrapping_content_takes_vert_size_from_tallest_child() {
        let mut ll = horz_ll_with_wrap_content();
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.add_child(Box::new(wrap_content_text_widget()));
        ll.inflate(&(100, 100), (1, 1));
        assert_eq!(ll.height(), 3);
    }

    #[test]
    fn vert_rendering_works() {
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.inflate(&(100, 100), (1, 1));

        assert_eq!("This is so\nwith multi".to_string(), ll.render());
    }

    #[test]
    fn vert_rendering_works_with_multiple_children() {
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.inflate(&(100, 100), (1, 1));

        assert_eq!("This is so\nwith multi\nThis is so\nwith multi".to_string(), ll.render());
    }

    #[test]
    fn horz_rendering_works_with_multiple_children() {
        let mut ll = horz_ll_with_wrap_content();
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.inflate(&(100, 100), (1, 1));

        assert_eq!("This is soThis is soThis is so\nwith multiwith multiwith multi".to_string(), ll.render());
    }

    #[test]
    fn when_invisible_renders_nothing() {
        let mut ll = vert_ll_with_fixed_size();
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.visible = false;
        ll.inflate(&(100, 100), (1, 1));
        assert_eq!(String::from(""), ll.render());
    }

    #[test]
    fn when_invisible_dims_are_0() {
        let mut ll = vert_ll_with_fixed_size();
        ll.add_child(Box::new(fixed_size_text_widget()));
        ll.visible = false;
        ll.inflate(&(100, 100), (1, 1));
        assert_eq!(ll.dims.size, (0, 0));
    }
}