use super::{CursorMove, Input};

#[derive(Default)]
pub struct DummyInput {}

impl Input for DummyInput {
    fn value(&self) -> String {
        String::default()
    }
    fn len(&self) -> usize {
        usize::default()
    }
    fn cursor(&self) -> (usize, usize) {
        (usize::default(), usize::default())
    }
    fn move_cursor(&mut self, _cursor_move: CursorMove) {}
    fn insert_newline(&mut self) {}
    fn insert_char(&mut self, _character: char) {}
    fn delete_char(&mut self) {}
    fn delete_next_char(&mut self) {}
    fn start_selection(&mut self) {}
    fn cancel_selection(&mut self) {}
}
