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

    pub fn process_move(&mut self, cmd: &str) -> bool{

        true
    }


    // TODO implement parse move and game logic for check and pin
    // fn parse_move(&self, cmd: &str)
}

impl Default for Game {
    fn default() -> Game {
        Self::new(Board::default())
    }
}


#[cfg(test)]
pub mod tests {


    // test for move_pieces

    use crate::board::{bitboard_single, Board, PositionBuilder};
    use crate::parser::{parse_move, ParseError, ParsedMove, Piece};

    // #[test]
    // fn test_process_move()
    // #[test]
    // fn test_parse_pawn_capture() {
    //     let white_pawns: u64 = PositionBuilder::new()
    //         .add_piece('e', 2)
    //         .add_piece('e', 3)
    //         .add_piece('a', 2)
    //         .add_piece('g', 2) // blocked
    //         .add_piece('h', 2)
    //         .build();
    //     let black_pawns: u64 = PositionBuilder::new()
    //         .add_piece('a', 7)
    //         .add_piece('d', 4)
    //         .add_piece('g', 3)
    //         .build();
    //     let board = Board::new(white_pawns, 0, 0, 0, 0, 0, black_pawns, 0, 0, 0, 0, 0);
    //
    //     assert_eq!(
    //         ParsedMove {
    //             piece: Piece::Pawn,
    //             from: bitboard_single('e', 3).unwrap(),
    //             to: bitboard_single('d', 4).unwrap(),
    //             is_capture: true,
    //             is_white: true,
    //             special_move: None,
    //         },
    //         parse_move(&board, "exd4", true).unwrap()
    //     );
    //
    //     assert_eq!(
    //         Err(ParseError::InvalidTarget),
    //         parse_move(&board, "exd", true)
    //     );
    //
    //     assert_eq!(
    //         ParsedMove {
    //             piece: Piece::Pawn,
    //             from: bitboard_single('g', 3).unwrap(),
    //             to: bitboard_single('h', 2).unwrap(),
    //             is_capture: true,
    //             is_white: false,
    //             special_move: None,
    //         },
    //         parse_move(&board, "gxh2", false).unwrap()
    //     );
    // }
    // #[test]
    // fn test_parse_pawn_promotion() {
    //     let white_pawns: u64 = PositionBuilder::new()
    //         .add_piece('e', 2)
    //         .add_piece('e', 3)
    //         .add_piece('a', 2)
    //         .add_piece('g', 2) // blocked
    //         .add_piece('h', 7)
    //         .build();
    //     let white_knights: u64 = PositionBuilder::new()
    //         .add_piece('b', 1)
    //         .add_piece('g', 1)
    //         .build();
    //     let black_pawns: u64 = PositionBuilder::new()
    //         .add_piece('a', 7)
    //         .add_piece('d', 2)
    //         .add_piece('g', 3)
    //         .build();
    //     let black_knights: u64 = PositionBuilder::new()
    //         .add_piece('b', 8)
    //         .add_piece('g', 8)
    //         .build();
    //     let board = Board::new(
    //         white_pawns,
    //         white_knights,
    //         0,
    //         0,
    //         0,
    //         0,
    //         black_pawns,
    //         black_knights,
    //         0,
    //         0,
    //         0,
    //         0,
    //     );
    //
    //     assert_eq!(
    //         ParsedMove {
    //             piece: Piece::Pawn,
    //             from: bitboard_single('h', 7).unwrap(),
    //             to: bitboard_single('g', 8).unwrap(),
    //             is_capture: true,
    //             is_white: true,
    //             special_move: Some(SpecialMove::Promotion(Piece::Rook)),
    //         },
    //         parse_move(&board, "hxg8=R", true).unwrap()
    //     );
    //
    //     assert_eq!(
    //         ParsedMove {
    //             piece: Piece::Pawn,
    //             from: bitboard_single('d', 2).unwrap(),
    //             to: bitboard_single('d', 1).unwrap(),
    //             is_capture: false,
    //             is_white: false,
    //             special_move: Some(SpecialMove::Promotion(Piece::Queen)),
    //         },
    //         parse_move(&board, "d1=Q", false).unwrap()
    //     );
    //
    //     assert_eq!(
    //         ParsedMove {
    //             piece: Piece::Pawn,
    //             from: bitboard_single('d', 2).unwrap(),
    //             to: bitboard_single('d', 1).unwrap(),
    //             is_capture: false,
    //             is_white: false,
    //             special_move: Some(SpecialMove::Promotion(Piece::Knight)),
    //         },
    //         parse_move(&board, "d1=N", false).unwrap()
    //     );
    //
    //     assert_eq!(
    //         ParsedMove {
    //             piece: Piece::Pawn,
    //             from: bitboard_single('d', 2).unwrap(),
    //             to: bitboard_single('d', 1).unwrap(),
    //             is_capture: false,
    //             is_white: false,
    //             special_move: Some(SpecialMove::Promotion(Piece::Bishop)),
    //         },
    //         parse_move(&board, "d1=B", false).unwrap()
    //     );
    //
    //     // can't promote if not at the end
    //     assert_eq!(
    //         Err(ParseError::InvalidTarget),
    //         parse_move(&board, "a3=Q", true)
    //     );
    //
    //     assert_eq!(
    //         Err(ParseError::InvalidTarget),
    //         parse_move(&board, "h8=", true)
    //     );
    //
    //     assert_eq!(
    //         Err(ParseError::InvalidTarget),
    //         parse_move(&board, "h8=O", true)
    //     );
    // }
    //
    // #[test]
    // fn test_parse_castling() {
    //     let white_rooks: u64 = PositionBuilder::new()
    //         .add_piece('a', 1)
    //         .add_piece('h', 1)
    //         .build();
    //     let white_king: u64 = PositionBuilder::new().add_piece('e', 1).build();
    //     let black_rooks: u64 = PositionBuilder::new()
    //         .add_piece('a', 8)
    //         .add_piece('h', 8)
    //         .build();
    //     let black_king: u64 = PositionBuilder::new().add_piece('e', 8).build();
    //     let board = Board::new(
    //         0,
    //         0,
    //         white_rooks,
    //         0,
    //         0,
    //         white_king,
    //         0,
    //         0,
    //         black_rooks,
    //         0,
    //         0,
    //         black_king,
    //     );
    //
    //     assert_eq!(
    //         ParsedMove {
    //             piece: Piece::Castling,
    //             from: 0,
    //             to: 0,
    //             is_capture: false,
    //             is_white: true,
    //             special_move: Some(SpecialMove::CastlingKing),
    //         },
    //         parse_move(&board, "O-O", true).unwrap()
    //     );
    //     assert_eq!(
    //         ParsedMove {
    //             piece: Piece::Castling,
    //             from: 0,
    //             to: 0,
    //             is_capture: false,
    //             is_white: false,
    //             special_move: Some(SpecialMove::CastlingKing),
    //         },
    //         parse_move(&board, "O-O", false).unwrap()
    //     );
    //     assert_eq!(
    //         ParsedMove {
    //             piece: Piece::Castling,
    //             from: 0,
    //             to: 0,
    //             is_capture: false,
    //             is_white: true,
    //             special_move: Some(SpecialMove::CastlingQueen),
    //         },
    //         parse_move(&board, "O-O-O", true).unwrap()
    //     );
    //     assert_eq!(
    //         ParsedMove {
    //             piece: Piece::Castling,
    //             from: 0,
    //             to: 0,
    //             is_capture: false,
    //             is_white: false,
    //             special_move: Some(SpecialMove::CastlingQueen),
    //         },
    //         parse_move(&board, "O-O-O", false).unwrap()
    //     );
    //     assert_eq!(
    //         Err(ParseError::InvalidCastling),
    //         parse_move(&board, "O-", true)
    //     );
    //
    //     let white_rooks: u64 = PositionBuilder::new()
    //         .add_piece('a', 1)
    //         .add_piece('h', 1)
    //         .build();
    //     let white_king: u64 = PositionBuilder::new().add_piece('g', 2).build();
    //     let board = Board::new(
    //         0,
    //         0,
    //         white_rooks,
    //         0,
    //         0,
    //         white_king,
    //         0,
    //         0,
    //         0,
    //         0,
    //         0,
    //         0,
    //     );
    //
    //     assert_eq!(
    //         Err(ParseError::InvalidCastling),
    //         parse_move(&board, "O-O", true)
    //     );
    // }
}