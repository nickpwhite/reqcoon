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
    Body,
    Output,
}

pub struct Model {
    pub current_panel: CurrentPanel,
    pub cursor_col: u16,
    pub cursor_row: u16,
    pub list_state: ListState,
    pub url_input: String,
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
            cursor_col: 0,
            cursor_row: 0,
            list_state: ListState::default().with_selected(Some(0)),
            url_input: String::new(),
            output_text: String::new(),
            exit: false
        }
    }

    pub fn next_panel(&mut self) {
        match self.current_panel {
            CurrentPanel::Method => self.current_panel = CurrentPanel::Url,
            CurrentPanel::Url => self.current_panel = CurrentPanel::Body,
            CurrentPanel::Body => self.current_panel = CurrentPanel::Output,
            CurrentPanel::Output => self.current_panel = CurrentPanel::Method,
        }
    }

    pub fn set_cursor(&mut self, column: u16, row: u16) {
        self.cursor_col = column;
        self.cursor_row = row;
    }

    pub fn previous_panel(&mut self) {
        match self.current_panel {
            CurrentPanel::Method => self.current_panel = CurrentPanel::Output,
            CurrentPanel::Url => self.current_panel = CurrentPanel::Method,
            CurrentPanel::Body => self.current_panel = CurrentPanel::Url,
            CurrentPanel::Output => self.current_panel = CurrentPanel::Body,
        }
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
        let url = Url::parse(&self.url_input).expect("Invalid URL");

        match Client::new().execute(Request::new(self.current_method().clone(), url)) {
            Ok(response) => self.output_text = response.text().expect("Error unwrapping body"),
            Err(error) => self.output_text = format!("{:?}", error),
        };
    }
}
