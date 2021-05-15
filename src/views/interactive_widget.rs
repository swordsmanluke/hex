use crate::views::{View, Widget, DimConstraint, Dimensions, desired_size, CharDims, ViewId, InteractiveWidget, TermLocation};
use std::cmp::{min, max};
use uuid::Uuid;
use std::slice::IterMut;
use log::info;
use crate::hexterm::formatting::TextFormatter;
use termion::cursor::Goto;
use std::io::{stdout, Write};

impl InteractiveWidget {
    pub fn new(width: DimConstraint, height: DimConstraint, location: TermLocation) -> InteractiveWidget {
        InteractiveWidget {
            id: Uuid::new_v4().to_string(),
            dims: Dimensions {
                width_constraint: width,
                height_constraint: height,
                size: CharDims::new(0, 0)
            },
            location: location,
            visible: true,
            empty_children: Vec::new(),
            dirty: true
        }
    }
}

impl View for InteractiveWidget {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn dirty(&self) -> bool {
        return self.dirty
    }

    fn wash(&mut self) {
        self.dirty = false
    }

    fn inflate(&mut self, parent_dimensions: &CharDims, location: &TermLocation) -> CharDims {
        if self.location != *location {
            self.location = location.clone();
            self.dirty = true;
        }

        if !self.visible {
            self.dims.size = CharDims::new(0, 0);
            return self.dims.size;
        }

        let new_size = self.update_dims(parent_dimensions);
        if new_size != self.dims.size {
            clear_area(self.location, self.dims.size);
            self.dims.size = new_size;
        }

        self.dims.size.clone()
    }

    fn constraints(&self) -> (DimConstraint, DimConstraint) {
        (self.dims.width_constraint.clone(), self.dims.height_constraint.clone())
    }

    fn width(&self) -> usize { self.dims.size.width }

    fn height(&self) -> usize { self.dims.size.height }

    fn render(&self) -> String {
        String::new()
    }

    fn children(&mut self) -> IterMut<Box<dyn View>> {
        self.empty_children.iter_mut()
    }

    // InteractiveWidgets print input immediately. It must already have been formatted, etc.
    fn update_content(&mut self, text: String) {
        self.print(text);
    }
}

impl InteractiveWidget {

    pub(crate) fn print(&self, text: String) {
        write!(stdout(), "{}", text);
    }

    fn update_dims(&mut self, parent_dimensions: &CharDims) -> CharDims {
        let desired_width_constraint = DimConstraint::UpTo(self.width());
        let desired_height_constraint = DimConstraint::UpTo(self.height());

        let least_restrictive_width = max(desired_width_constraint,
                                         max(self.dims.width_constraint, DimConstraint::Fixed(parent_dimensions.width)));

        let least_restrictive_height = max(desired_height_constraint,
                                          max(self.dims.height_constraint, DimConstraint::Fixed(parent_dimensions.height)));

        CharDims::new(desired_size(&least_restrictive_height),
         desired_size(&least_restrictive_width))
    }
}

fn clear_area(location: TermLocation, dims: CharDims) {
    for i in 0..dims.height as u16 {
        writeln!(stdout(), "{}{:width$}", Goto(location.x, location.y + i), " ", width=dims.width).unwrap();
        stdout().flush().unwrap();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn subject() -> InteractiveWidget {
        InteractiveWidget::new(DimConstraint::Fixed(3), DimConstraint::Fixed(3), TermLocation::new(1, 1))
    }

}