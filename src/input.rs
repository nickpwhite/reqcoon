use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub mod dummy_input;
pub mod multiline_readonly_input;
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
    fn value(&self) -> String;
    fn len(&self) -> usize;
    fn cursor(&self) -> (usize, usize);
    fn move_cursor(&mut self, cursor_move: CursorMove);
    fn scroll_offset(&self) -> (usize, usize);
    fn scroll(&mut self, cols: isize, lines: isize);
    fn insert_newline(&mut self);
    fn insert_char(&mut self, character: char);
    fn delete_char(&mut self);
    fn delete_next_char(&mut self);
    fn start_selection(&mut self);
    fn cancel_selection(&mut self);

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
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => self.delete_next_char(),
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
            } => self.move_cursor(CursorMove::Head),
            KeyEvent {
                code: KeyCode::End,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => self.move_cursor(CursorMove::End),
            _ => (),
        }
    }

    fn is_empty(&self) -> bool {
        self.value().is_empty()
    }
}

fn next_word(line: &str, cursor_col: usize) -> Option<usize> {
    let mut chars = line.char_indices();
    let (_, c) = chars.nth(cursor_col)?;

    if c.is_whitespace() {
        chars.find(|(_, x)| !x.is_whitespace()).map(|(i, _)| i)
    } else {
        let mut whitespace = false;
        chars
            .find(|(_, x)| {
                if x.is_whitespace() {
                    whitespace = true;
                    false
                } else {
                    x.is_alphanumeric() != c.is_alphanumeric() || whitespace
                }
            })
            .map(|(i, _)| i)
    }
}

fn prev_word(line: &str, cursor_col: usize) -> Option<usize> {
    if cursor_col == 0 {
        return None;
    }

    let mut chars = line
        .char_indices()
        .rev()
        .skip(line.len().saturating_sub(cursor_col));
    let mut target = chars.next()?;

    let mut non_whitespace = !target.1.is_whitespace();
    for (i, c) in chars {
        if (non_whitespace && c.is_whitespace())
            || (!target.1.is_whitespace() && c.is_alphanumeric() != target.1.is_alphanumeric())
        {
            break;
        } else {
            non_whitespace = non_whitespace || !c.is_whitespace();
            target = (i, c);
        }
    }

    Some(target.0)
}
