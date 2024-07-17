use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
use std::iter::Iterator;
use std::str;

use clippers::Clipboard;
use crossterm::event::{KeyCode, KeyEvent};
use enum_iterator::Sequence;
use nonempty::{nonempty, NonEmpty};
use reqwest::{blocking::Client, Url};
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};
use serde_json::Value;
use tui_textarea::{CursorMove, TextArea};

use crate::tmux::{select_tmux_panel, Direction};

#[derive(Default, PartialEq, Serialize)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "Normal"),
            Mode::Insert => write!(f, "Insert"),
            Mode::Visual => write!(f, "Visual"),
        }
    }
}

#[derive(Default, PartialEq, Sequence, Serialize)]
pub enum Panel {
    #[default]
    Method,
    Url,
    Input,
    Output,
}

#[derive(Clone, Copy, Default, PartialEq, Sequence, Serialize)]
pub enum InputType {
    #[default]
    Auth,
    Headers,
    Body,
}

impl fmt::Display for InputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputType::Auth => write!(f, "Auth"),
            InputType::Headers => write!(f, "Headers"),
            InputType::Body => write!(f, "Body"),
        }
    }
}

#[derive(Default, PartialEq, Sequence, Serialize)]
pub enum AuthFormat {
    #[default]
    None,
    Basic,
    Bearer,
}

impl fmt::Display for AuthFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthFormat::None => write!(f, "None"),
            AuthFormat::Basic => write!(f, "Basic"),
            AuthFormat::Bearer => write!(f, "Bearer"),
        }
    }
}

impl From<&str> for AuthFormat {
    fn from(string: &str) -> Self {
        match string {
            "None" => AuthFormat::None,
            "Basic" => AuthFormat::Basic,
            "Bearer" => AuthFormat::Bearer,
            _ => AuthFormat::default(),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Sequence, Serialize)]
pub enum BodyFormat {
    #[default]
    Json,
    Form,
}

impl fmt::Display for BodyFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BodyFormat::Json => write!(f, "JSON"),
            BodyFormat::Form => write!(f, "Form"),
        }
    }
}

#[derive(Default, PartialEq, Sequence, Serialize)]
pub enum InputField {
    #[default]
    Key,
    Value,
}

#[derive(Default, Serialize)]
pub struct InputRow {
    pub key: TextArea<'static>,
    pub value: TextArea<'static>,
}

impl Into<(String, String)> for &InputRow {
    fn into(self) -> (String, String) {
        (
            self.key.lines()[0].to_string(),
            self.value.lines()[0].to_string(),
        )
    }
}

impl InputRow {
    fn is_empty(&self) -> bool {
        self.key.is_empty() && self.value.is_empty()
    }
}

#[derive(Default, Serialize)]
pub struct Auth {
    pub format: AuthFormat,
    pub basic_input: InputRow,
    pub bearer_input: TextArea<'static>,
}

impl Auth {
    fn username(&self) -> String {
        self.basic_input.key.lines()[0].to_string()
    }

    fn password(&self) -> Option<String> {
        let password = &self.basic_input.value.lines()[0];

        if password.is_empty() {
            None
        } else {
            Some(password.to_string())
        }
    }

    fn token(&self) -> String {
        self.bearer_input.lines()[0].to_string()
    }
}

#[derive(Clone, Default, Serialize)]
pub enum Method {
    OPTIONS,
    #[default]
    GET,
    HEAD,
    POST,
    PUT,
    PATCH,
    DELETE,
    TRACE,
    CONNECT,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<&str> for Method {
    fn from(method: &str) -> Self {
        match method {
            "OPTIONS" => Method::OPTIONS,
            "GET" => Method::GET,
            "HEAD" => Method::HEAD,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "PATCH" => Method::PATCH,
            "DELETE" => Method::DELETE,
            "TRACE" => Method::TRACE,
            "CONNECT" => Method::CONNECT,
            _ => Method::default(),
        }
    }
}

impl Into<reqwest::Method> for Method {
    fn into(self) -> reqwest::Method {
        match self {
            Method::OPTIONS => reqwest::Method::OPTIONS,
            Method::GET => reqwest::Method::GET,
            Method::HEAD => reqwest::Method::HEAD,
            Method::POST => reqwest::Method::POST,
            Method::PUT => reqwest::Method::PUT,
            Method::PATCH => reqwest::Method::PATCH,
            Method::DELETE => reqwest::Method::DELETE,
            Method::TRACE => reqwest::Method::TRACE,
            Method::CONNECT => reqwest::Method::CONNECT,
        }
    }
}

impl Method {
    pub fn to_string(&self) -> String {
        match self {
            Method::OPTIONS => "OPTIONS",
            Method::GET => "GET",
            Method::HEAD => "HEAD",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::PATCH => "PATCH",
            Method::DELETE => "DELETE",
            Method::TRACE => "TRACE",
            Method::CONNECT => "CONNECT",
        }
        .to_string()
    }
}

pub struct Model {
    pub filename: String,
    pub current_mode: Mode,
    pub current_panel: Panel,
    pub current_method: Method,
    pub dummy_input: TextArea<'static>,
    pub url_input: TextArea<'static>,
    pub auth: Auth,
    pub current_input_type: InputType,
    pub current_input_field: InputField,
    pub current_body_format: BodyFormat,
    pub input_index: usize,
    pub headers_input_table: NonEmpty<InputRow>,
    pub body_input_table: NonEmpty<InputRow>,
    pub json_body_input: TextArea<'static>,
    pub output_input: TextArea<'static>,
    pub message: String,
    pub exit: bool,
}

impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Model", 6)?;
        state.serialize_field("current_method", &self.current_method)?;
        state.serialize_field("url_input", &self.url_input)?;
        state.serialize_field("auth", &self.auth)?;
        state.serialize_field("headers_input_table", &self.headers_input_table)?;
        state.serialize_field("body_input_table", &self.body_input_table)?;
        state.serialize_field("json_body_input", &self.json_body_input)?;
        state.end()
    }
}

impl Model {
    pub fn new(filename: String) -> Model {
        Model {
            filename,
            current_mode: Mode::default(),
            current_panel: Panel::default(),
            current_method: Method::GET,
            dummy_input: TextArea::default(),
            url_input: TextArea::default(),
            current_input_type: InputType::default(),
            current_input_field: InputField::default(),
            auth: Auth::default(),
            current_body_format: BodyFormat::default(),
            input_index: 0,
            headers_input_table: nonempty![InputRow::default()],
            body_input_table: nonempty![InputRow::default()],
            json_body_input: TextArea::default(),
            output_input: TextArea::default(),
            message: String::default(),
            exit: false,
        }
    }

    pub fn from_file(filename: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut input = String::new();
        let mut file = File::open(filename.clone())?;
        file.read_to_string(&mut input)?;

        let json_model: Value = serde_json::from_str(&input)?;

        let auth = Auth {
            format: json_model["auth"]["format"].as_str().unwrap().into(),
            basic_input: InputRow {
                key: TextArea::from(
                    json_model["auth"]["basic_input"]["key"]
                        .as_str()
                        .unwrap()
                        .lines(),
                ),
                value: TextArea::from(
                    json_model["auth"]["basic_input"]["value"]
                        .as_str()
                        .unwrap()
                        .lines(),
                ),
            },
            bearer_input: TextArea::from(
                json_model["auth"]["bearer_input"].as_str().unwrap().lines(),
            ),
        };

        let headers_input_table = match json_model["headers_input_table"].as_array() {
            Some(headers) => {
                let headers_vec = headers
                    .iter()
                    .map(|header| InputRow {
                        key: TextArea::from(header["key"].as_str().unwrap().lines()),
                        value: TextArea::from(header["value"].as_str().unwrap().lines()),
                    })
                    .collect::<Vec<InputRow>>();

                if headers_vec.len() > 0 {
                    NonEmpty::from_vec(headers_vec).unwrap()
                } else {
                    nonempty![InputRow::default()]
                }
            }
            None => nonempty![InputRow::default()],
        };

        let body_input_table = match json_model["body_input_table"].as_array() {
            Some(body_params) => {
                let body_vec = body_params
                    .iter()
                    .map(|body_param| InputRow {
                        key: TextArea::from(body_param["key"].as_str().unwrap().lines()),
                        value: TextArea::from(body_param["value"].as_str().unwrap().lines()),
                    })
                    .collect::<Vec<InputRow>>();

                if body_vec.len() > 0 {
                    NonEmpty::from_vec(body_vec).unwrap()
                } else {
                    nonempty![InputRow::default()]
                }
            }
            None => nonempty![InputRow::default()],
        };

        Ok(Self {
            filename,
            current_mode: Mode::default(),
            current_panel: Panel::default(),
            current_method: json_model["current_method"].as_str().unwrap().into(),
            dummy_input: TextArea::default(),
            url_input: json_model["url_input"].as_str().unwrap().lines().into(),
            current_input_type: InputType::default(),
            current_input_field: InputField::default(),
            auth,
            current_body_format: BodyFormat::default(),
            input_index: 0,
            headers_input_table,
            body_input_table,
            json_body_input: json_model["json_body_input"]
                .as_str()
                .unwrap()
                .lines()
                .into(),
            output_input: TextArea::default(),
            message: String::default(),
            exit: false,
        })
    }

    pub fn to_file(&self) -> io::Result<()> {
        let mut json_file = File::create(&self.filename)?;
        json_file.write_all(serde_json::to_string_pretty(&self)?.as_bytes())
    }

    pub fn append(&mut self) {
        self.current_mode = Mode::Insert;
        self.current_input_mut().move_cursor(CursorMove::Forward);
    }

    pub fn insert(&mut self) {
        self.current_mode = Mode::Insert;
    }

    pub fn leave_insert(&mut self) {
        self.current_input_mut().move_cursor(CursorMove::Back);
    }

    pub fn normal(&mut self) {
        self.current_mode = Mode::Normal;
    }

    pub fn visual(&mut self) {
        self.current_mode = Mode::Visual;
        self.current_input_mut().start_selection();
    }

    pub fn leave_visual(&mut self) {
        self.current_input_mut().cancel_selection();
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
        self.current_method.to_string().len() as u16
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
        };

        self.current_method = new_method;
    }

    pub fn cursor_col(&self) -> u16 {
        self.current_input().cursor().1 as u16
    }

    pub fn copy(&mut self) {
        self.current_input_mut().copy();
        match Clipboard::get().write_text(self.current_input().yank_text()) {
            Ok(_) => (),
            Err(err) => self.message = format!("Unable to save to system clipboard: {:?}", err),
        }
    }

    pub fn handle_insert_input(&mut self, event: KeyEvent) {
        self.current_input_mut().input(event);
    }

    pub fn handle_normal_input(&mut self, key_event: KeyEvent) {
        let cursor_move = match key_event.code {
            KeyCode::Char('h') | KeyCode::Left => Some(CursorMove::Back),
            KeyCode::Char('l') | KeyCode::Right => Some(CursorMove::Forward),
            KeyCode::Char('b') => Some(CursorMove::WordBack),
            KeyCode::Char('w') => Some(CursorMove::WordForward),
            KeyCode::Char('^') | KeyCode::Home => Some(CursorMove::Head),
            KeyCode::Char('$') | KeyCode::End => Some(CursorMove::End),
            KeyCode::Char('j') | KeyCode::Down if self.is_multiline_input() => {
                Some(CursorMove::Down)
            }
            KeyCode::Char('k') | KeyCode::Up if self.is_multiline_input() => Some(CursorMove::Up),
            _ => None,
        };

        match cursor_move {
            Some(request) => self.current_input_mut().move_cursor(request),
            None => (),
        };
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
        match self.current_input_type {
            InputType::Auth => match self.auth.format {
                AuthFormat::None | AuthFormat::Bearer => (),
                AuthFormat::Basic => {
                    self.current_input_field = self.current_input_field.next().unwrap_or_default();
                }
            },
            InputType::Headers | InputType::Body => {
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
        }
    }

    pub fn previous_input_field(&mut self) {
        match self.current_input_type {
            InputType::Auth => match self.auth.format {
                AuthFormat::None | AuthFormat::Bearer => (),
                AuthFormat::Basic => {
                    self.current_input_field = self
                        .current_input_field
                        .previous()
                        .unwrap_or(InputField::last().unwrap());
                }
            },
            InputType::Headers | InputType::Body => {
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
        }
    }

    pub fn next_input_format(&mut self) {
        match self.current_input_type {
            InputType::Auth => {
                self.auth.format = self.auth.format.next().unwrap_or_default();
            }
            InputType::Headers => (),
            InputType::Body => {
                self.current_body_format = self.current_body_format.next().unwrap_or_default();
            }
        }
    }

    pub fn previous_input_format(&mut self) {
        match self.current_input_type {
            InputType::Auth => {
                self.auth.format = self
                    .auth
                    .format
                    .previous()
                    .unwrap_or(AuthFormat::last().unwrap());
            }
            InputType::Headers => (),
            InputType::Body => {
                self.current_body_format = self
                    .current_body_format
                    .previous()
                    .unwrap_or(BodyFormat::last().unwrap());
            }
        }
    }

    pub fn current_input_table(&self) -> &NonEmpty<InputRow> {
        match self.current_input_type {
            InputType::Auth | InputType::Headers => &self.headers_input_table,
            InputType::Body => &self.body_input_table,
        }
    }

    pub fn submit_request(&mut self) {
        let url = Url::parse(&self.url_input.lines()[0]).expect("Invalid URL");
        let mut request_builder = Client::new().request(self.current_method.clone().into(), url);

        request_builder = match self.current_body_format {
            BodyFormat::Json => request_builder
                .header("Content-Type", "application/json")
                .body(self.json_body_input.lines().join("\n")),
            BodyFormat::Form => request_builder.form(&self.body_hash_map()),
        };
        request_builder = match self.auth.format {
            AuthFormat::None => request_builder,
            AuthFormat::Basic => {
                request_builder.basic_auth(self.auth.username(), self.auth.password())
            }
            AuthFormat::Bearer => request_builder.bearer_auth(self.auth.token()),
        };
        request_builder = self
            .non_empty_headers()
            .fold(request_builder, |builder, InputRow { key, value }| {
                builder.header(&key.lines()[0], &value.lines()[0])
            });

        let output = match request_builder.send() {
            Ok(response) => response
                .text()
                .unwrap_or("Error unwrapping body".to_string()),
            Err(error) => format!("{:?}", error),
        };

        self.output_input = TextArea::from(output.lines());
    }

    fn current_input(&self) -> &TextArea<'static> {
        match self.current_panel {
            Panel::Method => &self.dummy_input,
            Panel::Url => &self.url_input,
            Panel::Input => match self.current_input_type {
                InputType::Auth => match self.auth.format {
                    AuthFormat::None => &self.dummy_input,
                    AuthFormat::Basic => match self.current_input_field {
                        InputField::Key => &self.auth.basic_input.key,
                        InputField::Value => &self.auth.basic_input.value,
                    },
                    AuthFormat::Bearer => &self.auth.bearer_input,
                },
                InputType::Headers | InputType::Body => match self.current_input_field {
                    InputField::Key => &self.current_input_row().key,
                    InputField::Value => &self.current_input_row().value,
                },
            },
            Panel::Output => &self.output_input,
        }
    }

    fn current_input_mut(&mut self) -> &mut TextArea<'static> {
        match self.current_panel {
            Panel::Method => &mut self.dummy_input,
            Panel::Url => &mut self.url_input,
            Panel::Input => match self.current_input_type {
                InputType::Auth => match self.auth.format {
                    AuthFormat::None => &mut self.dummy_input,
                    AuthFormat::Basic => match self.current_input_field {
                        InputField::Key => &mut self.auth.basic_input.key,
                        InputField::Value => &mut self.auth.basic_input.value,
                    },
                    AuthFormat::Bearer => &mut self.auth.bearer_input,
                },
                InputType::Body if self.current_body_format == BodyFormat::Json => {
                    &mut self.json_body_input
                }
                InputType::Headers | InputType::Body => match self.current_input_field {
                    InputField::Key => &mut self.current_input_row_mut().key,
                    InputField::Value => &mut self.current_input_row_mut().value,
                },
            },
            Panel::Output => &mut self.output_input,
        }
    }

    fn current_input_row(&self) -> &InputRow {
        &self.current_input_table()[self.input_index]
    }

    fn current_input_table_mut(&mut self) -> &mut NonEmpty<InputRow> {
        match self.current_input_type {
            InputType::Auth | InputType::Headers => &mut self.headers_input_table,
            InputType::Body => &mut self.body_input_table,
        }
    }

    fn current_input_row_mut(&mut self) -> &mut InputRow {
        let input_index = self.input_index;
        &mut self.current_input_table_mut()[input_index]
    }

    fn non_empty_headers(&self) -> impl Iterator<Item = &InputRow> {
        self.headers_input_table
            .iter()
            .filter(|header| !header.key.is_empty())
    }

    fn non_empty_body(&self) -> impl Iterator<Item = &InputRow> {
        self.body_input_table
            .iter()
            .filter(|body_pair| !body_pair.key.is_empty())
    }

    fn body_hash_map(&self) -> HashMap<String, String> {
        self.non_empty_body().map(|row| row.into()).collect()
    }

    fn is_multiline_input(&self) -> bool {
        (self.current_panel == Panel::Input
            && self.current_input_type == InputType::Body
            && self.current_body_format == BodyFormat::Json)
            || self.current_panel == Panel::Output
    }
}
