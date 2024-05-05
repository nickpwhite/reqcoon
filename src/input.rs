use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum CursorMove {
    NextChar,
    PrevChar,
    NextWord,
    PrevWord,
    LineHead,
    LineEnd,
    Head,
    End,
    NextLine,
    PrevLine,
}

pub trait Input {
    fn value(&self) -> &str;
    fn len(&self) -> usize;
    fn cursor(&self) -> (usize, usize);
    fn move_cursor(&mut self, cursor_move: CursorMove);
    fn insert_newline(&mut self);
    fn insert_char(&mut self, character: char);
    fn delete_char(&mut self);
    fn delete_next_char(&mut self);

    fn handle_input(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => self.insert_newline(),
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => self.insert_char(c),
            KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => self.insert_char('\t'),
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => self.delete_char(),
            KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..
            } => self.move_cursor(CursorMove::PrevChar),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            } => self.move_cursor(CursorMove::NextLine),
            KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                ..
            } => self.move_cursor(CursorMove::PrevLine),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            } => self.move_cursor(CursorMove::NextLine),
            KeyEvent {
                code: KeyCode::Home,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => self.move_cursor(CursorMove::LineHead),
            KeyEvent {
                code: KeyCode::End,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => self.move_cursor(CursorMove::LineEnd),
            _ => todo!(),
        }
    }
}

pub struct OnelineInput {
    value: String,
    cursor_col: usize,
}

impl OnelineInput {
    fn next_word(&self) -> Option<(usize, char)> {
        let mut chars = self.value().chars().enumerate();
        let (_, c) = chars.nth(self.cursor().0)?;

        if c.is_whitespace() {
            chars.find(|(_, x)| !x.is_whitespace())
        } else {
            let mut whitespace = false;
            chars.find(|(_, x)| {
                if x.is_whitespace() {
                    whitespace = true;
                    false
                } else {
                    x.is_alphanumeric() != c.is_alphanumeric() || whitespace
                }
            })
        }
    }
}

impl Input for OnelineInput {
    fn value(&self) -> &str {
        &self.value
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
                self.cursor_col = match self.next_word() {
                    Some((i, _)) => i,
                    None => self.len(),
                }
            }
            CursorMove::PrevWord => self.cursor_col -= 1,
            CursorMove::LineHead | CursorMove::Head => self.cursor_col = 0,
            CursorMove::LineEnd | CursorMove::End => self.cursor_col = self.len(),
            CursorMove::NextLine | CursorMove::PrevLine => (),
        }
    }

    fn insert_newline(&mut self) {}

    fn insert_char(&mut self, character: char) {
        let byte_index = self
            .value
            .char_indices()
            .nth(self.cursor_col)
            .map_or_else(|| self.value.len(), |(index, _)| index);

        self.value.insert(byte_index, character);
    }

    fn delete_char(&mut self) {
        if self.cursor_col == 0 {
            return;
        };

        let byte_index = self
            .value
            .char_indices()
            .nth(self.cursor_col - 1)
            .map_or_else(|| self.value.len(), |(index, _)| index);

        self.value.remove(byte_index);
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
            };
            input.move_cursor(CursorMove::NextChar);

            assert_eq!(input.cursor(), (1, 0))
        }

        #[test]
        fn prev_char() {
            let mut input = OnelineInput {
                value: "test".to_string(),
                cursor_col: 2,
            };
            input.move_cursor(CursorMove::PrevChar);

            assert_eq!(input.cursor(), (1, 0))
        }

        #[test]
        fn next_word() {
            let mut input = OnelineInput {
                value: "hello $%^ world   ".to_string(),
                cursor_col: 1,
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
                value: "".to_string(),
                cursor_col: 1,
            };
            input.move_cursor(CursorMove::PrevWord);

            assert_eq!(input.cursor(), (0, 0))
        }

        #[test]
        fn head() {
            let mut input = OnelineInput {
                value: "".to_string(),
                cursor_col: 5,
            };
            input.move_cursor(CursorMove::LineHead);

            assert_eq!(input.cursor(), (0, 0))
        }

        #[test]
        fn end() {
            let mut input = OnelineInput {
                value: "test".to_string(),
                cursor_col: 0,
            };
            input.move_cursor(CursorMove::LineEnd);

            assert_eq!(input.cursor(), (4, 0))
        }

        #[test]
        fn next_line() {
            let mut input = OnelineInput {
                value: "".to_string(),
                cursor_col: 5,
            };
            input.move_cursor(CursorMove::NextLine);

            assert_eq!(input.cursor(), (5, 0))
        }

        #[test]
        fn prev_line() {
            let mut input = OnelineInput {
                value: "".to_string(),
                cursor_col: 5,
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
            };
            input.insert_char(',');

            assert_eq!(input.value(), "hello,")
        }

        #[test]
        fn in_middle() {
            let mut input = OnelineInput {
                value: "hello".to_string(),
                cursor_col: 3,
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
            };
            input.delete_char();

            assert_eq!(input.value(), "hello")
        }

        #[test]
        fn in_middle() {
            let mut input = OnelineInput {
                value: "hello".to_string(),
                cursor_col: 2,
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
            };
            input.delete_next_char();

            assert_eq!(input.value(), "hello")
        }

        #[test]
        fn in_middle() {
            let mut input = OnelineInput {
                value: "hello".to_string(),
                cursor_col: 2,
            };
            input.delete_next_char();

            assert_eq!(input.value(), "helo")
        }
    }
}
