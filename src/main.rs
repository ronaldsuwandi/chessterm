// use crossterm::{l
//     cursor, execute,
//     style::{self, Color, Print, SetBackgroundColor, SetForegroundColor},
//     terminal, ExecutableCommand,
// };
// use std::io::{stdout, Write};
//
// fn draw_chessboard() ->  Result<(), Box<dyn std::error::Error>> {
//     let mut stdout = stdout();
//
//     // Define piece symbols and initial board state
//     let pieces = vec![
//         vec!["♖", "♘", "♗", "♕", "♔", "♗", "♘", "♖"],
//         vec!["♙"; 8],
//         vec![" "; 8],
//         vec![" "; 8],
//         vec![" "; 8],
//         vec![" "; 8],
//         vec!["♟"; 8],
//         vec!["♜", "♞", "♝", "♛", "♚", "♝", "♞", "♜"],
//     ];
//
//     // Draw chessboard
//     stdout.execute(terminal::Clear(terminal::ClearType::All))?;
//     for (row_idx, row) in pieces.iter().enumerate() {
//         for (col_idx, &piece) in row.iter().enumerate() {
//             // Alternate background colors
//             let background_color = if (row_idx + col_idx) % 2 == 0 {
//                 Color::White
//             } else {
//                 Color::Black
//             };
//             let foreground_color = if (row_idx + col_idx) % 2 == 0 {
//                 Color::Red
//             } else {
//                 Color::Red
//             };
//
//             // Move cursor and draw square
//             stdout
//                 .execute(cursor::MoveTo(col_idx as u16 * 2, row_idx as u16))?
//                 .execute(SetBackgroundColor(background_color))?
//                 .execute(SetForegroundColor(foreground_color))?
//                 .execute(Print(format!(" {} ", piece)))?;
//         }
//         stdout.execute(Print("\n"))?;
//     }
//
//     stdout.flush()?;
//     Ok(())
// }

use crate::board::{bitboard_single, render_bitboard, Board};
use crate::game::Game;

mod board;
mod moves;
mod game;

fn main() {
    // draw_chessboard().unwrap();

    // let idx = bit_pos('f', 2).unwrap();

    // let mut bitboard: u64 = 1 << 53;
    // bitboard = bitboard | (1 << idx);

    // board::render_bitboard(&bitboard, '♟');

    // let file = 'e';
    // let rank = 2u64;

    // println!("bit_pos for {}{} = {:?}", file, rank, bit_pos(file, rank));

    let mut game = Game::default();
    game.board.render();
    game.move_pawn(bitboard_single('e',2).unwrap(), bitboard_single('e',4).unwrap(), true);
    // board.move_pawn(bitboard_single('e',1).unwrap(), bitboard_single('e',3).unwrap(), true); // ignored
    game.board.render();
    game.move_pawn(bitboard_single('f',7).unwrap(), bitboard_single('f',5).unwrap(), false);
    game.board.render();
    println!("black");

    render_bitboard(&game.board.black_pawns, 'b');

    // println!("CAPTURE F5");
    // println!("{}", game.move_pawn(bitboard_single('e',4).unwrap(), bitboard_single('f',5).unwrap(), true));
    game.board.render();
    game.move_knight(bitboard_single('g',8).unwrap(), bitboard_single('f',6).unwrap(), false);
    game.board.render();
    // println!("black2");
    // render_bitboard(&game.board.black_pawns, 'b');
}
