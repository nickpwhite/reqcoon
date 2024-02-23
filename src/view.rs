use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Text,
    widgets::{Block, Borders, Clear, HighlightSpacing, List, Paragraph, StatefulWidget, Wrap},
    Frame,
};

use crate::model::{CurrentPanel, Model, METHODS};

pub fn view(f: &mut Frame, model: &mut Model) {
    // Create the layout sections.
    let [top_section, input_section, output_section] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Max(10),
            Constraint::Min(1),
        ])
        .areas(f.size());

    let [method_section, url_section] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Max(10), Constraint::Min(1)])
        .areas(top_section);

    let [method_selector_section, _] =
        Layout::horizontal([Constraint::Max(10), Constraint::Min(1)]).areas(
            Layout::vertical([Constraint::Max(2), Constraint::Max(7), Constraint::Min(1)])
                .split(f.size())[1],
        );

    f.render_widget(method_block(model), method_section);
    f.render_widget(url_block(model), url_section);
    f.render_widget(input_block(model), input_section);
    f.render_widget(output_block(model), output_section);

    if model.current_panel == CurrentPanel::Method {
        f.render_widget(Clear, method_selector_section);
        StatefulWidget::render(
            method_selector(),
            method_selector_section,
            f.buffer_mut(),
            &mut model.list_state,
        );
    }

    let (col_offset, row_offset) = match model.current_panel {
        CurrentPanel::Method => (0, 0),
        CurrentPanel::Url => (url_section.x, 0),
        CurrentPanel::Input => (0, input_section.y),
        CurrentPanel::Output => (0, output_section.y),
    };

    f.set_cursor(model.cursor_col + col_offset, model.cursor_row + row_offset);
}

fn active_style() -> Style {
    Style::default().fg(Color::Blue)
}

fn method_block(model: &Model) -> Paragraph {
    let style = if model.current_panel == CurrentPanel::Method {
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
    let style = if model.current_panel == CurrentPanel::Url {
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

fn input_block(model: &Model) -> Paragraph {
    let style = if model.current_panel == CurrentPanel::Input {
        active_style()
    } else {
        Style::default()
    };

    let input_block = Block::default()
        .title("Body")
        .borders(Borders::ALL)
        .border_style(style);

    Paragraph::new("{}").block(input_block)
}

fn output_block(model: &Model) -> Paragraph {
    let style = if model.current_panel == CurrentPanel::Output {
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
