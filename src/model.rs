use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
use std::iter::Iterator;
use std::str;

use clippers::Clipboard;
use crossterm::event::{KeyCode, KeyEvent};
use enum_iterator::Sequence;
use http_auth_basic::Credentials;
use json::JsonValue;
use log::error;
use nonempty::{nonempty, NonEmpty};
use pest::Parser;
use pest_derive::Parser;
use ratatui::widgets::ListState;
use regex::RegexBuilder;
use reqwest::{blocking::Client, Method, Url};
use tui_textarea::{CursorMove, TextArea};

use crate::input::{CursorMove as CursorMoveNew, Input, OnelineInput};
use crate::tmux::{select_tmux_panel, Direction};

#[derive(Default, PartialEq)]
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

#[derive(Default, PartialEq, Sequence)]
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

#[derive(Default, Sequence)]
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

#[derive(Default, PartialEq, Sequence)]
pub enum InputField {
    #[default]
    Key,
    Value,
}

#[derive(Default)]
pub struct InputRow {
    pub key: TextArea<'static>,
    pub value: TextArea<'static>,
}

impl InputRow {
    fn is_empty(&self) -> bool {
        self.key.is_empty() && self.value.is_empty()
    }
}

impl Into<(String, String)> for &InputRow {
    fn into(self) -> (String, String) {
        (
            self.key.lines()[0].to_string(),
            self.value.lines()[0].to_string(),
        )
    }
}

#[derive(Default)]
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

#[derive(Parser)]
#[grammar = "http.pest"]
struct RequestParser;

pub struct Model {
    pub filename: String,
    pub current_mode: Mode,
    pub current_panel: Panel,
    pub list_state: ListState,
    pub current_method: Method,
    pub dummy_input: TextArea<'static>,
    pub url_input: TextArea<'static>,
    pub new_url_input: OnelineInput,
    pub auth: Auth,
    pub current_input_type: InputType,
    pub current_input_field: InputField,
    pub current_body_format: BodyFormat,
    pub input_index: usize,
    pub headers_input_table: NonEmpty<InputRow>,
    pub body_input_table: NonEmpty<InputRow>,
    pub output_row: usize,
    pub output_input: TextArea<'static>,
    pub message: String,
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
            dummy_input: TextArea::default(),
            url_input: TextArea::default(),
            new_url_input: OnelineInput::default(),
            current_input_type: InputType::default(),
            current_input_field: InputField::default(),
            auth: Auth::default(),
            current_body_format: BodyFormat::default(),
            input_index: 0,
            headers_input_table: nonempty![InputRow::default()],
            body_input_table: nonempty![InputRow::default()],
            output_row: 0,
            output_input: TextArea::default(),
            message: String::default(),
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
                            key: [key].into(),
                            value: [value].into(),
                        });
                    }
                }
                Rule::body => {
                    let object = json::parse(pair.as_str())?;
                    for (key, value) in object.entries() {
                        body_input.push(InputRow {
                            key: [key].into(),
                            value: [value.as_str().unwrap()].into(),
                        })
                    }
                }
                _ => (),
            }
        }

        let (auth, headers) = Self::parse_headers_input(headers_input);

        Ok(Self {
            filename,
            current_mode: Mode::default(),
            current_panel: Panel::default(),
            list_state: ListState::default().with_selected(Some(0)),
            current_method: method,
            dummy_input: TextArea::default(),
            url_input: TextArea::from([uri]),
            new_url_input: OnelineInput::from(uri),
            auth,
            current_input_type: InputType::default(),
            current_input_field: InputField::default(),
            current_body_format: BodyFormat::default(),
            input_index: 0,
            headers_input_table: NonEmpty::from_vec(headers)
                .unwrap_or(nonempty![InputRow::default()]),
            body_input_table: NonEmpty::from_vec(body_input)
                .unwrap_or(nonempty![InputRow::default()]),
            output_row: 0,
            output_input: TextArea::default(),
            message: String::default(),
            exit: false,
        })
    }

    fn parse_headers_input(mut headers_input: Vec<InputRow>) -> (Auth, Vec<InputRow>) {
        match headers_input
            .iter()
            .position(|input_row| input_row.key.lines()[0] == "Authorization")
        {
            Some(index) => {
                let re = RegexBuilder::new(r"(basic|bearer) (.*)")
                    .case_insensitive(true)
                    .build()
                    .unwrap();
                let auth_header = headers_input.remove(index);
                match re.captures(&auth_header.value.lines()[0]) {
                    Some(captures) => match captures[1].to_lowercase().as_str() {
                        "basic" => {
                            match Credentials::from_header(auth_header.value.lines()[0].clone()) {
                                Ok(credentials) => (
                                    Auth {
                                        format: AuthFormat::Basic,
                                        basic_input: InputRow {
                                            key: TextArea::from(credentials.user_id.lines()),
                                            value: TextArea::from(credentials.password.lines()),
                                        },
                                        bearer_input: TextArea::default(),
                                    },
                                    headers_input,
                                ),
                                Err(err) => {
                                    error!("{:?}", err);
                                    headers_input.insert(index, auth_header);
                                    (Auth::default(), headers_input)
                                }
                            }
                        }
                        "bearer" => (
                            Auth {
                                format: AuthFormat::Bearer,
                                basic_input: InputRow::default(),
                                bearer_input: TextArea::from(captures[2].to_string().lines()),
                            },
                            headers_input,
                        ),
                        _ => {
                            headers_input.insert(index, auth_header);
                            (Auth::default(), headers_input)
                        }
                    },
                    None => {
                        headers_input.insert(index, auth_header);
                        (Auth::default(), headers_input)
                    }
                }
            }
            None => (Auth::default(), headers_input),
        }
    }

    pub fn to_file(&self) -> io::Result<()> {
        let mut output = format!("{} {}", self.current_method, self.url_input.lines()[0]);
        if !self.auth_string().is_empty() {
            output.push_str("\n");
            output.push_str(&self.auth_string());
        }
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

    pub fn append(&mut self) {
        self.current_mode = Mode::Insert;
        match self.current_panel {
            Panel::Url => self
                .current_input_mut_new()
                .move_cursor(CursorMoveNew::NextChar),
            _ => self.current_input_mut().move_cursor(CursorMove::Forward),
        };
    }

    pub fn insert(&mut self) {
        self.current_mode = Mode::Insert;
    }

    pub fn leave_insert(&mut self) {
        match self.current_panel {
            Panel::Url => self
                .current_input_mut_new()
                .move_cursor(CursorMoveNew::PrevChar),
            _ => self.current_input_mut().move_cursor(CursorMove::Back),
        };
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

    pub fn cursor_col(&self) -> u16 {
        match self.current_panel {
            Panel::Url => self.current_input_new().cursor().0 as u16,
            _ => self.current_input().cursor().1 as u16,
        }
    }

    pub fn copy(&mut self) {
        self.current_input_mut().copy();
        match Clipboard::get().write_text(self.current_input().yank_text()) {
            Ok(_) => (),
            Err(err) => self.message = format!("Unable to save to system clipboard: {:?}", err),
        }
    }

    pub fn handle_insert_input(&mut self, event: KeyEvent) {
        if self.current_panel == Panel::Input
            && self.current_input_type == InputType::Auth
            && self.auth.format == AuthFormat::None
        {
            return;
        }

        match self.current_panel {
            Panel::Url => {
                self.current_input_mut_new().handle_input(event);
            }
            _ => {
                self.current_input_mut().input(event);
            }
        };
    }

    pub fn handle_normal_input(&mut self, key_event: KeyEvent) {
        if self.current_panel == Panel::Input
            && self.current_input_type == InputType::Auth
            && self.auth.format == AuthFormat::None
        {
            return;
        }

        if self.current_panel == Panel::Url {
            return self.handle_normal_input_new(key_event);
        }

        let cursor_move = match key_event.code {
            KeyCode::Char('h') | KeyCode::Left => Some(CursorMove::Back),
            KeyCode::Char('l') | KeyCode::Right => Some(CursorMove::Forward),
            KeyCode::Char('b') => Some(CursorMove::WordBack),
            KeyCode::Char('w') => Some(CursorMove::WordForward),
            KeyCode::Char('^') | KeyCode::Home => Some(CursorMove::Head),
            KeyCode::Char('$') | KeyCode::End => Some(CursorMove::End),
            KeyCode::Char('j') | KeyCode::Down if self.current_panel == Panel::Output => {
                Some(CursorMove::Down)
            }
            KeyCode::Char('k') | KeyCode::Up if self.current_panel == Panel::Output => {
                Some(CursorMove::Up)
            }
            _ => None,
        };

        match cursor_move {
            Some(request) => self.current_input_mut().move_cursor(request),
            None => (),
        };
    }

    pub fn handle_normal_input_new(&mut self, key_event: KeyEvent) {
        let cursor_move = match key_event.code {
            KeyCode::Char('h') | KeyCode::Left => Some(CursorMoveNew::PrevChar),
            KeyCode::Char('l') | KeyCode::Right => Some(CursorMoveNew::NextChar),
            KeyCode::Char('b') => Some(CursorMoveNew::PrevWord),
            KeyCode::Char('w') => Some(CursorMoveNew::NextWord),
            KeyCode::Char('^') | KeyCode::Home => Some(CursorMoveNew::LineHead),
            KeyCode::Char('$') | KeyCode::End => Some(CursorMoveNew::LineEnd),
            _ => None,
        };

        match cursor_move {
            Some(request) => self.current_input_mut_new().move_cursor(request),
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
        let mut request_builder = Client::new().request(self.current_method.clone(), url);

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
        request_builder = match self.current_body_format {
            BodyFormat::Json => request_builder.json(&self.body_hash_map()),
            BodyFormat::Form => request_builder.form(&self.body_hash_map()),
        };

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

    fn current_input_new(&self) -> &impl Input {
        &self.new_url_input
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
                InputType::Headers | InputType::Body => match self.current_input_field {
                    InputField::Key => &mut self.current_input_row_mut().key,
                    InputField::Value => &mut self.current_input_row_mut().value,
                },
            },
            Panel::Output => &mut self.output_input,
        }
    }

    fn current_input_mut_new(&mut self) -> &mut impl Input {
        &mut self.new_url_input
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

    fn auth_string(&self) -> String {
        match self.auth.format {
            AuthFormat::None => String::default(),
            AuthFormat::Basic => Credentials {
                user_id: self.auth.username(),
                password: self.auth.password().unwrap_or(String::default()),
            }
            .as_http_header(),
            AuthFormat::Bearer => format!("Bearer {}", self.auth.token()),
        }
    }

    fn non_empty_headers(&self) -> impl Iterator<Item = &InputRow> {
        self.headers_input_table
            .iter()
            .filter(|header| !header.key.is_empty())
    }

    fn headers_string(&self) -> String {
        self.non_empty_headers()
            .map(|input_row| {
                format!(
                    "{}: {}",
                    input_row.key.lines()[0],
                    input_row.value.lines()[0]
                )
            })
            .collect()
    }

    fn non_empty_body(&self) -> impl Iterator<Item = &InputRow> {
        self.body_input_table
            .iter()
            .filter(|body_pair| !body_pair.key.is_empty())
    }

    fn body_string(&self) -> String {
        match self.current_body_format {
            BodyFormat::Json => JsonValue::Object(
                self.non_empty_body()
                    .map(|InputRow { key, value }| {
                        (key.lines()[0].clone(), value.lines()[0].clone())
                    })
                    .collect(),
            )
            .dump(),
            BodyFormat::Form => "".to_string(),
        }
    }

    fn body_hash_map(&self) -> HashMap<String, String> {
        self.non_empty_body().map(|row| row.into()).collect()
    }
}
