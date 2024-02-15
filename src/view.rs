use ratatui::{
    layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, symbols, text::Text, widgets::{Block, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph, StatefulWidget}, Frame
};

use crate::model::{Model, CurrentPanel};

pub fn view(f: &mut Frame, model: &mut Model) {
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
            Constraint::Max(10),
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
    if model.current_panel == CurrentPanel::Method {
        let border_set = symbols::border::Set {
            top_left: symbols::line::VERTICAL_RIGHT,
            top_right: symbols::line::VERTICAL_LEFT,
            ..symbols::border::PLAIN
        };
        let block = Block::default().border_set(border_set).borders(Borders::ALL).border_style(active_style);
        let items = [
            ListItem::new("GET"),
            ListItem::new("POST"),
            ListItem::new("DELETE"),
            ListItem::new("PUT"),
            ListItem::new("PATCH")
        ];

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::Red),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
        let popup = method_selector(f.size());
        f.render_widget(Clear, popup);
        StatefulWidget::render(list, popup, f.buffer_mut(), &mut model.list_state);
    }
}

fn method_selector(r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Max(2),
        Constraint::Max(7),
        Constraint::Min(1),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Max(10),
        Constraint::Min(1),
    ])
    .split(popup_layout[1])[0]
}