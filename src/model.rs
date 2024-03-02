use std::{fmt, vec};

use crossterm::event::Event;
use enum_iterator::Sequence;
use json::object;
use nonempty::{nonempty, NonEmpty};
use ratatui::widgets::{ListState};
use reqwest::{blocking::Client, Method, Url};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::tmux::{select_tmux_panel, Direction};

#[derive(Default, PartialEq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "Normal"),
            Mode::Insert => write!(f, "Insert"),
        }
    }
}

#[derive(Default, PartialEq, Sequence)]
pub enum Panel {
    #[default]
    Method,
    Url,
    Input,
    Output,
}

#[derive(Default, PartialEq, Sequence)]
pub enum InputType {
    #[default]
    Headers,
    Body,
}

impl fmt::Display for InputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputType::Headers => write!(f, "Headers"),
            InputType::Body => write!(f, "Body"),
        }
    }
}

#[derive(Default, PartialEq, Sequence)]
pub enum InputField {
    #[default]
    Key,
    Value,
}

#[derive(Default)]
pub struct InputRow {
    key: Input,
    value: Input,
}

impl InputRow {
    fn is_empty(&self) -> bool {
        self.key.value().is_empty() && self.value.value().is_empty()
    }
}

impl IntoIterator for &InputRow {
    type Item = String;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            String::from(self.key.value()),
            String::from(self.value.value()),
        ]
        .into_iter()
    }
}

pub const METHODS: [Method; 5] = [
    Method::GET,
    Method::HEAD,
    Method::POST,
    Method::PUT,
    Method::DELETE,
];

pub struct Model {
    pub cursor_col: u16,
    pub cursor_row: u16,
    pub current_mode: Mode,
    pub current_panel: Panel,
    pub list_state: ListState,
    pub url_input: Input,
    pub current_input_type: InputType,
    pub current_input_field: InputField,
    pub input_index: usize,
    pub headers_input_table: NonEmpty<InputRow>,
    pub body_input_table: NonEmpty<InputRow>,
    pub output_text: String,
    pub exit: bool,
}

impl Model {
    pub fn new() -> Model {
        Model {
            cursor_col: 2,
            cursor_row: 0,
            current_mode: Mode::default(),
            current_panel: Panel::default(),
            list_state: ListState::default().with_selected(Some(0)),
            url_input: Input::default(),
            current_input_type: InputType::default(),
            current_input_field: InputField::Key,
            input_index: 0,
            headers_input_table: nonempty![InputRow::default()],
            body_input_table: nonempty![InputRow::default()],
            output_text: String::new(),
            exit: false,
        }
    }

    pub fn update_cursor(&mut self) {
        match self.current_panel {
            Panel::Method => self.set_cursor(self.current_method().to_string().len() as u16, 0),
            Panel::Url => self.set_cursor(self.url_input.visual_cursor() as u16, 0),
            Panel::Input => {
                self.set_cursor(
                    self.current_input().visual_cursor() as u16,
                    self.input_index as u16,
                );
            }
            Panel::Output => {
                let output_lines = self.output_text.lines();
                let (num_lines, last_line) =
                    output_lines.fold((0, None), |(count, _), elem| (count + 1, Some(elem)));

                self.set_cursor(1 + last_line.unwrap_or("").len() as u16, num_lines + 1);
            }
        };
    }

    pub fn enter_insert(&mut self) {
        self.current_mode = Mode::Insert;
    }

    pub fn enter_normal(&mut self) {
        self.current_mode = Mode::Normal;
    }

    pub fn select_panel(&mut self, panel: Panel) {
        self.current_panel = panel;
        self.update_cursor();
    }

    pub fn select_panel_left(&mut self) {
        match self.current_panel {
            Panel::Url => {
                self.current_panel = Panel::Method;
                self.update_cursor();
            }
            _ => select_tmux_panel(Direction::Left),
        };
    }

    pub fn select_panel_down(&mut self) {
        match self.current_panel {
            Panel::Url | Panel::Method => {
                self.current_panel = Panel::Input;
                self.update_cursor();
            }
            Panel::Input => {
                self.current_panel = Panel::Output;
                self.update_cursor();
            }
            _ => select_tmux_panel(Direction::Down),
        };
    }

    pub fn select_panel_up(&mut self) {
        match self.current_panel {
            Panel::Output => {
                self.current_panel = Panel::Input;
                self.update_cursor();
            }
            Panel::Input => {
                self.current_panel = Panel::Url;
                self.update_cursor();
            }
            _ => select_tmux_panel(Direction::Up),
        };
    }

    pub fn select_panel_right(&mut self) {
        match self.current_panel {
            Panel::Method => {
                self.current_panel = Panel::Url;
                self.update_cursor();
            }
            _ => select_tmux_panel(Direction::Right),
        };
    }

    pub fn next_method(&mut self) {
        let new_index = (self.list_state.selected().unwrap_or(0) + 1) % METHODS.len();
        self.list_state.select(Some(new_index));
        self.update_cursor();
    }

    pub fn previous_method(&mut self) {
        let new_index = self
            .list_state
            .selected()
            .unwrap_or(0)
            .checked_add_signed(-1)
            .unwrap_or(METHODS.len() - 1);
        self.list_state.select(Some(new_index));
        self.update_cursor();
    }

    pub fn current_method(&self) -> &Method {
        &METHODS[self.list_state.selected().unwrap_or(0)]
    }

    pub fn handle_url_input(&mut self, event: Event) {
        self.url_input.handle_event(&event);
        self.update_cursor();
    }

    pub fn next_input_type(&mut self) {
        self.current_input_type = self.current_input_type.next().unwrap_or_default();
        self.current_input_field = InputField::default();
        self.input_index = self.current_input_table().len() - 1;
        self.update_cursor();
    }

    pub fn previous_input_type(&mut self) {
        self.current_input_type = self
            .current_input_type
            .previous()
            .unwrap_or(InputType::last().unwrap());
        self.current_input_field = InputField::default();
        self.input_index = self.current_input_table().len() - 1;
        self.update_cursor();
    }

    pub fn next_input_field(&mut self) {
        if self.current_input_field == InputField::last().unwrap() {
            if !self.current_input_table().last().is_empty() {
                self.current_input_table_mut().push(InputRow::default());
            }
            if self.input_index < self.current_input_table().len() - 1 {
                self.input_index += 1
            }
        }
        self.current_input_field = self.current_input_field.next().unwrap_or_default();
        self.update_cursor();
    }

    pub fn previous_input_field(&mut self) {
        if self.current_input_field == InputField::first().unwrap() {
            if self.input_index == 0 {
                self.input_index = self.current_input_table().len() - 1;
            } else {
                self.input_index -= 1;
            }
        }
        self.current_input_field = self
            .current_input_field
            .previous()
            .unwrap_or(InputField::last().unwrap());
        self.update_cursor();
    }

    pub fn handle_input_input(&mut self, event: Event) {
        self.current_input_mut().handle_event(&event);
        self.update_cursor();
    }

    pub fn submit_request(&mut self) {
        let url = Url::parse(&self.url_input.value()).expect("Invalid URL");
        let mut request_builder = Client::new().request(self.current_method().clone(), url);

        request_builder = self
            .headers_input_table
            .iter()
            .filter(|header| !header.key.value().is_empty())
            .fold(request_builder, |builder, InputRow { key, value }| {
                builder.header(key.value(), value.value())
            });
        request_builder = request_builder.body(self.body_string());

        match request_builder.send() {
            Ok(response) => self.output_text = response.text().expect("Error unwrapping body"),
            Err(error) => self.output_text = format!("{:?}", error),
        };
    }

    fn set_cursor(&mut self, column: u16, row: u16) {
        self.cursor_col = column;
        self.cursor_row = row;
    }

    pub fn current_input_table(&self) -> &NonEmpty<InputRow> {
        match self.current_input_type {
            InputType::Headers => &self.headers_input_table,
            InputType::Body => &self.body_input_table,
        }
    }

    fn current_input_row(&self) -> &InputRow {
        &self.current_input_table()[self.input_index]
    }

    fn current_input(&self) -> &Input {
        match self.current_input_field {
            InputField::Key => &self.current_input_row().key,
            InputField::Value => &self.current_input_row().value,
        }
    }

    fn current_input_table_mut(&mut self) -> &mut NonEmpty<InputRow> {
        match self.current_input_type {
            InputType::Headers => &mut self.headers_input_table,
            InputType::Body => &mut self.body_input_table,
        }
    }

    fn current_input_row_mut(&mut self) -> &mut InputRow {
        let input_index = self.input_index;
        &mut self.current_input_table_mut()[input_index]
    }

    fn current_input_mut(&mut self) -> &mut Input {
        match self.current_input_field {
            InputField::Key => &mut self.current_input_row_mut().key,
            InputField::Value => &mut self.current_input_row_mut().value,
        }
    }

    fn body_string(&self) -> String {
        let mut object = object! {};
        self.body_input_table
            .iter()
            .for_each(|InputRow { key, value }| object[key.value()] = value.value().into());

        object.dump()
    }
}
