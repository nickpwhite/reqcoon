use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, Padding, Paragraph, Row,
        StatefulWidget, Table, Wrap,
    },
    Frame,
};

use crate::model::{InputField, InputType, Mode, Model, Panel, METHODS};

pub fn view(f: &mut Frame, model: &mut Model) {
    // Create the layout sections.
    let [top_section, input_section, output_section] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
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

    f.render_widget(method_block(model), method_section);
    f.render_widget(url_block(model), url_section);
    f.render_widget(input_block(model), input_section);
    f.render_widget(output_block(model), output_section);

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
        Panel::Input => match model.current_input_field {
            InputField::Key => (3, input_section.y + 4),
            InputField::Value => (input_section.width / 2 + 1, input_section.y + 4),
        },
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

fn input_block(model: &Model) -> Table {
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

    let rows = model.current_input()
        .iter()
        .map(|[key, value]| Row::new(vec![key.value(), value.value()]));

    Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
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
