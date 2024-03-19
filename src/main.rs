use std::{error::Error, time::Duration};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    filename: String,
}

#[derive(PartialEq)]
enum Message {
    // Mode
    EnterInsert,
    EnterNormal,

    // Navigation
    SelectPanelLeft,
    SelectPanelDown,
    SelectPanelUp,
    SelectPanelRight,

    // Method input
    NextMethod,
    PreviousMethod,

    // Input
    InsertInput(Event),
    NormalInput(KeyEvent),

    // Input input
    NextInputType,
    PreviousInputType,
    NextInputField,
    PreviousInputField,

    // Submission
    SubmitRequest,

    Quit,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logging::log_to_file("debug.log", LevelFilter::Info)?;

    let args = Args::parse();

    // setup terminal
    tui::install_panic_hook();
    let mut terminal = tui::init_terminal();
    let mut model = Model::from_file(args.filename.clone()).unwrap_or(Model::new(args.filename));

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

    model.to_file()?;

    tui::restore_terminal();
    Ok(())
}

fn handle_event(model: &Model) -> Option<Message> {
    if event::poll(Duration::from_millis(250)).expect("Unable to poll events") {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind == KeyEventKind::Press {
                match model.current_mode {
                    Mode::Normal => handle_normal_key(key, model),
                    Mode::Insert => handle_insert_key(key),
                }
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

fn handle_normal_key(key: KeyEvent, model: &Model) -> Option<Message> {
    let panel_specific_handler = match model.current_panel {
        Panel::Method => handle_normal_method_key,
        Panel::Url => handle_normal_url_key,
        Panel::Input => handle_normal_input_key,
        Panel::Output => handle_normal_output_key,
    };

    globally_pre_handle_normal_key(key)
        .or_else(|| panel_specific_handler(key))
        .or_else(|| globally_post_handle_normal_key(key))
}

fn handle_insert_key(key: KeyEvent) -> Option<Message> {
    globally_pre_handle_insert_key(key).or_else(|| globally_post_handle_insert_key(key))
}

fn globally_pre_handle_normal_key(key: KeyEvent) -> Option<Message> {
    match key.modifiers {
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
    }
}

fn globally_pre_handle_insert_key(key: KeyEvent) -> Option<Message> {
    match key.modifiers {
        KeyModifiers::CONTROL => match key.code {
            KeyCode::Char('c') => Some(Message::Quit),
            _ => None,
        },
        KeyModifiers::NONE => match key.code {
            KeyCode::Esc => Some(Message::EnterNormal),
            _ => Some(Message::InsertInput(Event::Key(key))),
        },
        _ => None,
    }
}

fn handle_normal_method_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Message::NextMethod),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::PreviousMethod),
        _ => None,
    }
}

fn handle_normal_url_key(_key: KeyEvent) -> Option<Message> {
    None
}

fn handle_normal_input_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Right if key.modifiers == KeyModifiers::SHIFT => Some(Message::NextInputType),
        KeyCode::Left if key.modifiers == KeyModifiers::SHIFT => Some(Message::PreviousInputType),
        KeyCode::Tab => Some(Message::NextInputField),
        KeyCode::BackTab => Some(Message::PreviousInputField),
        _ => None,
    }
}

fn handle_normal_output_key(_key: KeyEvent) -> Option<Message> {
    None
}

fn globally_post_handle_insert_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Enter => Some(Message::SubmitRequest),
        _ => None,
    }
}

fn globally_post_handle_normal_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Enter => Some(Message::SubmitRequest),
        _ => Some(Message::NormalInput(key)),
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::EnterInsert => model.enter_insert(),
        Message::EnterNormal => model.enter_normal(),
        Message::SelectPanelLeft => model.select_panel_left(),
        Message::SelectPanelDown => model.select_panel_down(),
        Message::SelectPanelUp => model.select_panel_up(),
        Message::SelectPanelRight => model.select_panel_right(),
        Message::NextMethod => model.next_method(),
        Message::PreviousMethod => model.previous_method(),
        Message::InsertInput(event) => model.handle_insert_input(event),
        Message::NormalInput(key_event) => model.handle_normal_input(key_event),
        Message::NextInputType => model.next_input_type(),
        Message::PreviousInputType => model.previous_input_type(),
        Message::NextInputField => model.next_input_field(),
        Message::PreviousInputField => model.previous_input_field(),
        Message::SubmitRequest => model.submit_request(),
        Message::Quit => model.exit = true,
    };
    None
}
