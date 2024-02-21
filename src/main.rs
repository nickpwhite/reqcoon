use std::{error::Error, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui_input::backend::crossterm::EventHandler;

mod model;
mod tui;
mod view;
use crate::{
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

    // Text input
    InputEvent(KeyEvent),

    // Input section
    NextInputType,
    PreviousInputType,

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
            if key.kind == KeyEventKind::Press {
                let panel_specific_handler = match model.current_panel {
                    CurrentPanel::Method => handle_method_key,
                    CurrentPanel::Url => handle_url_key,
                    CurrentPanel::Input => handle_input_key,
                    CurrentPanel::Output => handle_output_key,
                };

                globally_pre_handle_key(key)
                    .or_else(|| panel_specific_handler(key))
                    .or_else(|| globally_post_handle_key(key))
            } else {None}
        } else {None}
    } else {None}
}

fn handle_method_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::NextMethod),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::PreviousMethod),
        _ => None,
    }
}

fn handle_url_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Enter => Some(Message::SubmitRequest),
        _ => Some(Message::InputEvent(key))
    }
}

fn handle_input_key(key: KeyEvent) -> Option<Message> {
    match key.modifiers {
        KeyModifiers::SHIFT => match key.code {
            KeyCode::Right => Some(Message::NextInputType),
            KeyCode::Left => Some(Message::PreviousInputType),
            _ => None,
        },
        _ => None,
    }
}

fn handle_output_key(_key: KeyEvent) -> Option<Message> {
    None
}

fn globally_pre_handle_key(key: KeyEvent) -> Option<Message> {
    match key.modifiers {
        KeyModifiers::CONTROL => Some(Message::Quit),
        _ => match key.code {
            KeyCode::Tab => Some(Message::NextPanel),
            KeyCode::BackTab => Some(Message::PreviousPanel),
            _ => None,
        },
    }
}

fn globally_post_handle_key(_key: KeyEvent) -> Option<Message> {
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
        Message::NextInputType => {
            model.next_input_type();
        },
        Message::PreviousInputType => {
            model.previous_input_type();
        },
        Message::NextMethod => {
            model.next_method();
        },
        Message::PreviousMethod => {
            model.previous_method();
        },
        Message::InputEvent(key) => {
            model.url_input.handle_event(&Event::Key(key));
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
