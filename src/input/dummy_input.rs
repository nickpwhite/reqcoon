use super::{CursorMove, Input};

#[derive(Default)]
pub struct DummyInput {}

impl Input for DummyInput {
    fn value(&self) -> &str {
        ""
    }
    fn len(&self) -> usize {
        0
    }
    fn cursor(&self) -> (usize, usize) {
        (0, 0)
    }
    fn move_cursor(&mut self, cursor_move: CursorMove) {}
    fn insert_newline(&mut self) {}
    fn insert_char(&mut self, character: char) {}
    fn delete_char(&mut self) {}
    fn delete_next_char(&mut self) {}
}
