use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, HighlightSpacing, List, Paragraph, Row, StatefulWidget, Table, Wrap},
};

use crate::model::{Model, CurrentInputType, CurrentPanel, METHODS};

pub fn view(f: &mut Frame, model: &mut Model) {
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
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

    let mut input_block = Block::default()
        .title(input_title(model))
        .borders(Borders::ALL);

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
            model.set_cursor(top_chunks[1].x + 1 + model.url_input.visual_cursor() as u16, 1);
        },
        CurrentPanel::Input => {
            input_block = input_block.border_style(active_style);
            model.set_cursor(1 + model.body_input.visual_cursor() as u16, chunks[1].y + 1);
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
    let url_input = Paragraph::new(model.url_input.value()).block(url_block);
    let input_table = Table::default()
        .style(Style::new().white())
        .rows([Row::new(vec![model.current_header_key.value(), model.current_header_value.value()])])
        .header(Row::new(vec!["Name", "Value"]))
        .block(input_block);
    let output_text = Paragraph::new(model.output_text.clone()).wrap(Wrap { trim: false }).block(output_block);

    f.render_widget(method_text, top_chunks[0]);
    f.render_widget(url_input, top_chunks[1]);
    f.render_widget(input_table, chunks[1]);
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

fn input_title(model: &mut Model) -> Line<'static> {
    let mut headers_title = CurrentInputType::Headers.to_string().white();
    let mut auth_title = CurrentInputType::Auth.to_string().white();
    let mut body_title = CurrentInputType::Body.to_string().white();
    if model.current_panel == CurrentPanel::Input {
        match model.current_input_type {
            CurrentInputType::Headers => headers_title = headers_title.blue(),
            CurrentInputType::Auth => auth_title = auth_title.blue(),
            CurrentInputType::Body => body_title = body_title.blue(),
        };
    }

    Line::default().spans(vec![
        Span::styled("| ", Color::White),
        headers_title,
        Span::styled(" | ", Color::White),
        auth_title,
        Span::styled(" | ", Color::White),
        body_title,
        Span::styled(" |", Color::White),
    ])
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
