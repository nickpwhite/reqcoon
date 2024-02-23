use std::{error::Error, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

mod model;
mod tui;
mod view;
use crate::{
    model::{CurrentPanel, Model},
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
    UrlInput(Event),

    // Submission
    SubmitRequest,

    Quit,
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    tui::install_panic_hook();
    let mut terminal = tui::init_terminal();
    let mut model = Model::new();

    while model.exit == false {
        model.update_cursor();
        terminal.draw(|f| view(f, &mut model))?;

        let mut current_message = handle_event(&model);

        while current_message.is_some() {
            current_message = update(&mut model, current_message.unwrap());
        }
    }

    tui::restore_terminal();
    Ok(())
}

fn handle_event(model: &Model) -> Option<Message> {
    if event::poll(Duration::from_millis(250)).expect("Unable to poll events") {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind == event::KeyEventKind::Press {
                let panel_specific_handler = match model.current_panel {
                    CurrentPanel::Method => handle_method_key,
                    CurrentPanel::Url => handle_url_key,
                    CurrentPanel::Input => handle_input_key,
                    CurrentPanel::Output => handle_output_key,
                };

                globally_pre_handle_key(key)
                    .or_else(|| panel_specific_handler(key))
                    .or_else(|| globally_post_handle_key(key))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn globally_pre_handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Tab => Some(Message::NextPanel),
        KeyCode::BackTab => Some(Message::PreviousPanel),
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => Some(Message::Quit),
        _ => None,
    }
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
        KeyCode::Enter => Some(Message::SubmitRequest),
        _ => Some(Message::UrlInput(Event::Key(key))),
    }
}

fn handle_input_key(_key: event::KeyEvent) -> Option<Message> {
    None
}

fn handle_output_key(_key: event::KeyEvent) -> Option<Message> {
    None
}

fn globally_post_handle_key(key: event::KeyEvent) -> Option<Message> {
    None
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
        Message::UrlInput(event) => {
            model.handle_url_input(event);
        },
        Message::SubmitRequest => {
            model.submit_request();
        },
        Message::Quit => {
            model.exit = true;
        },
    };
    None
}
