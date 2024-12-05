use crate::board::{render_bitboard, Board};
use crate::moves::{compute_knights_moves, compute_pawns_moves};

/// Game struct responsible for all game logics (pin, check, valid captures, etc)
pub struct Game {
    pub board: Board,
}

impl Game {
    pub fn new(board: Board) -> Game {
        Game {
            board,
        }
    }

    pub fn move_knight(&mut self, from: u64, to: u64, is_white: bool) -> bool {
        let pseudolegal_knight_moves = compute_knights_moves(&self.board, is_white);
        let is_capture = self.board.is_capture(to, is_white);
        let knights = if is_white {
            self.board.white_knights
        } else {
            self.board.black_knights
        };

        // from is valid (from current knights)
        if (from & knights) == 0 {
            println!("NO");
            return false;
        }
        // check pseudolegal moves
        if (to & pseudolegal_knight_moves) == 0 {
            println!("NO2");
            return false;
        }

        if is_capture {
            self.board.move_piece(from, to, is_white);
            self.board.remove_piece(to, !is_white);

            // TODO check for check
            true
        } else {
            // Normal move
            // TODO check for pin
            self.board.move_piece(from, to, is_white);
            true
        }
    }

    pub fn move_pawn(&mut self, from: u64, to: u64, is_white: bool) -> bool {
        let pseudolegal_pawn_moves = compute_pawns_moves(&self.board, is_white);
        let is_capture = self.board.is_capture(to, is_white);

        let pawns = if is_white {
            self.board.white_pawns
        } else {
            self.board.black_pawns
        };

        println!("* is_white={}", is_white);
        render_bitboard(&pseudolegal_pawn_moves, 'L');

        // from is valid (from current pawns)
        if (from & pawns) == 0 {
            return false;
        }
        // check pseudolegal moves
        if (to & pseudolegal_pawn_moves) == 0 {
            return false;
        }

        // TODO check for actual legal moves

        if is_capture {
            self.board.move_piece(from, to, is_white);
            self.board.remove_piece(to, !is_white);

            // TODO check for check
            true
        } else {
            // Normal move
            // TODO check for pin
            self.board.move_piece(from, to, is_white);
            true
        }
    }
}


impl Default for Game {
    fn default() -> Game {
        Self::new(Board::default())
    }
}


#[cfg(test)]
pub mod tests {
    use crate::board::{bit_pos, bitboard_single, render_bitboard, Board, PositionBuilder};
    use super::*;
}