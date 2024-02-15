use std::{error::Error, time::Duration};

use crossterm::event::{self, Event, KeyCode};

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
            if key.kind == event::KeyEventKind::Press {
                let panel_specific_handler = match model.current_panel {
                    CurrentPanel::Method => handle_method_key,
                    CurrentPanel::Url => handle_url_key,
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

// fn run_app<B: Backend>(
//     terminal: &mut Terminal<B>,
//     model: &mut Model,
// ) -> io::Result<bool> {
//     loop {

//         if let Event::Key(key) = event::read()? {
//             if key.kind == event::KeyEventKind::Release {
//                 // Skip events that are not KeyEventKind::Press
//                 continue;
//             }
//             match model.current_screen {
//                 CurrentScreen::Main => match key.code {
//                     KeyCode::Char('e') => {
//                         model.current_screen = CurrentScreen::Editing;
//                         model.currently_editing = Some(CurrentlyEditing::Key);
//                     }
//                     KeyCode::Char('q') => {
//                         model.current_screen = CurrentScreen::Exiting;
//                     }
//                     _ => {}
//                 },
//                 CurrentScreen::Exiting => match key.code {
//                     KeyCode::Char('y') => {
//                         return Ok(true);
//                     }
//                     KeyCode::Char('n') | KeyCode::Char('q') => {
//                         return Ok(false);
//                     }
//                     _ => {}
//                 },
//                 CurrentScreen::Editing if key.kind == KeyEventKind::Press => {
//                     match key.code {
//                         KeyCode::Enter => {
//                             if let Some(editing) = &model.currently_editing {
//                                 match editing {
//                                     CurrentlyEditing::Key => {
//                                         model.currently_editing =
//                                             Some(CurrentlyEditing::Value);
//                                     }
//                                     CurrentlyEditing::Value => {
//                                         model.save_key_value();
//                                         model.current_screen =
//                                             CurrentScreen::Main;
//                                     }
//                                 }
//                             }
//                         }
//                         KeyCode::Backspace => {
//                             if let Some(editing) = &model.currently_editing {
//                                 match editing {
//                                     CurrentlyEditing::Key => {
//                                         model.key_input.pop();
//                                     }
//                                     CurrentlyEditing::Value => {
//                                         model.value_input.pop();
//                                     }
//                                 }
//                             }
//                         }
//                         KeyCode::Esc => {
//                             model.current_screen = CurrentScreen::Main;
//                             model.currently_editing = None;
//                         }
//                         KeyCode::Tab => {
//                             model.toggle_editing();
//                         }
//                         KeyCode::Char(value) => {
//                             if let Some(editing) = &model.currently_editing {
//                                 match editing {
//                                     CurrentlyEditing::Key => {
//                                         model.key_input.push(value);
//                                     }
//                                     CurrentlyEditing::Value => {
//                                         model.value_input.push(value);
//                                     }
//                                 }
//                             }
//                         }
//                         _ => {}
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }
// }

// fn read_input(prompt: &str) -> String {
//     print!("{}> ", prompt);
//     io::stdout().flush().expect("Unable to flush write buffer");
//     let mut method = String::new();
//     io::stdin().read_line(&mut method).expect("Unable to read");

//     String::from(method.trim())
// }


// fn handle_events() -> io::Result<bool> {
//     if event::poll(std::time::Duration::from_millis(50))? {
//         if let Event::Key(key) = event::read()? {
//             if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
//                 return Ok(true);
//             }
//         }
//     }
//     Ok(false)
// }

// fn ui(frame: &mut Frame) {
//     let outer_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints(vec![
//             Constraint::Percentage(10),
//             Constraint::Percentage(30),
//             Constraint::Percentage(30),
//             Constraint::Percentage(30),
//         ])
//         .split(frame.size());

//     let inner_layout = Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints(vec![
//             Constraint::Percentage(20),
//             Constraint::Percentage(80),
//         ])
//         .split(outer_layout[0]);
//     frame.render_widget(
//         Paragraph::new("Inner 0")
//             .block(Block::default().title("Method").borders(Borders::ALL)),
//         inner_layout[0]
//     );
//     frame.render_widget(
//         Paragraph::new("Inner 1")
//             .block(Block::default().title("URL").borders(Borders::ALL)),
//         inner_layout[1]
//     );
//     frame.render_widget(
//         Paragraph::new("Outer 1")
//             .block(Block::default().title("Headers").borders(Borders::ALL)),
//         outer_layout[1]
//     );
//     frame.render_widget(
//         Paragraph::new("Outer 2")
//             .block(Block::default().title("Body").borders(Borders::ALL)),
//         outer_layout[2]
//     );
// }
