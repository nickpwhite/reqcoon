use std::{error::Error, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use log::LevelFilter;

mod model;
mod text_wrapping;
mod tmux;
mod tui;
mod view;
use crate::{
    model::{Mode, Model, Panel},
    view::view,
};

#[derive(PartialEq)]
enum Message {
    // Mode
    EnterInsert,
    EnterNormal,

    // Navigation
    SelectPanel(Panel),
    SelectPanelLeft,
    SelectPanelDown,
    SelectPanelUp,
    SelectPanelRight,

    // Method input
    NextMethod,
    PreviousMethod,

    // URL input
    UrlInput(Event),

    // Input input
    NextInputType,
    PreviousInputType,
    NextInputField,
    PreviousInputField,
    InputInput(Event),

    // Submission
    SubmitRequest,

    Quit,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logging::log_to_file("debug.log", LevelFilter::Info)?;

    // setup terminal
    tui::install_panic_hook();
    let mut terminal = tui::init_terminal();
    let mut model = Model::new();

    while model.exit == false {
        match model.current_mode {
            Mode::Normal => tui::set_cursor_block(),
            Mode::Insert => tui::set_cursor_bar(),
        };

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
                    Panel::Method => handle_method_key,
                    Panel::Url => handle_url_key,
                    Panel::Input => handle_input_key,
                    Panel::Output => handle_output_key,
                };

                globally_pre_handle_key(key, model)
                    .or_else(|| panel_specific_handler(key, model))
                    .or_else(|| globally_post_handle_key(key, model))
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

fn globally_pre_handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    match model.current_mode {
        Mode::Normal => match key.modifiers {
            KeyModifiers::CONTROL => match key.code {
                KeyCode::Char('h') => Some(Message::SelectPanelLeft),
                KeyCode::Char('j') => Some(Message::SelectPanelDown),
                KeyCode::Char('k') => Some(Message::SelectPanelUp),
                KeyCode::Char('l') => Some(Message::SelectPanelRight),
                _ => None,
            },
            KeyModifiers::NONE => match key.code {
                KeyCode::Char('i') => Some(Message::EnterInsert),
                _ => None,
            },
            _ => None,
        },
        Mode::Insert => match key.modifiers {
            KeyModifiers::ALT => match key.code {
                KeyCode::Char('1') => Some(Message::SelectPanel(Panel::Method)),
                KeyCode::Char('2') => Some(Message::SelectPanel(Panel::Url)),
                KeyCode::Char('3') => Some(Message::SelectPanel(Panel::Input)),
                KeyCode::Char('4') => Some(Message::SelectPanel(Panel::Output)),
                _ => None,
            },
            KeyModifiers::CONTROL => match key.code {
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => Some(Message::Quit),
                _ => None,
            },
            KeyModifiers::NONE => match key.code {
                KeyCode::Esc => Some(Message::EnterNormal),
                _ => None,
            },
            _ => None,
        },
    }
}

fn handle_method_key(key: event::KeyEvent, _model: &Model) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::NextMethod),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::PreviousMethod),
        _ => None,
    }
}

fn handle_url_key(key: event::KeyEvent, _model: &Model) -> Option<Message> {
    match key.code {
        KeyCode::Enter => Some(Message::SubmitRequest),
        _ => Some(Message::UrlInput(Event::Key(key))),
    }
}

fn handle_input_key(key: event::KeyEvent, _model: &Model) -> Option<Message> {
    match key.code {
        KeyCode::Right if key.modifiers == KeyModifiers::SHIFT => Some(Message::NextInputType),
        KeyCode::Left if key.modifiers == KeyModifiers::SHIFT => Some(Message::PreviousInputType),
        KeyCode::Tab => Some(Message::NextInputField),
        KeyCode::BackTab => Some(Message::PreviousInputField),
        _ => Some(Message::InputInput(Event::Key(key))),
    }
}

fn handle_output_key(_key: event::KeyEvent, _model: &Model) -> Option<Message> {
    None
}

fn globally_post_handle_key(_key: event::KeyEvent, _model: &Model) -> Option<Message> {
    None
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::EnterInsert => model.enter_insert(),
        Message::EnterNormal => model.enter_normal(),
        Message::SelectPanel(panel) => model.select_panel(panel),
        Message::SelectPanelLeft => model.select_panel_left(),
        Message::SelectPanelDown => model.select_panel_down(),
        Message::SelectPanelUp => model.select_panel_up(),
        Message::SelectPanelRight => model.select_panel_right(),
        Message::NextMethod => model.next_method(),
        Message::PreviousMethod => model.previous_method(),
        Message::UrlInput(event) => model.handle_url_input(event),
        Message::NextInputType => model.next_input_type(),
        Message::PreviousInputType => model.previous_input_type(),
        Message::NextInputField => model.next_input_field(),
        Message::PreviousInputField => model.previous_input_field(),
        Message::InputInput(event) => model.handle_input_input(event),
        Message::SubmitRequest => model.submit_request(),
        Message::Quit => model.exit = true,
    };
    None
}
