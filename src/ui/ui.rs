use crate::engine::game::MoveError;
use crate::ui::app::{App, CurrentScreen};
use image::imageops::FilterType;
use ratatui::buffer::Buffer;
use ratatui::layout::{
    Alignment, Constraint, Direction, Layout, Margin, Offset, Position, Rect, Size,
};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table,
    Wrap,
};
use ratatui::Frame;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::ImageSource;
use ratatui_image::{Image, Resize, StatefulImage};
use std::cmp::min;
use std::ops::Add;
use std::rc::Rc;

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

const LIGHT_SQUARE: Color = Color::Rgb(235, 209, 166);
const DARK_SQUARE: Color = Color::Rgb(165, 117, 80);

const DEFAULT_SQUARE_SIZE: u16 = 12;
const LARGE_SQUARE_SIZE: u16 = 15;

/// compute board layouts returning tuple of 3 rects:
/// - rank_layout[8] for the actual board
/// - rank_label_layout[9] for label on rank
/// - file_label_layout[9] for label on file
/// label layout has additional item because it's a combination of the actual
/// rank layout with buffer for the label
fn compute_board_layouts(area: Rect, square_size: u16) -> (Rc<[Rect]>, Rc<[Rect]>, Rc<[Rect]>) {
    // add additional file for label
    let board_horizontal =
        Layout::horizontal([Constraint::Length(3), Constraint::Length(square_size * 8)])
            .split(area);

    // add additional row for label
    let board_vertical = Layout::vertical([
        Constraint::Length((square_size / 2) * 8),
        Constraint::Length(1),
    ])
    .split(board_horizontal[1]);

    let rank_constraints = [Constraint::Length(square_size / 2); 8];
    let rank_layout = Layout::vertical(rank_constraints).split(board_vertical[0]);

    // copy rank layout setup (vertical spacing)
    let mut rank_layout_constraints = [Constraint::Length(square_size / 2); 9];
    rank_layout_constraints[8] = Constraint::Length(1);

    let rank_label_layout = Layout::vertical(rank_layout_constraints).split(board_horizontal[0]);

    let file_label_layout =
        Layout::horizontal([Constraint::Length(square_size); 8]).split(board_vertical[1]);

    (rank_layout, rank_label_layout, file_label_layout)
}

fn render_rank_label(frame: &mut Frame, rank: usize, area: Rect) {
    let rank_label = Paragraph::new(format!("{}", rank))
        .fg(Color::Yellow)
        .bold()
        .alignment(Alignment::Center);
    frame.render_widget(rank_label, area);
}

fn render_file_labels(frame: &mut Frame, file_label_layout: Rc<[Rect]>, flipped: bool) {
    for file in 0..8 {
        let actual_file = if flipped { 7 - file } else { file }; // Flip files

        let f = (actual_file as u8 + 'A' as u8) as char;
        let file_label = Paragraph::new(format!("{}", f))
            .fg(Color::Yellow)
            .bold()
            .alignment(Alignment::Left);
        frame.render_widget(file_label, file_label_layout[file])
    }
}

fn actual_file(file: usize, flipped: bool) -> usize {
    if flipped {
        7 - file
    } else {
        file
    } // Flip files
}

fn actual_rank(rank: usize, flipped: bool) -> usize {
    if flipped {
        rank
    } else {
        7 - rank
    } // Flip ranks
}

fn render_square(
    frame: &mut Frame,
    file_layout: &Rc<[Rect]>,
    rank: usize,
    file: usize,
    flipped: bool,
) {
    let actual_file = actual_file(file, flipped);

    let is_white = (rank + file) & 1 == 1;
    let bg = if is_white { LIGHT_SQUARE } else { DARK_SQUARE };

    let square = Block::default().bg(bg);
    frame.render_widget(square, file_layout[actual_file]);
}

fn render_piece(
    frame: &mut Frame,
    app: &App,
    file_layout: &Rc<[Rect]>,
    file: usize,
    piece: char,
    flipped: bool,
) {
    let actual_file = actual_file(file, flipped);

    if piece != '.' {
        let protocol_ref = app.chess_pieces.get(&piece).unwrap();
        let i = StatefulImage::default();
        frame.render_stateful_widget(i, file_layout[actual_file], &mut protocol_ref.borrow_mut());
    }
}

fn render_board(app: &App, frame: &mut Frame, area: Rect, large_board: bool) {
    let square_size = if large_board {
        LARGE_SQUARE_SIZE
    } else {
        DEFAULT_SQUARE_SIZE
    };

    let (rank_layout, rank_label_layout, file_label_layout) =
        compute_board_layouts(area, square_size);
    let pieces = app.game.board.pieces_array(false);
    for (rank, files) in pieces.iter().enumerate().rev() {
        let actual_rank = actual_rank(rank, app.flipped);
        let rank_layout_idx = actual_rank; // in reverse order for rendering

        let file_layout = Layout::horizontal([Constraint::Length(square_size); 8])
            .split(rank_layout[rank_layout_idx]);

        render_rank_label(frame, rank + 1, rank_label_layout[rank_layout_idx]);

        // iterate files
        for (file, piece) in files.iter().enumerate() {
            render_square(frame, &file_layout, rank, file, app.flipped);
            render_piece(frame, app, &file_layout, file, *piece, app.flipped);
        }
    }
    render_file_labels(frame, file_label_layout, app.flipped);
}

pub const MIN_WIDTH_LARGE: u16 = 164;
pub const MIN_HEIGHT_LARGE: u16 = 62;

fn large_board(frame: &Frame) -> bool {
    let size = frame.area();
    size.width >= MIN_WIDTH_LARGE && size.height >= MIN_HEIGHT_LARGE
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let large_board = large_board(frame);
    // number needs to be divisible by 8 (+1 row for label)
    let board_vertical = if large_board { 57 } else { 41 };

    let main_layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(board_vertical), // use fixed size for divisible by 8 (add extra 1 row for label)
        Constraint::Fill(1),                // filler
        Constraint::Length(2),
    ])
    .split(frame.area());

    // divisible by 8 + 3 pixel for label
    let board_horizontal = if large_board { 125 } else { 100 };
    let content_layout = Layout::horizontal([
        Constraint::Fill(1), // filler
        Constraint::Min(board_horizontal),
        Constraint::Length(40),
        Constraint::Fill(1), // filler
    ])
    .split(main_layout[1]);

    render_title(frame, main_layout[0]);
    render_board(app, frame, content_layout[1], large_board);
    render_moves(frame, app, content_layout[2]);
    render_footer(frame, main_layout[3]);

    match app.current_screen {
        CurrentScreen::Main => {}
        CurrentScreen::Exiting => {
            let popup_block = Block::default()
                .title("Confirm exit game")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center)
                .style(Style::default().bg(Color::DarkGray));

            let exit_text = Text::styled(
                "\nConfirm exit game? (y/n)",
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

            let exit_text = Text::styled("Play again? (y/n)", Style::default().fg(Color::Black));

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

fn render_title(frame: &mut Frame, area: Rect) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "chessterm 0.0.1",
        Style::default().fg(Color::Green),
    ))
    .alignment(Alignment::Center)
    .block(title_block);
    frame.render_widget(title, area);
}

fn render_moves(frame: &mut Frame, app: &mut App, area: Rect) {
    let moves_layout = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(area);

    let input_block = Block::default().title("Input").borders(Borders::ALL);

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
        .height(1);

    let rows: Vec<Row> = app
        .moves
        .chunks(2)
        .enumerate()
        .map(|(i, chunk)| {
            let white_move = chunk
                .get(0)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string());
            let black_move = chunk
                .get(1)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string());
            Row::new([format!("{}", i + 1), white_move, black_move])
        })
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ];

    let moves = Block::default().borders(Borders::ALL).title("Moves");

    // update scrollbar state
    app.scrollbar_state = app
        .scrollbar_state
        .content_length(rows.len())
        .position(app.scroll_offset);
    *app.table_state.offset_mut() = app.scroll_offset;

    app.visible_moves = (moves_layout[1].height as usize).saturating_sub(3);

    let table = Table::new(rows, widths).header(header).block(moves);
    frame.render_stateful_widget(table, moves_layout[1], &mut app.table_state);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"));

    if app.show_scrollbar {
        frame.render_stateful_widget(
            scrollbar,
            moves_layout[1].inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut app.scrollbar_state,
        );
    }
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        "[.]".blue().bold(),
        " Flip  ".into(),
        "[▲ / ▼]".blue().bold(),
        " Scroll moves  ".into(),
        "[ESC]".blue().bold(),
        " Quit".into(),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default());

    frame.render_widget(footer, area);
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

pub fn render_size_error(frame: &mut Frame, min_width: u16, min_height: u16, current_size: Size) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let paragraph = Paragraph::new(Line::from(vec![
        "Terminal size too small. Minimium ".into(),
        format!("{min_width}").bold().yellow(),
        Span::from("x"),
        format!("{min_height}").bold().yellow(),
        ". Current size is ".into(),
        format!("{}", current_size.width).bold().red(),
        "x".into(),
        format!("{}", current_size.height).bold().red(),
    ]));
    frame.render_widget(paragraph, area);
}
