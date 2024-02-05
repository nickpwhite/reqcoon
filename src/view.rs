use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::model::{Model, CurrentPanel};

pub fn view(f: &mut Frame, model: &Model) {
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(3),
            Constraint::Min(1),
        ])
        .split(f.size());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(10),
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

    let mut output_block = Block::default()
        .title("Output")
        .borders(Borders::ALL)
        .style(Style::default());

    let active_style = Style::default().fg(Color::Blue);

    match model.current_panel {
        CurrentPanel::Method => {
            method_block = method_block.border_style(active_style)
        },
        CurrentPanel::Url => {
            url_block = url_block.border_style(active_style)
        },
        CurrentPanel::Output => {
            output_block = output_block.border_style(active_style)
        },
    };

    let method_text = Paragraph::new(Text::styled(
        model.method_input.to_string().clone(),
        Style::default().fg(Color::Green),
    ))
    .block(method_block);
    let url_text = Paragraph::new(model.url_input.clone()).block(url_block);
    let output_text = Paragraph::new(model.output_text.clone()).block(output_block);

    f.render_widget(method_text, top_chunks[0]);
    f.render_widget(url_text, top_chunks[1]);
    f.render_widget(output_text, chunks[1]);
    // f.set_cursor(model.current_cursor_position.x, model.current_cursor_position.y);
}
