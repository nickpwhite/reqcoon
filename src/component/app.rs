use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use enum_iterator::Sequence;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget, WidgetRef, Wrap},
};
use reqwest::Method;

use crate::component::Component;

#[derive(Clone, Default, PartialEq, Sequence)]
enum CurrentPanel {
    #[default]
    Method,
    Url,
    Input,
    Output,
}

const METHODS: [Method; 5] = [
    Method::GET,
    Method::HEAD,
    Method::POST,
    Method::PUT,
    Method::DELETE,
];

#[derive(Clone)]
pub struct App {
    current_panel: CurrentPanel,
    url_input: String,
    output_text: String,
    pub exit: bool,
}

impl App {
    fn current_method(&self) -> &Method {
        &METHODS[0]
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_panel: CurrentPanel::default(),
            url_input: String::default(),
            output_text: String::default(),
            exit: bool::default(),
        }
    }
}

impl Component for App {
    fn iter_children(self) -> dyn Iterator<Item = dyn Component> {
        [].into_iter()
    }

    fn handle_key_event(self, key_event: KeyEvent) -> Self {
        match key_event.modifiers {
            KeyModifiers::CONTROL => match key_event.code {
                KeyCode::Char('c') => Self {
                    exit: true,
                    ..self.clone()
                },
                _ => self,
            },
            _ => match key_event.code {
                KeyCode::Tab => Self {
                    current_panel: self.current_panel.next().unwrap_or_default(),
                    ..self.clone()
                },
                KeyCode::BackTab => Self {
                    current_panel: self.current_panel.previous().unwrap_or(CurrentPanel::last().unwrap()),
                    ..self.clone()
                },
                _ => self,
            },
        }
    }
}

impl WidgetRef for App {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        // Create the layout sections.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Max(10),
                Constraint::Min(1),
            ])
            .split(area);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Max(13),
                Constraint::Min(1),
            ])
            .split(chunks[0]);

        let mut method_block = Block::default()
            .title("Method")
            .borders(Borders::ALL)
            .style(Style::default());

        let mut url_block = Block::default()
            .title("URL")
            .borders(Borders::ALL)
            .style(Style::default());

        let mut body_block = Block::default()
            .title("Input")
            .borders(Borders::ALL)
            .style(Style::default());

        let mut output_block = Block::default()
            .title("Output")
            .borders(Borders::ALL)
            .style(Style::default());

        let active_style = Style::default().fg(Color::Blue);

        match self.current_panel {
            CurrentPanel::Method => {
                method_block = method_block.border_style(active_style);
            },
            CurrentPanel::Url => {
                url_block = url_block.border_style(active_style);
            },
            CurrentPanel::Input => {
                body_block = body_block.border_style(active_style);
            },
            CurrentPanel::Output => {
                output_block = output_block.border_style(active_style);
            },
        };

        let method_text = Paragraph::new(Text::styled(
                self.current_method().to_string().clone(),
                Style::default().fg(Color::Green),
        ))
            .block(method_block);
        let url_text = Paragraph::new(self.url_input.clone()).block(url_block);
        let body_text = Paragraph::new("{}").block(body_block);
        let output_text = Paragraph::new(self.output_text.clone()).wrap(Wrap { trim: false }).block(output_block);

        method_text.render(top_chunks[0], buf);
        url_text.render(top_chunks[1], buf);
        body_text.render(chunks[1], buf);
        output_text.render(chunks[2], buf);
    }
}
