use std::fmt;

use crossterm::event::Event;
use enum_iterator::Sequence;
use ratatui::widgets::ListState;
use reqwest::{
    blocking::{Client, Request},
    Method, Url,
};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Default, PartialEq, Sequence)]
pub enum CurrentPanel {
    #[default]
    Method,
    Url,
    Input,
    Output,
}

#[derive(Default, PartialEq, Sequence)]
pub enum CurrentInputType {
    #[default]
    Headers,
    Body,
}

impl fmt::Display for CurrentInputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrentInputType::Headers => write!(f, "Headers"),
            CurrentInputType::Body => write!(f, "Body"),
        }
    }
}

#[derive(Default, PartialEq, Sequence)]
pub enum CurrentInputField {
    #[default]
    Key,
    Value,
}

pub struct Model {
    pub cursor_col: u16,
    pub cursor_row: u16,
    pub current_panel: CurrentPanel,
    pub list_state: ListState,
    pub url_input: Input,
    pub current_input_type: CurrentInputType,
    pub current_input_field: CurrentInputField,
    pub headers_input: Vec<(Input, Input)>,
    pub body_input: Vec<(Input, Input)>,
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
            cursor_col: 0,
            cursor_row: 0,
            current_panel: CurrentPanel::default(),
            list_state: ListState::default().with_selected(Some(0)),
            url_input: Input::default(),
            current_input_type: CurrentInputType::default(),
            current_input_field: CurrentInputField::Key,
            headers_input: vec![(Input::default(), Input::default())],
            body_input: vec![(Input::default(), Input::default())],
            output_text: String::new(),
            exit: false,
        }
    }

    pub fn update_cursor(&mut self) {
        match self.current_panel {
            CurrentPanel::Method => {
                self.set_cursor(self.current_method().to_string().len() as u16, 0)
            }
            CurrentPanel::Url => self.set_cursor(self.url_input.visual_cursor() as u16, 0),
            CurrentPanel::Input => {
                let input_to_use = match self.current_input_type {
                    CurrentInputType::Headers => &self.headers_input,
                    CurrentInputType::Body => &self.body_input,
                };

                self.set_cursor(
                    match self.current_input_field {
                        CurrentInputField::Key => input_to_use[0].0.visual_cursor() as u16,
                        CurrentInputField::Value => input_to_use[0].1.visual_cursor() as u16,
                    },
                    0,
                );
            }
            CurrentPanel::Output => {
                let output_lines = self.output_text.lines();
                let (num_lines, last_line) =
                    output_lines.fold((0, None), |(count, _), elem| (count + 1, Some(elem)));

                self.set_cursor(1 + last_line.unwrap_or("").len() as u16, num_lines + 1);
            }
        };
    }

    pub fn next_panel(&mut self) {
        self.current_panel = self.current_panel.next().unwrap_or_default();
    }

    pub fn previous_panel(&mut self) {
        self.current_panel = self
            .current_panel
            .previous()
            .unwrap_or(CurrentPanel::last().unwrap());
    }

    pub fn next_method(&mut self) {
        let new_index = (self.list_state.selected().unwrap_or(0) + 1) % METHODS.len();
        self.list_state.select(Some(new_index));
    }

    pub fn previous_method(&mut self) {
        let new_index = self
            .list_state
            .selected()
            .unwrap_or(0)
            .checked_add_signed(-1)
            .unwrap_or(METHODS.len() - 1);
        self.list_state.select(Some(new_index));
    }

    pub fn current_method(&self) -> &Method {
        &METHODS[self.list_state.selected().unwrap_or(0)]
    }

    pub fn handle_url_input(&mut self, event: Event) {
        self.url_input.handle_event(&event);
    }

    pub fn next_input_type(&mut self) {
        self.current_input_type = self.current_input_type.next().unwrap_or_default();
        self.current_input_field = CurrentInputField::default();
    }

    pub fn previous_input_type(&mut self) {
        self.current_input_type = self
            .current_input_type
            .previous()
            .unwrap_or(CurrentInputType::last().unwrap());
        self.current_input_field = CurrentInputField::default();
    }

    pub fn focus_input_field(&mut self, input_field: CurrentInputField) {
        self.current_input_field = input_field;
    }

    pub fn handle_input_input(&mut self, event: Event) {
        match self.current_input_type {
            CurrentInputType::Headers => match self.current_input_field {
                CurrentInputField::Key => self.headers_input[0].0.handle_event(&event),
                CurrentInputField::Value => self.headers_input[0].1.handle_event(&event),
            },
            CurrentInputType::Body => match self.current_input_field {
                CurrentInputField::Key => self.body_input[0].0.handle_event(&event),
                CurrentInputField::Value => self.body_input[0].1.handle_event(&event),
            },
        };
    }

    pub fn submit_request(&mut self) {
        let url = Url::parse(&self.url_input.value()).expect("Invalid URL");

        match Client::new().execute(Request::new(self.current_method().clone(), url)) {
            Ok(response) => self.output_text = response.text().expect("Error unwrapping body"),
            Err(error) => self.output_text = format!("{:?}", error),
        };
    }

    fn set_cursor(&mut self, column: u16, row: u16) {
        self.cursor_col = column;
        self.cursor_row = row;
    }
}
