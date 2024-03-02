use crossterm::{
    cursor::SetCursorStyle,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::{io::stdout, panic};

pub fn init_terminal() -> Terminal<impl Backend> {
    enable_raw_mode().unwrap();
    stdout().execute(EnterAlternateScreen).unwrap();

    Terminal::new(CrosstermBackend::new(stdout())).expect("Unable to create terminal")
}

pub fn restore_terminal() {
    stdout().execute(LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}

pub fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        stdout().execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}

pub fn set_cursor_block() {
    stdout().execute(SetCursorStyle::BlinkingBlock).unwrap();
}

pub fn set_cursor_bar() {
    stdout().execute(SetCursorStyle::BlinkingBar).unwrap();
}
