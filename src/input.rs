use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub mod dummy_input;
pub mod oneline_input;

#[derive(Debug)]
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
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            } => self.move_cursor(CursorMove::NextChar),
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

    fn is_empty(&self) -> bool {
        self.value().is_empty()
    }
}
