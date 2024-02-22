use std::{error::Error, time::Duration};

use crossterm::event::{self, Event, KeyCode};

mod component;
mod model;
mod tui;
mod view;
use crate::{
    component::{
        app::App,
        Component,
    },
    model::{Model, CurrentPanel},
    view::view,
};

#[derive(PartialEq)]
enum Message {
    // Navigation
    NextPanel,
    PreviousPanel,

    // Method input
    NextMethod,
    PreviousMethod,

    // URL input
    BackspaceUrlChar,
    TypeUrlChar(char),

    // Submission
    SubmitRequest,

    Quit,
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    tui::install_panic_hook();
    let mut terminal = tui::init_terminal();
    let mut app = App::default();

    while app.exit == false {
        terminal.draw(|f| f.render_widget(&app, f.size()))?;

        app = app.handle_event(get_event());
    }

    tui::restore_terminal();
    Ok(())
}

fn get_event() -> Option<Event> {
    if event::poll(Duration::from_millis(250)).expect("Unable to poll events") {
        if let Ok(event) = event::read() {
            return Some(event);
        }
    };

    None
}

fn handle_event(model: &Model) -> Option<Message> {
    if event::poll(Duration::from_millis(250)).expect("Unable to poll events") {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind == event::KeyEventKind::Press {
                let panel_specific_handler = match model.current_panel {
                    CurrentPanel::Method => handle_method_key,
                    CurrentPanel::Url => handle_url_key,
                    CurrentPanel::Body => handle_body_key,
                    CurrentPanel::Output => handle_output_key,
                };

                globally_pre_handle_key(key)
                    .or_else(|| panel_specific_handler(key))
                    .or_else(|| globally_post_handle_key(key))
            } else {None}
        } else {None}
    } else {None}
}

fn handle_method_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::NextMethod),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::PreviousMethod),
        _ => None,
    }
}

fn handle_url_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char(c) => Some(Message::TypeUrlChar(c)),
        KeyCode::Backspace => Some(Message::BackspaceUrlChar),
        KeyCode::Enter => Some(Message::SubmitRequest),
        _ => None
    }
}

fn handle_body_key(_key: event::KeyEvent) -> Option<Message> {
    None
}

fn handle_output_key(_key: event::KeyEvent) -> Option<Message> {
    None
}

fn globally_pre_handle_key(key: event::KeyEvent) -> Option<Message> {
    // eprintln!("{:?}", key);
    match key.code {
        KeyCode::Tab => Some(Message::NextPanel),
        KeyCode::BackTab => Some(Message::PreviousPanel),
        _ => None,
    }
}

fn globally_post_handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::NextPanel => {
            model.next_panel();
        },
        Message::PreviousPanel => {
            model.previous_panel();
        },
        Message::NextMethod => {
            model.next_method();
        },
        Message::PreviousMethod => {
            model.previous_method();
        },
        Message::BackspaceUrlChar => {
            model.url_input.pop();
        },
        Message::TypeUrlChar(value) => {
            model.url_input.push(value);
        },
        Message::SubmitRequest => {
            model.submit_request();
        }
        Message::Quit => {
            model.exit = true;
        }
    }
    None
}
