use super::{next_word, prev_word, CursorMove, Input};

#[derive(Default)]
pub struct MultilineReadonlyInput {
    lines: Vec<String>,
    cursor_col: usize,
    cursor_row: usize,
    scroll_offset_col: usize,
    scroll_offset_row: usize,
    selection_start: Option<(usize, usize)>,
}

impl MultilineReadonlyInput {
    fn current_line(&self) -> &str {
        &self.lines[self.cursor_row]
    }
}

impl From<String> for MultilineReadonlyInput {
    fn from(item: String) -> Self {
        Self {
            lines: item.split('\n').map(str::to_string).collect(),
            cursor_col: 0,
            cursor_row: 0,
            scroll_offset_col: 0,
            scroll_offset_row: 0,
            selection_start: None,
        }
    }
}

impl Input for MultilineReadonlyInput {
    fn value(&self) -> String {
        self.lines.join("\n")
    }

    fn len(&self) -> usize {
        self.current_line().len()
    }

    fn cursor(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_row)
    }

    fn move_cursor(&mut self, cursor_move: CursorMove) {
        match cursor_move {
            CursorMove::NextChar => self.cursor_col = self.len().min(self.cursor_col + 1),
            CursorMove::PrevChar => self.cursor_col = self.cursor_col.saturating_sub(1),
            CursorMove::NextLine => {
                self.cursor_row = (self.lines.len() - 1).min(self.cursor_row + 1);
                self.cursor_col = self.len().min(self.cursor_col)
            }
            CursorMove::PrevLine => {
                self.cursor_row = self.cursor_row.saturating_sub(1);
                self.cursor_col = self.len().min(self.cursor_col)
            }
            CursorMove::NextWord => match next_word(self.current_line(), self.cursor_col) {
                Some(i) => self.cursor_col = i,
                None => {
                    if self.cursor_row < self.lines.len() - 1 {
                        self.cursor_row += 1;
                        self.cursor_col = match self
                            .current_line()
                            .char_indices()
                            .find(|(_, c)| !c.is_whitespace())
                        {
                            Some((i, _)) => i,
                            None => 0,
                        }
                    } else {
                        self.cursor_col = self.len();
                    }
                }
            },
            CursorMove::PrevWord => match prev_word(self.current_line(), self.cursor_col) {
                Some(i) => self.cursor_col = i,
                None => {
                    if self.cursor_row > 0 {
                        self.cursor_row -= 1;
                        self.cursor_col = match self
                            .current_line()
                            .char_indices()
                            .find(|(_, c)| !c.is_whitespace())
                        {
                            Some((i, _)) => i,
                            None => self.current_line().len(),
                        }
                    } else {
                        self.cursor_col = 0;
                    }
                }
            },
            CursorMove::LineHead => self.cursor_col = 0,
            CursorMove::LineEnd => self.cursor_col = self.len(),
            CursorMove::Head => {
                self.cursor_col = 0;
                self.cursor_row = 0;
            }
            CursorMove::End => {
                self.cursor_col = self.lines.last().map_or(0, |line| line.len());
                self.cursor_row = self.lines.len() - 1;
            }
        }
    }

    fn scroll_offset(&self) -> (usize, usize) {
        (self.scroll_offset_col, self.scroll_offset_row)
    }

    fn scroll(&mut self, cols: isize, rows: isize) {
        self.scroll_offset_col = self
            .scroll_offset_col
            .saturating_add(cols as usize)
            .min(self.len());
        self.scroll_offset_row = self
            .scroll_offset_row
            .saturating_add(rows as usize)
            .min(self.lines.len());
    }

    fn insert_newline(&mut self) {}

    fn insert_char(&mut self, _character: char) {}

    fn delete_char(&mut self) {}

    fn delete_next_char(&mut self) {}

    fn start_selection(&mut self) {
        self.selection_start = Some((self.cursor_col, self.cursor_row));
    }

    fn cancel_selection(&mut self) {
        self.selection_start = None;
    }
}
