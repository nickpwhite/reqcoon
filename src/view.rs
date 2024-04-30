use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, Paragraph, Row, Table, TableState, Widget},
    Frame,
};

use crate::{
    model::{BodyFormat, InputField, InputType, Model, Panel},
    text_wrapping::{truncate_ellipse, wrap_string},
};

pub fn view(f: &mut Frame, model: &mut Model) {
    // Create the layout sections.
    let [top_section, input_section, output_section, statusbar_section] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(30),
            Constraint::Percentage(70),
            Constraint::Length(1),
        ])
        .areas(f.size());

    let [method_section, url_section] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Max(12), Constraint::Min(1)])
        .areas(top_section);

    let input_field_width = (input_section.width - 6) / 2 - 1;

    f.render_widget(method_block(model), method_section);
    f.render_widget(url_block(model), url_section);
    f.render_widget(output_block(model), output_section);
    f.render_widget(mode_block(model), statusbar_section);

    let mut table_state = TableState::default().with_selected(model.input_index);
    f.render_stateful_widget(
        input_block(model, input_field_width as usize),
        input_section,
        &mut table_state,
    );

    let (col, row) = match model.current_panel {
        Panel::Method => (model.method_cursor_position(), 1),
        Panel::Url => (model.cursor_col() + url_section.x + 1, 1),
        Panel::Input => {
            let start_col = match model.current_input_field {
                InputField::Key => 3,
                InputField::Value => input_section.width / 2 + 1,
            };
            let input_row = model.cursor_col() / input_field_width;
            (
                start_col + model.cursor_col() % input_field_width,
                (model.input_index - table_state.offset()) as u16 + input_section.y + 4 + input_row,
            )
        }
        Panel::Output => {
            let (scroll_row, scroll_col) = model.output_textarea.viewport.scroll_top();
            let (row, col) = model.output_textarea.cursor();
            (
                col as u16 - scroll_col + 1,
                row as u16 - scroll_row + output_section.y + 1,
            )
        }
    };

    f.set_cursor(col, row);
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
        model.current_method.to_string().clone(),
        Style::default().fg(Color::Green),
    ))
    .block(method_block)
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
        .title_bottom(input_footer(model))
        .borders(Borders::ALL)
        .border_style(style)
        .padding(Padding::proportional(1));

    model
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
        })
        .collect::<Table>()
        .widths([Constraint::Percentage(50), Constraint::Percentage(50)])
        .block(input_block)
        .header(Row::new(vec!["Key", "Value"]).bottom_margin(1))
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

fn input_footer(model: &Model) -> Line<'static> {
    let mut json_title = BodyFormat::Json.to_string().white();
    let mut form_title = BodyFormat::Form.to_string().white();
    if model.current_panel == Panel::Input {
        match model.current_body_format {
            BodyFormat::Json => json_title = json_title.blue(),
            BodyFormat::Form => form_title = form_title.blue(),
        };
    }

    if model.current_input_type == InputType::Body {
        Line::default().spans(vec![
            Span::styled("| ", Color::White),
            json_title,
            Span::styled(" | ", Color::White),
            form_title,
            Span::styled(" |", Color::White),
        ])
    } else {
        Line::default()
    }
}

fn output_block(model: &mut Model) -> impl Widget + '_ {
    let style = if model.current_panel == Panel::Output {
        active_style()
    } else {
        Style::default()
    };

    let output_block = Block::default()
        .title("Output")
        .borders(Borders::ALL)
        .border_style(style);

    model
        .output_textarea
        .set_cursor_line_style(Style::default());
    model.output_textarea.set_cursor_style(Style::default());
    model.output_textarea.set_block(output_block);

    model.output_textarea.widget()
}

fn mode_block(model: &Model) -> Paragraph {
    Paragraph::new(format!(
        "{mode} {message}",
        mode = model.current_mode.to_string(),
        message = model.message
    ))
}
