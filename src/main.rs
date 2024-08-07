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
    Append,
    Insert,
    LeaveInsert,
    Normal,
    Visual,
    LeaveVisual,

    // Navigation
    SelectPanelLeft,
    SelectPanelDown,
    SelectPanelUp,
    SelectPanelRight,

    // Method input
    NextMethod,
    PreviousMethod,

    // Input
    Copy,
    InsertInput(KeyEvent),
    NormalInput(KeyEvent),

    // Input input
    NextInputType,
    PreviousInputType,
    NextInputField,
    PreviousInputField,
    NextInputFormat,
    PreviousInputFormat,

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
            Mode::Normal | Mode::Visual => tui::set_cursor_block(),
            Mode::Insert => tui::set_cursor_bar(),
        };

        terminal.draw(|f| view(f, &mut model))?;

        let mut current_message = handle_event(&mut model);

        while current_message.is_some() {
            current_message = update(&mut model, current_message.unwrap());
        }
    }

    tui::restore_terminal();
    model.to_file()?;

    Ok(())
}

fn handle_event(model: &mut Model) -> Option<Message> {
    if event::poll(Duration::from_millis(250)).expect("Unable to poll events") {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind == KeyEventKind::Press {
                match model.current_mode {
                    Mode::Normal => handle_normal_key(key, model),
                    Mode::Insert => handle_insert_key(key),
                    Mode::Visual => handle_visual_key(key),
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
    match key {
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(Message::Quit),
        KeyEvent {
            code: KeyCode::Esc, ..
        } => Some(Message::LeaveInsert),
        _ => Some(Message::InsertInput(key)),
    }
}

fn handle_visual_key(key: KeyEvent) -> Option<Message> {
    match key {
        KeyEvent {
            code: KeyCode::Esc, ..
        } => Some(Message::LeaveVisual),
        KeyEvent {
            code: KeyCode::Char('y'),
            ..
        } => Some(Message::Copy),
        _ => Some(Message::NormalInput(key)),
    }
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
            KeyCode::Char('a') => Some(Message::Append),
            KeyCode::Char('i') => Some(Message::Insert),
            KeyCode::Char('v') => Some(Message::Visual),
            _ => None,
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
        KeyCode::Right if key.modifiers == KeyModifiers::SHIFT | KeyModifiers::CONTROL => {
            Some(Message::NextInputFormat)
        }
        KeyCode::Left if key.modifiers == KeyModifiers::SHIFT | KeyModifiers::CONTROL => {
            Some(Message::PreviousInputFormat)
        }
        KeyCode::Tab => Some(Message::NextInputField),
        KeyCode::BackTab => Some(Message::PreviousInputField),
        _ => None,
    }
}

fn handle_normal_output_key(_key: KeyEvent) -> Option<Message> {
    None
}

fn globally_post_handle_normal_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Enter => Some(Message::SubmitRequest),
        _ => Some(Message::NormalInput(key)),
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Append => model.append(),
        Message::Insert => model.insert(),
        Message::LeaveInsert => {
            model.leave_insert();
            return Some(Message::Normal);
        }
        Message::Normal => model.normal(),
        Message::Visual => model.visual(),
        Message::LeaveVisual => {
            model.leave_visual();
            return Some(Message::Normal);
        }
        Message::SelectPanelLeft => model.select_panel_left(),
        Message::SelectPanelDown => model.select_panel_down(),
        Message::SelectPanelUp => model.select_panel_up(),
        Message::SelectPanelRight => model.select_panel_right(),
        Message::NextMethod => model.next_method(),
        Message::PreviousMethod => model.previous_method(),
        Message::Copy => {
            model.copy();
            return Some(Message::Normal);
        }
        Message::InsertInput(key_event) => model.handle_insert_input(key_event),
        Message::NormalInput(key_event) => model.handle_normal_input(key_event),
        Message::NextInputType => model.next_input_type(),
        Message::PreviousInputType => model.previous_input_type(),
        Message::NextInputField => model.next_input_field(),
        Message::PreviousInputField => model.previous_input_field(),
        Message::NextInputFormat => model.next_input_format(),
        Message::PreviousInputFormat => model.previous_input_format(),
        Message::SubmitRequest => model.submit_request(),
        Message::Quit => model.exit = true,
    };
    None
}
