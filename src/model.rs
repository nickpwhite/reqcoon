use std::fmt;

use ratatui::widgets::ListState;
use reqwest::{
    blocking::{Client, Request},
    Method,
    Url
};
use tui_input::Input;

pub enum CurrentInputType {
    Headers,
    Auth,
    Body,
}

impl fmt::Display for CurrentInputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrentInputType::Headers => write!(f, "Headers"),
            CurrentInputType::Auth => write!(f, "Auth"),
            CurrentInputType::Body => write!(f, "Body"),
        }
    }
}

#[derive(PartialEq)]
pub enum CurrentPanel {
    Method,
    Url,
    Input,
    Output,
}

pub struct Model {
    pub current_panel: CurrentPanel,
    pub current_input_type: CurrentInputType,
    pub cursor_col: u16,
    pub cursor_row: u16,
    pub list_state: ListState,
    pub url_input: Input,
    pub headers_input: Vec<(Input, Input)>,
    pub current_header_key: Input,
    pub current_header_value: Input,
    pub body_input: Input,
    pub output_text: String,
    pub exit: bool,
}

pub const METHODS: [Method; 5] = [
    Method::GET,
    Method::HEAD,
    Method::POST,
    Method::PUT,
    Method::DELETE,
];

impl Model {
    pub fn new() -> Model {
        Model {
            current_panel: CurrentPanel::Method,
            current_input_type: CurrentInputType::Headers,
            cursor_col: 0,
            cursor_row: 0,
            list_state: ListState::default().with_selected(Some(0)),
            url_input: Input::default(),
            headers_input: Vec::new(),
            current_header_key: Input::default(),
            current_header_value: Input::default(),
            body_input: Input::default(),
            output_text: String::new(),
            exit: false
        }
    }

    pub fn next_panel(&mut self) {
        match self.current_panel {
            CurrentPanel::Method => self.current_panel = CurrentPanel::Url,
            CurrentPanel::Url => self.current_panel = CurrentPanel::Input,
            CurrentPanel::Input => self.current_panel = CurrentPanel::Output,
            CurrentPanel::Output => self.current_panel = CurrentPanel::Method,
        }
    }

    pub fn previous_panel(&mut self) {
        match self.current_panel {
            CurrentPanel::Method => self.current_panel = CurrentPanel::Output,
            CurrentPanel::Url => self.current_panel = CurrentPanel::Method,
            CurrentPanel::Input => self.current_panel = CurrentPanel::Url,
            CurrentPanel::Output => self.current_panel = CurrentPanel::Input,
        }
    }

    pub fn next_input_type(&mut self) {
        match self.current_input_type {
            CurrentInputType::Headers => self.current_input_type = CurrentInputType::Auth,
            CurrentInputType::Auth => self.current_input_type = CurrentInputType::Body,
            CurrentInputType::Body => self.current_input_type = CurrentInputType::Headers,
        }
    }

    pub fn previous_input_type(&mut self) {
        match self.current_input_type {
            CurrentInputType::Headers => self.current_input_type = CurrentInputType::Body,
            CurrentInputType::Auth => self.current_input_type = CurrentInputType::Headers,
            CurrentInputType::Body => self.current_input_type = CurrentInputType::Auth,
        }
    }

    pub fn set_cursor(&mut self, column: u16, row: u16) {
        self.cursor_col = column;
        self.cursor_row = row;
    }

    pub fn next_method(&mut self) {
        let new_index = (self.list_state.selected().unwrap_or(0) + 1) % METHODS.len();
        self.list_state.select(Some(new_index));
    }

    pub fn previous_method(&mut self) {
        let new_index = self.list_state.selected().unwrap_or(0).checked_add_signed(-1).unwrap_or(METHODS.len() - 1);
        self.list_state.select(Some(new_index));
    }

    pub fn current_method(&self) -> &Method {
        &METHODS[self.list_state.selected().unwrap_or(0)]
    }

    pub fn submit_request(&mut self) {
        let url = Url::parse(&self.url_input.value()).expect("Invalid URL");

        match Client::new().execute(Request::new(self.current_method().clone(), url)) {
            Ok(response) => self.output_text = response.text().expect("Error unwrapping body"),
            Err(error) => self.output_text = format!("{:?}", error),
        };
    }
}
