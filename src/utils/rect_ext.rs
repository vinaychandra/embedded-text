//! Rectangle helper extensions.
//!
//! Extends the embedded-graphics [`Rectangle`] struct with some helper methods.
use embedded_graphics::{prelude::*, primitives::Rectangle};

/// [`Rectangle`] extensions
pub trait RectExt {
    /// Returns the (correct) size of a [`Rectangle`].
    fn size(self) -> Size;

    /// Sorts the coordinates of a [`Rectangle`] so that `top` < `bottom` and `left` < `right`.
    fn into_well_formed(self) -> Rectangle;
}

impl RectExt for Rectangle {
    #[inline]
    #[must_use]
    fn size(self) -> Size {
        // TODO: remove if fixed in embedded-graphics
        let width = (self.bottom_right().unwrap().x - self.top_left.x) as u32 + 1;
        let height = (self.bottom_right().unwrap().y - self.top_left.y) as u32 + 1;

        Size::new(width, height)
    }

    #[inline]
    #[must_use]
    fn into_well_formed(self) -> Rectangle {
        Rectangle::with_corners(
            Point::new(
                self.top_left.x.min(self.bottom_right().unwrap().x),
                self.top_left.y.min(self.bottom_right().unwrap().y),
            ),
            Point::new(
                self.top_left.x.max(self.bottom_right().unwrap().x),
                self.top_left.y.max(self.bottom_right().unwrap().y),
            ),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_well_formed() {
        assert_eq!(
            Rectangle::with_corners(Point::new(0, -4), Point::new(3, 0)),
            Rectangle::with_corners(Point::new(3, -4), Point::zero()).into_well_formed()
        );
    }
}
