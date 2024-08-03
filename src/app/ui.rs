use tui::widgets::Paragraph;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::{app::InputMode, App};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // define the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(1), Constraint::Min(1)].as_ref())
        .split(f.size());

    // define the help message
    let (msg, style) = match app.input_mode {
        // Help message for normal mode
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to execute, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        // Help message for input mode
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to execute"),
            ],
            Style::default(),
        ),
    };

    // render the help message
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    // render cursor based on current input mode
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + 1 + app.position as u16,
                // Move one line down, from the border to the input line
                chunks[1].y + 1 + 2 * app.state.selected().unwrap() as u16,
            )
        }
    }

    // render the table
    let rows = app.items.chunks(1).map(|item| {
        let height = item
            .iter()
            .filter(|element| !element.is_empty())
            .map(|content| content.chars().filter(|c| *c == '#').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.clone()));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });

    // Make selected row bold
    let selected_style = Style::default().add_modifier(Modifier::BOLD);
    let t = Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Commands"))
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Percentage(100),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, chunks[1], &mut app.state);
}
