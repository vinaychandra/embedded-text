//! Pixel iterators used for text rendering

/// Character rendering
pub mod character;

/// Whitespace rendering
pub mod whitespace;

/// Line rendering
pub mod line;

/// Cursor to track rendering position
pub mod cursor;

use crate::{
    alignment::TextAlignment,
    parser::Parser,
    style::{StyledTextBox, TextBoxStyle},
};
use embedded_graphics::prelude::*;

/// This trait is used to associate a state type to a horizontal alignment option.
pub trait StateFactory {
    /// The type of the state variable used for rendering.
    type PixelIteratorState;

    /// Creates a new state variable.
    fn create_state(&self) -> Self::PixelIteratorState;
}

/// Pixel iterator for styled text.
pub struct StyledTextBoxIterator<'a, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
    StyledTextBox<'a, C, F, A>: StateFactory,
{
    /// Parser to process the text during rendering
    pub parser: Parser<'a>,

    /// Style used for rendering
    pub style: TextBoxStyle<C, F, A>,

    /// State information used by the rendering algorithms
    pub state: <StyledTextBox<'a, C, F, A> as StateFactory>::PixelIteratorState,
}

impl<'a, C, F, A> StyledTextBoxIterator<'a, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
    StyledTextBox<'a, C, F, A>: StateFactory,
{
    /// Creates a new pixel iterator to render the styled [`TextBox`]
    ///
    /// [`TextBox`]: ../struct.TextBox.html
    #[inline]
    #[must_use]
    pub fn new(styled: &'a StyledTextBox<'a, C, F, A>) -> Self {
        Self {
            parser: Parser::parse(styled.text_box.text),
            style: styled.style,
            state: styled.create_state(),
        }
    }
}
