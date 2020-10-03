//! Parse text into words, newlines and whitespace sequences.
//!
//! ```rust
//! use embedded_text::parser::{Parser, Token};
//!
//! let parser = Parser::parse("Hello, world!\n");
//! let tokens = parser.collect::<Vec<Token<'_>>>();
//!
//! assert_eq!(
//!     vec![
//!         Token::Word("Hello,"),
//!         Token::Whitespace(1),
//!         Token::Word("world!"),
//!         Token::NewLine
//!     ],
//!     tokens
//! );
//! ```
use core::str::Chars;

/// A text token
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'a> {
    /// A newline character.
    NewLine,

    /// A \r character.
    CarriageReturn,

    /// A \t character.
    Tab,

    /// A \x1b
    Escape,

    /// A number of whitespace characters.
    Whitespace(u32),

    /// A word (a sequence of non-whitespace characters).
    Word(&'a str),

    /// A possible wrapping point
    Break(Option<char>),

    /// An extra character - used to carry soft breaking chars.
    ExtraCharacter(char),
}

/// Text parser. Turns a string into a stream of [`Token`] objects.
///
/// [`Token`]: enum.Token.html
#[derive(Clone, Debug)]
pub struct Parser<'a> {
    inner: Chars<'a>,
}

pub(crate) const SPEC_CHAR_NBSP: char = '\u{a0}';
pub(crate) const SPEC_CHAR_ZWSP: char = '\u{200b}';
pub(crate) const SPEC_CHAR_SHY: char = '\u{ad}';
pub(crate) const SPEC_CHAR_ESCAPE: char = '\x1b';

fn is_word_char(c: char) -> bool {
    // Word tokens are terminated when a whitespace, zwsp or shy character is found. An exception
    // to this rule is the nbsp, which is whitespace but is included in the word.
    (!c.is_whitespace() || c == SPEC_CHAR_NBSP)
        && ![SPEC_CHAR_ZWSP, SPEC_CHAR_SHY, SPEC_CHAR_ESCAPE].contains(&c)
}

fn is_space_char(c: char) -> bool {
    // zero-width space breaks whitespace sequences - this works as long as
    // space handling is symmetrical (i.e. starting == ending behaviour)
    c.is_whitespace() && !['\n', '\r', '\t', SPEC_CHAR_NBSP].contains(&c) || c == SPEC_CHAR_ZWSP
}

fn try_parse_escape_seq<'a>(chars: &mut Chars<'a>) -> Option<Token<'a>> {
    chars.next().and_then(|c| match c {
        SPEC_CHAR_ESCAPE => Some(Token::Escape),
        _ => None,
    })
}

impl<'a> Parser<'a> {
    /// Create a new parser object to process the given piece of text.
    #[inline]
    #[must_use]
    pub fn parse(text: &'a str) -> Self {
        Self {
            inner: text.chars(),
        }
    }

    /// Returns true if there are no tokens to process.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.as_str().is_empty()
    }

    fn remaining(&self) -> usize {
        self.inner.as_str().len()
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let string = self.inner.as_str();

        if let Some(c) = self.inner.next() {
            let mut iter = self.inner.clone();

            if is_word_char(c) {
                // find the longest consecutive slice of text for a Word token
                while let Some(c) = iter.next() {
                    if is_word_char(c) {
                        // Need to advance internal state here, otherwise we would need to store the
                        // revious iterator state and overwrite self.inner in the current else
                        // branch.
                        // This copy seems unavoidable.
                        self.inner = iter.clone();
                    } else {
                        let offset = string.len() - self.remaining();
                        return Some(Token::Word(unsafe {
                            // don't worry
                            string.get_unchecked(0..offset)
                        }));
                    }
                }

                // consume all the text
                self.inner = "".chars();

                Some(Token::Word(&string))
            } else {
                match c {
                    // special characters
                    '\n' => Some(Token::NewLine),
                    '\r' => Some(Token::CarriageReturn),
                    '\t' => Some(Token::Tab),
                    SPEC_CHAR_ZWSP => Some(Token::Break(None)),
                    SPEC_CHAR_SHY => Some(Token::Break(Some('-'))),
                    SPEC_CHAR_ESCAPE => {
                        let mut lookahead = self.inner.clone();
                        if let tok @ Some(_) = try_parse_escape_seq(&mut lookahead) {
                            self.inner = lookahead;
                            tok
                        } else {
                            // we don't have anything, so ignore the escape character and move on
                            Some(Token::Escape)
                        }
                    }

                    // count consecutive whitespace
                    _ => {
                        let mut len = 1;
                        while let Some(c) = iter.next() {
                            if is_space_char(c) {
                                if c != SPEC_CHAR_ZWSP {
                                    len += 1;
                                }
                                // Same as in the word case, this copy seems unavoidable.
                                self.inner = iter.clone();
                            } else {
                                // consume the whitespaces
                                return Some(Token::Whitespace(len));
                            }
                        }

                        // consume all the text
                        self.inner = "".chars();

                        Some(Token::Whitespace(len))
                    }
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Parser, Token};

    fn assert_tokens(text: &str, tokens: Vec<Token>) {
        assert_eq!(Parser::parse(text).collect::<Vec<Token>>(), tokens)
    }

    #[test]
    fn test_parse() {
        // (At least) for now, \r is considered a whitespace
        assert_tokens(
            "Lorem ipsum \r dolor sit am\u{00AD}et,\tconse😅ctetur adipiscing\nelit",
            vec![
                Token::Word("Lorem"),
                Token::Whitespace(1),
                Token::Word("ipsum"),
                Token::Whitespace(1),
                Token::CarriageReturn,
                Token::Whitespace(1),
                Token::Word("dolor"),
                Token::Whitespace(1),
                Token::Word("sit"),
                Token::Whitespace(1),
                Token::Word("am"),
                Token::Break(Some('-')),
                Token::Word("et,"),
                Token::Tab,
                Token::Word("conse😅ctetur"),
                Token::Whitespace(1),
                Token::Word("adipiscing"),
                Token::NewLine,
                Token::Word("elit"),
            ],
        );
    }

    #[test]
    fn parse_zwsp() {
        assert_eq!(9, "two\u{200B}words".chars().count());

        assert_tokens(
            "two\u{200B}words",
            vec![Token::Word("two"), Token::Break(None), Token::Word("words")],
        );

        assert_tokens("  \u{200B} ", vec![Token::Whitespace(3)]);
    }

    #[test]
    fn parse_multibyte_last() {
        assert_tokens("test😅", vec![Token::Word("test😅")]);
    }

    #[test]
    fn parse_nbsp_as_word_char() {
        assert_eq!(9, "test\u{A0}word".chars().count());
        assert_tokens("test\u{A0}word", vec![Token::Word("test\u{A0}word")]);
        assert_tokens(
            " \u{A0}word",
            vec![Token::Whitespace(1), Token::Word("\u{A0}word")],
        );
    }

    #[test]
    fn parse_shy_issue_42() {
        assert_tokens(
            "foo\u{AD}bar",
            vec![
                Token::Word("foo"),
                Token::Break(Some('-')),
                Token::Word("bar"),
            ],
        );
    }

    #[test]
    fn escape_char_ignored_if_not_ansi_sequence() {
        assert_tokens(
            "foo\x1bbar",
            vec![Token::Word("foo"), Token::Escape, Token::Word("bar")],
        );

        // can escape the escape char
        assert_tokens(
            "foo\x1b\x1bbar",
            vec![Token::Word("foo"), Token::Escape, Token::Word("bar")],
        );
    }
}
