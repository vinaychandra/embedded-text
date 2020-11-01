//! TextBox for embedded-graphics.
//!
//! This crate provides a configurable [`TextBox`] to render multiline text inside a bounding
//! `Rectangle` using [embedded-graphics].
//!
//! [`TextBox`] supports the common text alignments:
//!  - Horizontal:
//!      - [`LeftAligned`]
//!      - [`RightAligned`]
//!      - [`CenterAligned`]
//!      - [`Justified`]
//!  - Vertical:
//!      - [`TopAligned`]
//!      - [`CenterAligned`]
//!      - [`BottomAligned`]
//!
//! [`TextBox`] also supports some special characters not handled by embedded-graphics' `Text`:
//!  - non-breaking space (`\u{200b}`)
//!  - zero-width space (`\u{a0}`)
//!  - soft hyphen (`\u{ad}`)
//!  - carriage return (`\r`)
//!  - tab (`\t`) with configurable tab size
//!
//! `TextBox` also supports text coloring using [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code).
//!
//! ### Example
//!
//! The examples are based on [the embedded-graphics simulator]. The simulator is built on top of
//! `SDL2`. See the [simulator README] for more information.
//!
//! ![embedded-text example with center aligned text](https://raw.githubusercontent.com/bugadani/embedded-text/master/assets/center.png)
//!
//! ![embedded-text example with colored text](https://raw.githubusercontent.com/bugadani/embedded-text/master/assets/colored_text.png)
//!
//! ```rust,no_run
//! use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
//! use embedded_graphics_simulator::{
//!     BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
//! };
//! use embedded_text::prelude::*;
//!
//! fn main() -> Result<(), core::convert::Infallible> {
//!     let text = "Hello, World!\nLorem Ipsum is simply dummy text of the printing and typesetting \
//!     industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
//!     an unknown printer took a galley of type and scrambled it to make a type specimen book.";
//!
//!     let textbox_style = TextBoxStyleBuilder::new(Font6x8)
//!         .alignment(CenterAligned)
//!         .height_mode(FitToText)
//!         .text_color(BinaryColor::On)
//!         .build();
//!
//!     let text_box = TextBox::new(text, Rectangle::with_corners(Point::zero(), Point::new(128, 0)))
//!         .into_styled(textbox_style);
//!
//!     // Create a window just tall enough to fit the text.
//!     let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(text_box.size());
//!     text_box.draw(&mut display).unwrap();
//!
//!     let output_settings = OutputSettingsBuilder::new()
//!         .theme(BinaryColorTheme::OledBlue)
//!         .build();
//!     Window::new("Hello center aligned TextBox", &output_settings).show_static(&display);
//!     Ok(())
//! }
//! ```
//!
//! [embedded-graphics]: https://github.com/jamwaffles/embedded-graphics/
//! [the embedded-graphics simulator]: https://github.com/jamwaffles/embedded-graphics/tree/master/simulator
//! [simulator README]: https://github.com/jamwaffles/embedded-graphics/tree/master/simulator#usage-without-sdl2
//! [`TextBox`]: ./struct.TextBox.html
//! [`LeftAligned`]: ./alignment/left/struct.LeftAligned.html
//! [`RightAligned`]: ./alignment/right/struct.RightAligned.html
//! [`CenterAligned`]: ./alignment/center/struct.CenterAligned.html
//! [`Justified`]: ./alignment/justified/struct.Justified.html
//! [`TopAligned`]: ./alignment/top/struct.TopAligned.html
//! [`BottomAligned`]: ./alignment/bottom/struct.BottomAligned.html

#![cfg_attr(not(test), no_std)]
#![deny(clippy::missing_inline_in_public_items)]
#![deny(clippy::cargo)]
#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod alignment;
pub mod parser;
pub mod rendering;
pub mod style;
pub mod utils;

use alignment::{HorizontalTextAlignment, VerticalTextAlignment};
use embedded_graphics::{prelude::*, primitives::Rectangle};
use rendering::RendererFactory;
use style::{height_mode::HeightMode, TextBoxStyle};
use utils::rect_ext::RectExt;

/// Prelude.
///
/// A collection of useful imports. Also re-exports some types from `embedded-graphics` for
/// convenience.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        alignment::*,
        style::{
            height_mode::{Exact, FitToText, HeightMode, ShrinkToText},
            TextBoxStyle, TextBoxStyleBuilder,
        },
        StyledTextBox, TextBox,
    };

    #[doc(no_inline)]
    pub use embedded_graphics::{
        primitives::Rectangle,
        style::{TextStyle, TextStyleBuilder},
    };
}

/// A textbox object.
///
/// The `TextBox` struct represents a piece of text that can be drawn on a display inside the given
/// bounding box.
///
/// The struct only contains the text and the bounding box, no additional information. To draw
/// a `TextBox` it is necessary to attach a [`TextBoxStyle`] to it using the [`into_styled`] method
/// to create a [`StyledTextBox`] object.
///
/// See the [module-level documentation] for more information.
///
/// [`into_styled`]: #method.into_styled
/// [`StyledTextBox`]: struct.StyledTextBox.html
/// [`TextBoxStyle`]: style/struct.TextBoxStyle.html
/// [module-level documentation]: index.html
pub struct TextBox<'a> {
    /// The text to be displayed in this `TextBox`
    pub text: &'a str,

    /// The bounding box of this `TextBox`
    pub bounds: Rectangle,
}

impl<'a> TextBox<'a> {
    /// Creates a new `TextBox` instance with a given bounding `Rectangle`.
    #[inline]
    #[must_use]
    pub fn new(text: &'a str, bounds: Rectangle) -> Self {
        Self {
            text,
            bounds: bounds.into_well_formed(),
        }
    }

    /// Creates a [`StyledTextBox`] by attaching a [`TextBoxStyle`] to the `TextBox` object.
    ///
    /// By default, the size of the [`StyledTextBox`] is equal to the size of the [`TextBox`]. Use
    /// [`HeightMode`] options to change this.
    ///
    /// # Example:
    ///
    /// In this example, we make a [`TextBox`] and give it all our available space as size.
    /// We create a [`TextBoxStyle`] object to set how our [`TextBox`] should be drawn.
    ///  * Set the 6x8 font
    ///  * Set the text color to `BinaryColor::On`
    ///  * Leave the background color transparent
    ///  * Leave text alignment top/left
    ///  * Set [`ShrinkToText`] [`HeightMode`] to shrink the [`TextBox`] when possible.
    ///
    /// ```rust
    /// use embedded_text::{prelude::*, style::vertical_overdraw::FullRowsOnly};
    /// use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
    ///
    /// let text_box = TextBox::new(
    ///     "Two lines\nof text",
    ///     Rectangle::with_corners(Point::zero(), Point::new(59, 59)),
    /// );
    /// let style = TextBoxStyleBuilder::new(Font6x8)
    ///     .height_mode(ShrinkToText(FullRowsOnly))
    ///     .text_color(BinaryColor::On)
    ///     .build();
    ///
    /// let styled_text_box = text_box.into_styled(style);
    /// assert_eq!(16, styled_text_box.size().height);
    /// ```
    ///
    /// [`HeightMode`]: style/height_mode/trait.HeightMode.html
    /// [`ShrinkToText`]: style/height_mode/struct.ShrinkToText.html
    #[inline]
    #[must_use]
    pub fn into_styled<C, F, A, V, H>(
        self,
        style: TextBoxStyle<C, F, A, V, H>,
    ) -> StyledTextBox<'a, C, F, A, V, H>
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        let mut styled = StyledTextBox {
            text_box: self,
            style,
        };
        H::apply(&mut styled);

        styled
    }
}

impl Transform for TextBox<'_> {
    #[inline]
    #[must_use]
    fn translate(&self, by: Point) -> Self {
        Self {
            bounds: self.bounds.translate(by),
            ..*self
        }
    }

    #[inline]
    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.bounds.translate_mut(by);

        self
    }
}

impl Dimensions for TextBox<'_> {
    #[inline]
    #[must_use]
    fn bounding_box(&self) -> Rectangle {
        self.bounds
    }
}

/// A styled [`TextBox`] struct.
///
/// This structure is constructed by calling the [`into_styled`] method of a [`TextBox`] object.
/// Use the [`draw`] method to draw the textbox on a display.
///
/// [`TextBox`]: struct.TextBox.html
/// [`into_styled`]: struct.TextBox.html#method.into_styled
/// [`draw`]: #method.draw
pub struct StyledTextBox<'a, C, F, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    /// A [`TextBox`] that has an associated [`TextBoxStyle`].
    ///
    /// [`TextBoxStyle`]: style/struct.TextBoxStyle.html
    pub text_box: TextBox<'a>,

    /// The style of the [`TextBox`].
    pub style: TextBoxStyle<C, F, A, V, H>,
}

impl<C, F, A, V, H> StyledTextBox<'_, C, F, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    /// Sets the height of the [`StyledTextBox`] to the height of the text.
    #[inline]
    pub fn fit_height(&mut self) -> &mut Self {
        self.fit_height_limited(u32::max_value())
    }

    /// Sets the height of the [`StyledTextBox`] to the height of the text, limited to `max_height`.
    ///
    /// This method allows you to set a maximum height. The [`StyledTextBox`] will take up at most
    /// `max_height` pixel vertical space.
    #[inline]
    pub fn fit_height_limited(&mut self, max_height: u32) -> &mut Self {
        // Measure text given the width of the textbox
        let text_height = self
            .style
            .measure_text_height(self.text_box.text, self.text_box.bounding_box().size.width)
            .min(max_height)
            .min(i32::max_value() as u32) as i32;

        // Apply height
        // let y = self.text_box.bounds.top_left.y;
        // let new_y = y.saturating_add(text_height - 1);
        // self.text_box.bounds.bottom_right.y = new_y;
        self.text_box.bounds.size.height = (text_height - 1) as u32;

        self
    }
}

impl<'a, C, F, A, V, H> Drawable for StyledTextBox<'a, C, F, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    StyledTextBox<'a, C, F, A, V, H>: RendererFactory<'a, C>,
    H: HeightMode,
{
    type Color = C;

    #[inline]
    fn draw<D: DrawTarget<Color = C>>(&self, display: &mut D) -> Result<(), D::Error> {
        display.draw_iter(StyledTextBox::create_renderer(self))
    }
}

impl<C, F, A, V, H> Transform for StyledTextBox<'_, C, F, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    #[inline]
    #[must_use]
    fn translate(&self, by: Point) -> Self {
        Self {
            text_box: self.text_box.translate(by),
            style: self.style,
        }
    }

    #[inline]
    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.text_box.bounds.translate_mut(by);

        self
    }
}

impl<C, F, A, V, H> Dimensions for StyledTextBox<'_, C, F, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    #[inline]
    #[must_use]
    fn bounding_box(&self) -> Rectangle {
        self.text_box.bounds
    }
}
