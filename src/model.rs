use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
use std::iter::Iterator;

use crossterm::event::Event;
use enum_iterator::Sequence;
use json::object;
use nonempty::{nonempty, NonEmpty};
use pest::Parser;
use pest_derive::Parser;
use ratatui::widgets::ListState;
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
    pub key: Input,
    pub value: Input,
}

impl InputRow {
    fn is_empty(&self) -> bool {
        self.key.value().is_empty() && self.value.value().is_empty()
    }
}

#[derive(Parser)]
#[grammar = "http.pest"]
struct RequestParser;

pub struct Model {
    pub filename: String,
    pub current_mode: Mode,
    pub current_panel: Panel,
    pub list_state: ListState,
    pub current_method: Method,
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
    pub fn new(filename: String) -> Model {
        Model {
            filename,
            current_mode: Mode::default(),
            current_panel: Panel::default(),
            list_state: ListState::default().with_selected(Some(0)),
            current_method: Method::GET,
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

    pub fn from_file(filename: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut input = String::new();
        let mut file = File::open(filename.clone())?;
        file.read_to_string(&mut input)?;

        let mut method = Method::GET;
        let mut uri = "";
        let mut headers_input = vec![];
        let mut body_input = vec![];

        let pairs = RequestParser::parse(Rule::request, &input)?;
        for pair in pairs {
            match pair.as_rule() {
                Rule::method => method = Method::from_bytes(pair.as_str().as_bytes())?,
                Rule::uri => uri = pair.as_str(),
                Rule::headers => {
                    for header in pair.into_inner() {
                        let mut key = "";
                        let mut value = "";
                        for inner_rule in header.into_inner() {
                            match inner_rule.as_rule() {
                                Rule::header_name => key = inner_rule.as_str(),
                                Rule::header_value => value = inner_rule.as_str(),
                                _ => (),
                            }
                        }
                        headers_input.push(InputRow {
                            key: key.into(),
                            value: value.into(),
                        });
                    }
                }
                Rule::body => {
                    let object = json::parse(pair.as_str())?;
                    for (key, value) in object.entries() {
                        body_input.push(InputRow {
                            key: key.into(),
                            value: value.as_str().unwrap().into(),
                        })
                    }
                }
                _ => (),
            }
        }

        Ok(Self {
            filename,
            current_mode: Mode::default(),
            current_panel: Panel::default(),
            list_state: ListState::default().with_selected(Some(0)),
            current_method: method,
            url_input: Input::from(uri),
            current_input_type: InputType::default(),
            current_input_field: InputField::Key,
            input_index: 0,
            headers_input_table: NonEmpty::from_vec(headers_input)
                .unwrap_or(nonempty![InputRow::default()]),
            body_input_table: NonEmpty::from_vec(body_input)
                .unwrap_or(nonempty![InputRow::default()]),
            output_text: String::new(),
            exit: false,
        })
    }

    pub fn to_file(&self) -> io::Result<()> {
        let mut output = format!("{} {}", self.current_method, self.url_input.value());
        if !self.headers_string().is_empty() {
            output.push_str("\n");
            output.push_str(&self.headers_string());
        }
        if !self.body_string().is_empty() {
            output.push_str("\n\n");
            output.push_str(&self.body_string());
        }
        let mut file = File::create(&self.filename)?;

        file.write_all(output.as_bytes())
    }

    pub fn enter_insert(&mut self) {
        self.current_mode = Mode::Insert;
    }

    pub fn enter_normal(&mut self) {
        self.current_mode = Mode::Normal;
    }

    pub fn select_panel_left(&mut self) {
        match self.current_panel {
            Panel::Url => {
                self.current_panel = Panel::Method;
            }
            _ => select_tmux_panel(Direction::Left),
        };
    }

    pub fn select_panel_down(&mut self) {
        match self.current_panel {
            Panel::Url | Panel::Method => {
                self.current_panel = Panel::Input;
            }
            Panel::Input => {
                self.current_panel = Panel::Output;
            }
            _ => select_tmux_panel(Direction::Down),
        };
    }

    pub fn select_panel_up(&mut self) {
        match self.current_panel {
            Panel::Output => {
                self.current_panel = Panel::Input;
            }
            Panel::Input => {
                self.current_panel = Panel::Url;
            }
            _ => select_tmux_panel(Direction::Up),
        };
    }

    pub fn select_panel_right(&mut self) {
        match self.current_panel {
            Panel::Method => {
                self.current_panel = Panel::Url;
            }
            _ => select_tmux_panel(Direction::Right),
        };
    }

    pub fn method_cursor_position(&self) -> u16 {
        let offset = match self.current_mode {
            Mode::Normal => 0,
            Mode::Insert => 1,
        };

        self.current_method.to_string().len() as u16 + offset
    }

    pub fn next_method(&mut self) {
        let new_method = match self.current_method {
            Method::OPTIONS => Method::GET,
            Method::GET => Method::HEAD,
            Method::HEAD => Method::POST,
            Method::POST => Method::PUT,
            Method::PUT => Method::PATCH,
            Method::PATCH => Method::DELETE,
            Method::DELETE => Method::TRACE,
            Method::TRACE => Method::CONNECT,
            Method::CONNECT => Method::OPTIONS,
            _ => return,
        };

        self.current_method = new_method;
    }

    pub fn previous_method(&mut self) {
        let new_method = match self.current_method {
            Method::OPTIONS => Method::CONNECT,
            Method::GET => Method::OPTIONS,
            Method::HEAD => Method::GET,
            Method::POST => Method::HEAD,
            Method::PUT => Method::POST,
            Method::PATCH => Method::PUT,
            Method::DELETE => Method::PATCH,
            Method::TRACE => Method::DELETE,
            Method::CONNECT => Method::TRACE,
            _ => return,
        };

        self.current_method = new_method;
    }

    pub fn url_cursor_position(&self) -> u16 {
        let offset = match self.current_mode {
            Mode::Normal => {
                if self.url_input.visual_cursor() == 0 {
                    1
                } else {
                    0
                }
            }
            Mode::Insert => 1,
        };

        self.url_input.visual_cursor() as u16 + offset
    }

    pub fn handle_url_input(&mut self, event: Event) {
        self.url_input.handle_event(&event);
    }

    pub fn input_cursor_position(&self) -> u16 {
        let offset = match self.current_mode {
            Mode::Normal => {
                if self.current_input().visual_cursor() == 0 {
                    1
                } else {
                    0
                }
            }
            Mode::Insert => 1,
        };

        self.current_input().visual_cursor() as u16 + offset
    }

    pub fn next_input_type(&mut self) {
        self.current_input_type = self.current_input_type.next().unwrap_or_default();
        self.current_input_field = InputField::default();
        self.input_index = self.current_input_table().len() - 1;
    }

    pub fn previous_input_type(&mut self) {
        self.current_input_type = self
            .current_input_type
            .previous()
            .unwrap_or(InputType::last().unwrap());
        self.current_input_field = InputField::default();
        self.input_index = self.current_input_table().len() - 1;
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
    }

    pub fn handle_input_input(&mut self, event: Event) {
        self.current_input_mut().handle_event(&event);
    }

    pub fn submit_request(&mut self) {
        let url = Url::parse(&self.url_input.value()).expect("Invalid URL");
        let mut request_builder = Client::new().request(self.current_method.clone(), url);

        request_builder = self
            .non_empty_headers()
            .fold(request_builder, |builder, InputRow { key, value }| {
                builder.header(key.value(), value.value())
            });
        request_builder = request_builder.body(self.body_string());

        match request_builder.send() {
            Ok(response) => self.output_text = response.text().expect("Error unwrapping body"),
            Err(error) => self.output_text = format!("{:?}", error),
        };
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

    pub fn current_input(&self) -> &Input {
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

    fn non_empty_headers(&self) -> impl Iterator<Item = &InputRow> {
        self.headers_input_table
            .iter()
            .filter(|header| !header.key.value().is_empty())
    }

    fn headers_string(&self) -> String {
        self.non_empty_headers()
            .map(|input_row| format!("{}: {}", input_row.key.value(), input_row.value.value()))
            .collect()
    }

    fn non_empty_body(&self) -> impl Iterator<Item = &InputRow> {
        self.body_input_table
            .iter()
            .filter(|body_pair| !body_pair.key.value().is_empty())
    }

    fn body_string(&self) -> String {
        let mut object = object! {};
        self.non_empty_body()
            .for_each(|InputRow { key, value }| object[key.value()] = value.value().into());

        object.dump()
    }
}
