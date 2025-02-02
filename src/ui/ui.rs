use std::ops::Add;
use image::imageops::FilterType;
use ratatui::buffer::Buffer;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Offset, Position, Rect};
use ratatui::prelude::{Line, Stylize, Text, Widget};
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Clear, Padding, Paragraph, Row, Table, Wrap};
use ratatui_image::{Image, Resize, StatefulImage};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::ImageSource;
use crate::engine::game::MoveError;
use crate::ui::app::{App, CurrentScreen};

const ERROR_MOVE: &str = "×";
const ERROR_AMBIGUOUS: &str = "? Ambiguous";
const ERROR_NONE: &str = "";

fn render_error<'a>(err: Option<MoveError>) -> Span<'a> {
    if let Some(err) = err {
        if err == MoveError::AmbiguousSource {
            Span::from(ERROR_AMBIGUOUS).style(Style::default().fg(Color::Yellow).bold())
        } else {
            Span::from(ERROR_MOVE).style(Style::default().fg(Color::Red).bold())
        }
    } else {
        Span::from(ERROR_NONE)
    }
}

fn render_board(app: &App, frame: &mut Frame, area: Rect) {
    let constraint = 12;

    let board_horizontal = Layout::horizontal(
        [Constraint::Length(3), Constraint::Length(constraint * 8)]
    ).split(area);

    let board_vertical = Layout::vertical(
        [Constraint::Length((constraint/2) * 8), Constraint::Length(1)]
    ).split(board_horizontal[1]);

    let rank_constraints = [Constraint::Length(constraint/2); 8];
    let rank_layout = Layout::vertical(rank_constraints).split(board_vertical[0]);

    // copy rank layout setup (vertical spacing)
    let mut rank_layout_constraints = [Constraint::Length(constraint/2); 9];
    rank_layout_constraints[8] = Constraint::Length(1);

    let rank_label_layout = Layout::vertical(rank_layout_constraints).split(board_horizontal[0]);
    let pieces = app.game.board.pieces_array(false);

    for (rank, files) in pieces.iter().enumerate().rev() {
        let actual_rank = if app.flipped { rank } else { 7 - rank }; // Flip ranks
        let rank_layout_idx = actual_rank; // in reverse order for rendering

        let file_layout = Layout::horizontal([Constraint::Length(constraint); 8]).split(rank_layout[rank_layout_idx]);

        let rank_label = Paragraph::new(format!("{}", rank+1))
            .fg(Color::Yellow)
            .bold()
            .alignment(Alignment::Center);
        frame.render_widget(rank_label, rank_label_layout[rank_layout_idx]);

        // iterate files
        for (file, piece) in files.iter().enumerate() {
            let actual_file = if app.flipped { 7 - file } else { file }; // Flip files

            let is_white =  (rank + file) & 1 == 1;
            let bg = if is_white { Color::Rgb(235, 209, 166) } else { Color::Rgb(165, 117, 80) };

            let square = Block::default().bg(bg);

            // let t = Paragraph::new(
            //         Line::from(format!("w={},h={}", file_layout[file].width, file_layout[file].height))
            //         // Line::from(format!("r={rank},f={file},i={piece}"))
            //             .fg(Color::Black)
            //             .bg(Color::Red))
            //     .block(square);
            frame.render_widget(square, file_layout[actual_file]);

            if *piece != '.' {
                let protocol_ref = app.chess_pieces.get(piece).unwrap();
                let i = StatefulImage::default();
                frame.render_stateful_widget(i, file_layout[actual_file], &mut protocol_ref.borrow_mut());
            }
        }
    }

    let file_label_layout = Layout::horizontal([Constraint::Length(constraint); 8])
        .split(board_vertical[1]);

    for file in 0..8 {
        let actual_file = if app.flipped { 7 - file } else { file }; // Flip files

        let f = (actual_file as u8 + 'A' as u8) as char;;
        let file_label = Paragraph::new(format!("{}",f))
            .fg(Color::Yellow)
            .bold()
            .alignment(Alignment::Left);
        frame.render_widget(file_label, file_label_layout[file])
    }
    //     let file_idx = file as u8 - 'a' as u8;
    // Some((rank - 1) * 8 + file_idx as u64)
}

pub fn render(frame: &mut Frame, app: &App) {
    let main_layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(41), // use fixed size for divisible by 8 (add extra 1 row for label)
            Constraint::Fill(1), // filler
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


    let content_layout = Layout::horizontal([
            Constraint::Min(132),
            Constraint::Fill(1),
        ])
        .split(main_layout[1]);


    render_board(app, frame, content_layout[0]);

    let moves_layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(content_layout[1]);


    let input_block = Block::default()
        .title("Input")
        .borders(Borders::ALL);

    let input_texts = vec![
        Span::from(format!("{:<10}", app.input.as_str())).fg(Color::White),
        render_error(app.error),
    ];

    let input = Paragraph::new(Line::from(input_texts)).block(input_block);
    frame.render_widget(input, moves_layout[0]);


    frame.set_cursor_position(Position::new(
        moves_layout[0].x + app.character_index as u16 + 1,
        moves_layout[0].y + 1,
    ));

    // let moves_list =

    let header = ["#", "White", "Black"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        // .style(header_style)
        .height(1);

    let rows: Vec<Row> = app.moves
        .chunks(2)
        .enumerate()
        .map(|(i, chunk)| {
            let white_move = chunk.get(0).map(|s| s.to_string()).unwrap_or_else(|| "".to_string());
            let black_move = chunk.get(1).map(|s| s.to_string()).unwrap_or_else(|| "".to_string());

            Row::new([format!("{}", i + 1), white_move, black_move])
        })
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ];

    let moves = Block::default()
        .borders(Borders::ALL)
        .title("Moves");

    // FIXME handle scrolling
    let table = Table::new(rows, widths)
        .header(header)
        .block(moves);
    frame.render_widget(table, moves_layout[1]);

    let footer = Paragraph::new(
        Line::from(vec![
            "Flip ".into(),
            "[.]".blue().bold(),
            "  Quit ".into(),
            "[ESC]".blue().bold()
        ]))
        .alignment(Alignment::Center)
        .block(Block::default());

    frame.render_widget(footer, main_layout[3]);

    match app.current_screen {
        CurrentScreen::Main => {}
        CurrentScreen::Exiting => {

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
            frame.render_widget(Clear, area); // clear the area behind popup
            frame.render_widget(exit_paragraph, area);
        }
        CurrentScreen::GameOver => {
            let popup_block = Block::default()
                .title("Game over")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center)
                .style(Style::default().bg(Color::DarkGray));

            let exit_text = Text::styled(
                "Play again? (y/n)",
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
