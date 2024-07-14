use std::fmt;

use super::{next_word, prev_word, CursorMove, Input};

#[derive(Default)]
pub struct OnelineInput {
    value: String,
    cursor_col: usize,
    scroll_offset_col: usize,
    selection_start: Option<usize>,
}

impl fmt::Display for OnelineInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl From<&str> for OnelineInput {
    fn from(item: &str) -> Self {
        Self {
            value: item.into(),
            cursor_col: 0,
            scroll_offset_col: 0,
            selection_start: None,
        }
    }
}

impl From<String> for OnelineInput {
    fn from(item: String) -> Self {
        Self {
            value: item,
            cursor_col: 0,
            scroll_offset_col: 0,
            selection_start: None,
        }
    }
}

impl Into<String> for OnelineInput {
    fn into(self) -> String {
        self.value().into()
    }
}

impl Input for OnelineInput {
    fn value(&self) -> String {
        self.value.clone()
    }

    fn len(&self) -> usize {
        self.value.chars().count()
    }

    fn cursor(&self) -> (usize, usize) {
        (self.cursor_col, 0)
    }

    fn move_cursor(&mut self, cursor_move: CursorMove) {
        match cursor_move {
            CursorMove::NextChar => self.cursor_col = self.len().min(self.cursor_col + 1),
            CursorMove::PrevChar => self.cursor_col = self.cursor_col.saturating_sub(1),
            CursorMove::NextWord => {
                self.cursor_col = match next_word(&self.value(), self.cursor_col) {
                    Some(i) => i,
                    None => self.len(),
                }
            }
            CursorMove::PrevWord => {
                self.cursor_col = match prev_word(&self.value(), self.cursor_col) {
                    Some(i) => i,
                    None => 0,
                }
            }
            CursorMove::LineHead | CursorMove::Head => self.cursor_col = 0,
            CursorMove::LineEnd | CursorMove::End => self.cursor_col = self.len(),
            CursorMove::NextLine | CursorMove::PrevLine => (),
        }
    }

    fn scroll_offset(&self) -> (usize, usize) {
        (self.scroll_offset_col, 0)
    }

    fn scroll(&mut self, cols: isize, _rows: isize) {
        self.scroll_offset_col = self
            .scroll_offset_col
            .saturating_add(cols as usize)
            .min(self.len());
    }

    fn insert_newline(&mut self) {}

    fn insert_char(&mut self, character: char) {
        let byte_index = self
            .value
            .char_indices()
            .nth(self.cursor_col)
            .map_or_else(|| self.value.len(), |(index, _)| index);

        self.value.insert(byte_index, character);
        self.move_cursor(CursorMove::NextChar);
    }

    fn delete_char(&mut self) {
        if self.cursor_col == 0 {
            return;
        };

        let byte_index = self
            .value
            .char_indices()
            .nth(self.cursor_col - 1)
            .map_or_else(|| self.value.len() - 1, |(index, _)| index);

        self.value.remove(byte_index);
        self.move_cursor(CursorMove::PrevChar);
    }

    fn delete_next_char(&mut self) {
        if self.cursor_col == self.value.len() {
            return;
        }

        let byte_index = self
            .value
            .char_indices()
            .nth(self.cursor_col)
            .map_or_else(|| self.value.len(), |(index, _)| index);

        self.value.remove(byte_index);
    }

    fn start_selection(&mut self) {
        self.selection_start = Some(self.cursor_col);
    }

    fn cancel_selection(&mut self) {
        self.selection_start = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod move_cursor {
        use super::*;
        #[test]
        fn next_char() {
            let mut input = OnelineInput {
                value: "test".to_string(),
                cursor_col: 0,
                selection_start: None,
            };
            input.move_cursor(CursorMove::NextChar);

            assert_eq!(input.cursor(), (1, 0))
        }

        #[test]
        fn prev_char() {
            let mut input = OnelineInput {
                value: "test".to_string(),
                cursor_col: 2,
                selection_start: None,
            };
            input.move_cursor(CursorMove::PrevChar);

            assert_eq!(input.cursor(), (1, 0))
        }

        #[test]
        fn next_word() {
            let mut input = OnelineInput {
                value: "hello $%^ world   ".to_string(),
                cursor_col: 1,
                selection_start: None,
            };
            input.move_cursor(CursorMove::NextWord);
            assert_eq!(input.cursor(), (6, 0));

            input.move_cursor(CursorMove::NextWord);
            assert_eq!(input.cursor(), (10, 0));

            input.move_cursor(CursorMove::NextWord);
            assert_eq!(input.cursor(), (18, 0));

            input.move_cursor(CursorMove::NextWord);
            assert_eq!(input.cursor(), (18, 0));
        }
        #[test]
        fn prev_word() {
            let mut input = OnelineInput {
                value: "   hello $%^ world".to_string(),
                cursor_col: 17,
                selection_start: None,
            };

            input.move_cursor(CursorMove::PrevWord);
            assert_eq!(input.cursor(), (13, 0));

            input.move_cursor(CursorMove::PrevWord);
            assert_eq!(input.cursor(), (9, 0));

            input.move_cursor(CursorMove::PrevWord);
            assert_eq!(input.cursor(), (3, 0));
        }

        #[test]
        fn head() {
            let mut input = OnelineInput {
                value: "".to_string(),
                cursor_col: 5,
                selection_start: None,
            };
            input.move_cursor(CursorMove::LineHead);

            assert_eq!(input.cursor(), (0, 0))
        }

        #[test]
        fn end() {
            let mut input = OnelineInput {
                value: "test".to_string(),
                cursor_col: 0,
                selection_start: None,
            };
            input.move_cursor(CursorMove::LineEnd);

            assert_eq!(input.cursor(), (4, 0))
        }

        #[test]
        fn next_line() {
            let mut input = OnelineInput {
                value: "".to_string(),
                cursor_col: 5,
                selection_start: None,
            };
            input.move_cursor(CursorMove::NextLine);

            assert_eq!(input.cursor(), (5, 0))
        }

        #[test]
        fn prev_line() {
            let mut input = OnelineInput {
                value: "".to_string(),
                cursor_col: 5,
                selection_start: None,
            };
            input.move_cursor(CursorMove::PrevLine);

            assert_eq!(input.cursor(), (5, 0))
        }
    }

    mod insert_char {
        use super::*;

        #[test]
        fn at_end() {
            let mut input = OnelineInput {
                value: "hello".to_string(),
                cursor_col: 5,
                selection_start: None,
            };
            input.insert_char(',');

            assert_eq!(input.value(), "hello,")
        }

        #[test]
        fn in_middle() {
            let mut input = OnelineInput {
                value: "hello".to_string(),
                cursor_col: 3,
                selection_start: None,
            };
            input.insert_char('-');

            assert_eq!(input.value(), "hel-lo")
        }
    }

    mod delete_char {
        use super::*;

        #[test]
        fn at_start() {
            let mut input = OnelineInput {
                value: "hello".to_string(),
                cursor_col: 0,
                selection_start: None,
            };
            input.delete_char();

            assert_eq!(input.value(), "hello")
        }

        #[test]
        fn in_middle() {
            let mut input = OnelineInput {
                value: "hello".to_string(),
                cursor_col: 2,
                selection_start: None,
            };
            input.delete_char();

            assert_eq!(input.value(), "hllo")
        }
    }

    mod delete_next_char {
        use super::*;

        #[test]
        fn at_end() {
            let mut input = OnelineInput {
                value: "hello".to_string(),
                cursor_col: 5,
                selection_start: None,
            };
            input.delete_next_char();

            assert_eq!(input.value(), "hello")
        }

        #[test]
        fn in_middle() {
            let mut input = OnelineInput {
                value: "hello".to_string(),
                cursor_col: 2,
                selection_start: None,
            };
            input.delete_next_char();

            assert_eq!(input.value(), "helo")
        }
    }
}
