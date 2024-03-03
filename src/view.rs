use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, Padding, Paragraph, Row, StatefulWidget,
        Table, TableState, Wrap,
    },
    Frame,
};

use crate::{
    model::{InputField, InputType, Mode, Model, Panel, METHODS},
    text_wrapping::{truncate_ellipse, wrap_string},
};

pub fn view(f: &mut Frame, model: &mut Model) {
    // Create the layout sections.
    let [top_section, input_section, output_section, statusbar_section] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
            Constraint::Length(1),
        ])
        .areas(f.size());

    let [method_section, url_section] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Max(12), Constraint::Min(1)])
        .areas(top_section);

    let [method_selector_section, _] =
        Layout::horizontal([Constraint::Max(12), Constraint::Min(1)]).areas(
            Layout::vertical([Constraint::Max(2), Constraint::Max(7), Constraint::Min(1)])
                .split(f.size())[1],
        );

    let input_field_width = (input_section.width - 6) / 2 - 1;

    f.render_widget(method_block(model), method_section);
    f.render_widget(url_block(model), url_section);
    f.render_widget(output_block(model), output_section);
    f.render_widget(statusbar_block(model), statusbar_section);

    let mut table_state = TableState::default().with_selected(model.input_index);
    f.render_stateful_widget(
        input_block(model, input_field_width as usize),
        input_section,
        &mut table_state,
    );

    if model.current_panel == Panel::Method && model.current_mode == Mode::Insert {
        f.render_widget(Clear, method_selector_section);
        StatefulWidget::render(
            method_selector(),
            method_selector_section,
            f.buffer_mut(),
            &mut model.list_state,
        );
    }

    let (col_offset, row_offset) = match model.current_panel {
        Panel::Method => (1, 1),
        Panel::Url => (url_section.x + 1, 1),
        Panel::Input => {
            let col = match model.current_input_field {
                InputField::Key => 3,
                InputField::Value => input_section.width / 2 + 1,
            };
            let input_col = model.current_input().visual_cursor() as u16 % input_field_width;
            let input_row = model.current_input().visual_cursor() as u16 / input_field_width;
            (
                col + input_col,
                input_section.y + 4 + input_row - (table_state.offset() as u16),
            )
        }
        Panel::Output => (0, output_section.y),
    };

    f.set_cursor(model.cursor_col + col_offset, model.cursor_row + row_offset);
}

fn active_style() -> Style {
    Style::default().fg(Color::Blue)
}

fn method_block(model: &Model) -> Paragraph {
    let style = if model.current_panel == Panel::Method {
        active_style()
    } else {
        Style::default()
    };

    let method_block = Block::default()
        .title("Method")
        .borders(Borders::ALL)
        .border_style(style);

    Paragraph::new(Text::styled(
        model.current_method().to_string().clone(),
        Style::default().fg(Color::Green),
    ))
    .block(method_block)
}

fn method_selector() -> List<'static> {
    let border_set = symbols::border::Set {
        top_left: symbols::line::VERTICAL_RIGHT,
        top_right: symbols::line::VERTICAL_LEFT,
        ..symbols::border::PLAIN
    };
    let block = Block::default()
        .border_set(border_set)
        .borders(Borders::ALL)
        .border_style(active_style());

    let items = METHODS.map(|method| String::from(method.as_str()));

    List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Red),
        )
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always)
}

fn url_block(model: &Model) -> Paragraph {
    let style = if model.current_panel == Panel::Url {
        active_style()
    } else {
        Style::default()
    };

    let url_block = Block::default()
        .title("URL")
        .borders(Borders::ALL)
        .border_style(style);

    Paragraph::new(model.url_input.value()).block(url_block)
}

fn input_block(model: &Model, field_width: usize) -> Table {
    let style = if model.current_panel == Panel::Input {
        active_style()
    } else {
        Style::default()
    };

    let input_block = Block::default()
        .title(input_title(model))
        .borders(Borders::ALL)
        .border_style(style)
        .padding(Padding::proportional(1));

    let table = Table::default()
        .widths([Constraint::Percentage(50), Constraint::Percentage(50)])
        .block(input_block)
        .header(Row::new(vec!["Key", "Value"]).bottom_margin(1));

    let rows = model
        .current_input_table()
        .iter()
        .enumerate()
        .map(|(i, input_row)| {
            let (key, value) = if model.current_panel == Panel::Input && model.input_index == i {
                match model.current_input_field {
                    InputField::Key => (
                        wrap_string(input_row.key.value(), field_width),
                        truncate_ellipse(input_row.value.value(), field_width),
                    ),
                    InputField::Value => (
                        truncate_ellipse(input_row.key.value(), field_width),
                        wrap_string(input_row.value.value(), field_width),
                    ),
                }
            } else {
                (
                    truncate_ellipse(input_row.key.value(), field_width),
                    truncate_ellipse(input_row.value.value(), field_width),
                )
            };
            let height = std::cmp::max(key.lines().count(), value.lines().count()) as u16;

            Row::new(vec![key, value]).height(height)
        });

    table.rows(rows)
}

fn input_title(model: &Model) -> Line<'static> {
    let mut headers_title = InputType::Headers.to_string().white();
    let mut body_title = InputType::Body.to_string().white();
    if model.current_panel == Panel::Input {
        match model.current_input_type {
            InputType::Headers => headers_title = headers_title.blue(),
            InputType::Body => body_title = body_title.blue(),
        };
    }

    Line::default().spans(vec![
        Span::styled("| ", Color::White),
        headers_title,
        Span::styled(" | ", Color::White),
        body_title,
        Span::styled(" |", Color::White),
    ])
}

fn output_block(model: &Model) -> Paragraph {
    let style = if model.current_panel == Panel::Output {
        active_style()
    } else {
        Style::default()
    };

    let output_block = Block::default()
        .title("Output")
        .borders(Borders::ALL)
        .border_style(style);

    Paragraph::new(model.output_text.clone())
        .wrap(Wrap { trim: false })
        .block(output_block)
}

fn statusbar_block(model: &Model) -> Paragraph {
    Paragraph::new(model.current_mode.to_string())
}
