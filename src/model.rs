use ratatui::widgets::ListState;
use reqwest::{
    blocking::{Client, Request},
    Method,
    Url
};

#[derive(PartialEq)]
pub enum CurrentPanel {
    Method,
    Url,
    Output,
}

pub struct CursorPosition {
    pub x: u16,
    pub y: u16,
}

impl CursorPosition {
    pub fn new(x: u16, y: u16) -> CursorPosition {
        CursorPosition {
            x: x,
            y: y,
        }
    }
}

pub struct Model {
    pub current_panel: CurrentPanel,
    pub current_cursor_position: CursorPosition,
    pub method_input: Method,
    pub list_state: ListState,
    pub url_input: String,
    pub output_text: String,
    pub exit: bool,
}

impl Model {
    pub fn new() -> Model {
        Model {
            current_panel: CurrentPanel::Method,
            current_cursor_position: CursorPosition::new(4, 1),
            method_input: Method::GET,
            list_state: ListState::default().with_selected(Some(0)),
            url_input: String::new(),
            output_text: String::new(),
            exit: false
        }
    }

    pub fn next_panel(&mut self) {
        match self.current_panel {
            CurrentPanel::Method => self.current_panel = CurrentPanel::Url,
            CurrentPanel::Url => self.current_panel = CurrentPanel::Output,
            CurrentPanel::Output => self.current_panel = CurrentPanel::Method,
        }
    }

    pub fn previous_panel(&mut self) {
        match self.current_panel {
            CurrentPanel::Method => self.current_panel = CurrentPanel::Output,
            CurrentPanel::Url => self.current_panel = CurrentPanel::Method,
            CurrentPanel::Output => self.current_panel = CurrentPanel::Url,
        }
    }

    pub fn next_method(&mut self) {
        let new_index = self.list_state.selected().unwrap_or(0) + 1;
        self.list_state.select(Some(new_index));
    }

    pub fn previous_method(&mut self) {
        let new_index = self.list_state.selected().unwrap_or(0) - 1;
        self.list_state.select(Some(new_index));
    }

    pub fn submit_request(&mut self) {
        let url = Url::parse(&self.url_input).expect("Invalid URL");

        match Client::new().execute(Request::new(self.method_input.clone(), url)) {
            Ok(response) => self.output_text = response.text().expect("Error unwrapping body"),
            Err(error) => self.output_text = format!("{:?}", error),
        };
    }
}
