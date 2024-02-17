use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Text,
    widgets::{Block, Borders, Clear, HighlightSpacing, List, Paragraph, StatefulWidget, Wrap},
};

use crate::model::{Model, CurrentPanel, METHODS};

pub fn view(f: &mut Frame, model: &mut Model) {
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Max(10),
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

    let mut body_block = Block::default()
        .title("Body")
        .borders(Borders::ALL)
        .style(Style::default());

    let mut output_block = Block::default()
        .title("Output")
        .borders(Borders::ALL)
        .style(Style::default());

    let active_style = Style::default().fg(Color::Blue);

    match model.current_panel {
        CurrentPanel::Method => {
            method_block = method_block.border_style(active_style);
            model.set_cursor(1 + model.current_method().to_string().len() as u16, 1)
        },
        CurrentPanel::Url => {
            url_block = url_block.border_style(active_style);
            model.set_cursor(top_chunks[1].x + 1 + model.url_input.len() as u16, 1)
        },
        CurrentPanel::Body => {
            body_block = body_block.border_style(active_style);
            model.set_cursor(3, chunks[1].y + 1);
        },
        CurrentPanel::Output => {
            output_block = output_block.border_style(active_style);
            let output_lines = model.output_text.lines();
            let (num_lines, last_line) = output_lines.fold((0, None), |(count, _), elem| {
                (count + 1, Some(elem))
            });
            model.set_cursor(1 + last_line.unwrap_or("").len() as u16, chunks[2].y + num_lines + 1);
        },
    };

    let method_text = Paragraph::new(Text::styled(
        model.current_method().to_string().clone(),
        Style::default().fg(Color::Green),
    ))
    .block(method_block);
    let url_text = Paragraph::new(model.url_input.clone()).block(url_block);
    let body_text = Paragraph::new("{}").block(body_block);
    let output_text = Paragraph::new(model.output_text.clone()).wrap(Wrap { trim: false }).block(output_block);

    f.render_widget(method_text, top_chunks[0]);
    f.render_widget(url_text, top_chunks[1]);
    f.render_widget(body_text, chunks[1]);
    f.render_widget(output_text, chunks[2]);

    if model.current_panel == CurrentPanel::Method {
        let border_set = symbols::border::Set {
            top_left: symbols::line::VERTICAL_RIGHT,
            top_right: symbols::line::VERTICAL_LEFT,
            ..symbols::border::PLAIN
        };
        let block = Block::default().border_set(border_set).borders(Borders::ALL).border_style(active_style);

        let items = METHODS.map(|method| {String::from(method.as_str())});
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

    f.set_cursor(model.cursor_col, model.cursor_row);
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
