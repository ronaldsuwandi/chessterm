use crate::board::Board;
use crate::moves::{compute_bishops_moves, compute_knights_moves, compute_pawns_moves, compute_rooks_moves};

/// Game struct responsible for all game logics (pin, check, valid captures, etc)
pub struct Game {
    pub board: Board,
    pub turn: u8,
}

impl Game {
    pub fn new(board: Board) -> Game {
        Game {
            board,
            turn: 1,
        }
    }

    fn is_white(&self) -> bool {
        self.turn % 2 == 1
    }

    fn move_piece<F>(
        &mut self,
        from: u64,
        to: u64,
        pieces: u64,
        is_white: bool,
        compute_moves: F,
        is_capture_check: Option<fn(u64) -> bool>,
    ) -> bool
    where
        F: Fn(&Board, bool) -> u64,
    {
        let pseudolegal_moves = compute_moves(&self.board, is_white);
        let is_capture = self.board.is_capture(to, is_white);

        if from == to {
            println!("Invalid from and to square");
            return false;
        }
        // from is valid (from current rooks)
        if (from & pieces) == 0 {
            println!("Invalid from square");
            return false;
        }
        // check pseudolegal moves
        if (to & pseudolegal_moves) == 0 {
            println!("Invalid target square");
            return false;
        }

        if is_capture {
            self.board.move_piece(from, to, is_white);
            self.board.remove_piece(to, !is_white);

            // additional capture rule
            if let Some(capture_check_fn) = is_capture_check {
                // capture_check_fn(to);
            }

            // TODO check for pin
            // TODO check for check
            self.turn += 1;
            true
        } else {
            // Normal move
            // TODO check for pin
            self.board.move_piece(from, to, is_white);
            self.turn += 1;
            true
        }
    }

    pub fn move_bishop(&mut self, from: u64, to: u64) -> bool {
        let is_white = self.is_white();
        let bishops = if is_white {
            self.board.white_bishops
        } else {
            self.board.black_bishops
        };

        self.move_piece(from, to, bishops, is_white, compute_bishops_moves, None)
    }

    pub fn move_rook(&mut self, from: u64, to: u64) -> bool {
        let is_white = self.is_white();
        let rooks = if is_white {
            self.board.white_rooks
        } else {
            self.board.black_rooks
        };

        self.move_piece(from, to, rooks, is_white, compute_rooks_moves, None)
    }

    pub fn move_knight(&mut self, from: u64, to: u64) -> bool {
        let is_white = self.is_white();
        let knights = if is_white {
            self.board.white_knights
        } else {
            self.board.black_knights
        };

        self.move_piece(from, to, knights, is_white, compute_knights_moves, None)
    }

    pub fn move_pawn(&mut self, from: u64, to: u64) -> bool {
        let is_white = self.is_white();
        let pawns = if is_white {
            self.board.white_pawns
        } else {
            self.board.black_pawns
        };

        // TODO add additional capture rule for pawn (must be diagonal)
        // TODO promotion
        self.move_piece(from, to, pawns, is_white, compute_pawns_moves, None)
    }
}


impl Default for Game {
    fn default() -> Game {
        Self::new(Board::default())
    }
}


#[cfg(test)]
pub mod tests {


    // test for move_pieces
}