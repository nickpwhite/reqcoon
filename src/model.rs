use crossterm::event::Event;
use ratatui::widgets::ListState;
use reqwest::{blocking::{Client, Request}, Method, Url};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(PartialEq)]
pub enum CurrentPanel {
    Method,
    Url,
    Input,
    Output,
}

pub struct Model {
    pub current_panel: CurrentPanel,
    pub cursor_col: u16,
    pub cursor_row: u16,
    pub list_state: ListState,
    pub url_input: Input,
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
            url_input: Input::default(),
            output_text: String::new(),
            exit: false,
        }
    }

    pub fn update_cursor(&mut self) {
        match self.current_panel {
            CurrentPanel::Method => {
                self.set_cursor(1 + self.current_method().to_string().len() as u16, 1)
            }
            CurrentPanel::Url => self.set_cursor(1 + self.url_input.visual_cursor() as u16, 1),
            CurrentPanel::Input => {
                self.set_cursor(3, 1);
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
        match self.current_panel {
            CurrentPanel::Method => self.current_panel = CurrentPanel::Url,
            CurrentPanel::Url => self.current_panel = CurrentPanel::Input,
            CurrentPanel::Input => self.current_panel = CurrentPanel::Output,
            CurrentPanel::Output => self.current_panel = CurrentPanel::Method,
        };
    }

    pub fn previous_panel(&mut self) {
        match self.current_panel {
            CurrentPanel::Method => self.current_panel = CurrentPanel::Output,
            CurrentPanel::Url => self.current_panel = CurrentPanel::Method,
            CurrentPanel::Input => self.current_panel = CurrentPanel::Url,
            CurrentPanel::Output => self.current_panel = CurrentPanel::Input,
        }
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
