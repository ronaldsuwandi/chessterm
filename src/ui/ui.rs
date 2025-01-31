use std::ops::Add;
use ratatui::buffer::Buffer;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position, Rect};
use ratatui::prelude::{Line, Stylize, Text, Widget};
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui_image::{Image, StatefulImage};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::ImageSource;
use crate::ui::app::{App, CurrentScreen};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Yoo ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            // self.counter.to_string().yellow()
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);

    }
}

fn render_board(app: &App, frame: &mut Frame, area: Rect) {
    // for (k, mut v) in app.chess_pieces {
    // let stateful_image = StatefulImage::default();
    // let img = app.chess_pieces.get_mut(&'q');
    // frame.render_stateful_widget(stateful_image, area, img.unwrap());
    // }
    let board_area = frame.size();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Ratio(1, 8); 8]) // 8 equal rows
        .split(board_area);

    for (row_idx, &row) in rows.iter().enumerate() {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 8); 8]) // 8 equal columns
            .split(row);

        for (col_idx, &col) in cols.iter().enumerate() {
            let is_dark_square = (row_idx + col_idx) % 2 == 1;
            let square_color = if is_dark_square { Color::DarkGray } else { Color::White };

            let square = Block::default()
                .title(Span::styled(" ", Style::default().fg(square_color)))
                .borders(Borders::ALL);

            frame.render_widget(square, col);
        }
    }
}

pub fn ui(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "chessterm 0.0.1",
        Style::default().fg(Color::Green),
    )).alignment(Alignment::Center)
    .block(title_block);
    frame.render_widget(title, main_layout[0]);


    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Max(50),
        ])
        .split(main_layout[1]);


    let chessboard_block = Block::default()
        .borders(Borders::ALL);
    let chess = Paragraph::new(
        Text::styled("dummy board", Style::default().fg(Color::Blue)))
            .block(chessboard_block);


    let moves_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(3),
            Constraint::Fill(1),
        ])
        .split(content_layout[1]);


    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Input");

    let input = Paragraph::new(Text::styled(app.input.as_str(), Style::default().fg(Color::White))).block(input_block);
    frame.render_widget(chess, content_layout[0]);
    frame.render_widget(input, moves_layout[0]);
    // render_board(app, frame, chunks[1]);


    frame.set_cursor_position(Position::new(
        moves_layout[0].x + app.character_index as u16 + 1,
        moves_layout[0].y + 1,
    ));


    let moves = Block::default()
        .borders(Borders::ALL)
        .title("Moves");
    frame.render_widget(moves, moves_layout[1]);

    let current_navigation_text = vec![
        // The first half of the text
        match app.current_screen {
            CurrentScreen::Main => Span::styled("Normal Mode", Style::default().fg(Color::Green)),
            CurrentScreen::Exiting => Span::styled("Exiting", Style::default().fg(Color::LightRed)),
            CurrentScreen::GameOver => Span::styled("Game Over", Style::default().fg(Color::LightRed)),
        }.to_owned(),


        // A white divider bar to separate the two sections
        Span::styled(" | ", Style::default().fg(Color::White)),
        Span::styled(app.input.as_str(), Style::default().fg(Color::White)),
    ];


    let footer = Paragraph::new(
        Text::styled("Press ESC to quit", Style::default().fg(Color::Gray)))
        .alignment(Alignment::Center)
        .block(Block::default());

    frame.render_widget(footer, main_layout[2]);


    if let CurrentScreen::Exiting = app.current_screen {
        let popup_block = Block::default()
            .title("Confirm exit game")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::DarkGray));

        let exit_text = Text::styled(
            "Confirm exit game? (y/n)",
            Style::default().fg(Color::Black),
        );

        // the `trim: false` will stop the text from being cut off when over the edge of the block
        let exit_paragraph = Paragraph::new(exit_text)
            .alignment(Alignment::Center)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(40, 10, frame.area());
        frame.render_widget(exit_paragraph, area);
    }
}


/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}


// Ne3xf4
// O-O-O