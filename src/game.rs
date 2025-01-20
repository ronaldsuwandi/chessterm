use crate::board::{is_rank, render_bitboard, Board};
use crate::moves::{
    compute_bishops_moves, compute_knights_moves, compute_pawns_moves, compute_rooks_moves,
    resolve_pawn_source,
};
use crate::parser::{parse_move, ParsedMove, Piece, SpecialMove};

/// Game struct responsible for all game logics (pin, check, valid captures, etc)
pub struct Game {
    pub board: Board,
    pub turn: u8,

    // castling
    pub white_can_castle_kingside: bool,
    pub white_can_castle_queenside: bool,
    pub black_can_castle_kingside: bool,
    pub black_can_castle_queenside: bool,
    // check

    // pin

    // en passant

    // end game (checkmate, draw)
}

#[derive(Debug, PartialEq)]
pub enum MoveError {
    AmbiguousSource,
    InvalidMove,
    Pinned,
    ParseError,
}

impl Game {
    pub fn new(board: Board) -> Game {
        Game {
            board,
            turn: 1,
            white_can_castle_kingside: true,
            white_can_castle_queenside: true,
            black_can_castle_kingside: true,
            black_can_castle_queenside: true,
        }
    }

    fn is_white(&self) -> bool {
        self.turn % 2 == 1
    }

    pub fn process_move(&mut self, cmd: &str) -> Result<bool, MoveError> {
        if let Ok(parsed_move) = parse_move(cmd) {
            match parsed_move.piece {
                Piece::Pawn => {
                    self.process_pawn(parsed_move)?;
                }
                Piece::Knight => {
                    self.process_knight(parsed_move)?;
                }
                Piece::Bishop | Piece::Queen => {

                }
                Piece::Rook => {}
                Piece::King => {}
                Piece::Castling => {}
            }

            Ok(true)
        } else {
            Err(MoveError::ParseError)
        }
    }

    fn process_pawn(&mut self, mv: ParsedMove) -> Result<bool, MoveError> {
        let to = mv.to;
        let from = resolve_pawn_source(&self.board, &mv, self.is_white());

        if !self.validate_pawn_move(from, to, &mv, self.is_white()) {
            return Err(MoveError::InvalidMove);
        }
        if self.move_pawn(from, to, mv) {
            Ok(true)
        } else {
            Err(MoveError::InvalidMove)
        }
    }

    fn process_knight(&mut self, mv: ParsedMove) -> Result<bool, MoveError> {
        let to = mv.to;
        let from = resolve_pawn_source(&self.board, &mv, self.is_white());

        if !self.validate_pawn_move(from, to, &mv, self.is_white()) {
            return Err(MoveError::InvalidMove);
        }
        if self.move_pawn(from, to, mv) {
            Ok(true)
        } else {
            Err(MoveError::InvalidMove)
        }
    }

    // pawn specific move validation (diagonal capture, promotion, etc)
    fn validate_pawn_move(&self, from: u64, to: u64, mv: &ParsedMove, is_white: bool) -> bool {
        if from == 0 || to == 0 {
            // no valid from/to
            return false;
        }

        if mv.is_capture {
            let diagonal_moves = if is_white {
                (from << 7) | (from << 9) // diagonal forward left/right
            } else {
                (from >> 7) | (from >> 9) // diagonal backward left/right
            };
            if to & diagonal_moves == 0 {
                return false; // not diagonal capture, invalid move
            }
        }

        if let Some(SpecialMove::Promotion(_)) = mv.special_move {
            // promotion only allowed on rank 8 for white and rank 1 for black
            let correct_rank = if is_white {
                is_rank(to, 8)
            } else {
                is_rank(to, 1)
            };

            if !correct_rank {
                return false;
            }
        }

        // TODO check for enpassant?
        true
    }

    fn validate_move_piece<F>(
        &self,
        from: u64,
        to: u64,
        pieces: u64,
        is_white: bool,
        is_capture: bool,
        compute_moves: F,
    ) -> bool
    where
        F: Fn(&Board, bool) -> u64,
    {
        let pseudolegal_moves = compute_moves(&self.board, is_white);

        if from == to {
            println!("Invalid from and to square");
            return false;
        }

        if (from & pieces) == 0 {
            println!("Invalid from square");
            return false;
        }

        if (to & pseudolegal_moves) == 0 {
            println!("Invalid target square");
            return false;
        }

        let target_must_be_captured = self.board.is_capture(to, is_white);
        if is_capture != target_must_be_captured {
            println!("Target must be captured, not moved");
            return false;
        }

        let target_must_be_captured = self.board.is_capture(to, is_white);

        if is_capture != target_must_be_captured {
            return false;
        }

        // TODO check for pin

        true
    }

    fn move_piece<F>(
        &mut self,
        from: u64,
        to: u64,
        pieces: u64,
        is_white: bool,
        is_capture: bool,
        compute_moves: F,
    ) -> bool
    where
        F: Fn(&Board, bool) -> u64,
    {
        if !self.validate_move_piece(from, to, pieces, is_white, is_capture, &compute_moves) {
            return false;
        }

        if is_capture {
            self.board.move_piece(from, to, is_white);
            self.board.remove_piece(to, !is_white);
        } else {
            // Normal move
            // TODO check for pin
            self.board.move_piece(from, to, is_white);
        }
        self.turn += 1;
        true
    }

    pub fn move_bishop(&mut self, from: u64, to: u64, is_capture: bool) -> bool {
        let is_white = self.is_white();
        let bishops = if is_white {
            self.board.white_bishops
        } else {
            self.board.black_bishops
        };

        self.move_piece(
            from,
            to,
            bishops,
            is_white,
            is_capture,
            compute_bishops_moves,
        )
    }

    pub fn move_rook(&mut self, from: u64, to: u64, is_capture: bool) -> bool {
        let is_white = self.is_white();
        let rooks = if is_white {
            self.board.white_rooks
        } else {
            self.board.black_rooks
        };

        self.move_piece(from, to, rooks, is_white, is_capture, compute_rooks_moves)
    }

    pub fn move_knight(&mut self, from: u64, to: u64, is_capture: bool) -> bool {
        let is_white = self.is_white();
        let knights = if is_white {
            self.board.white_knights
        } else {
            self.board.black_knights
        };

        self.move_piece(
            from,
            to,
            knights,
            is_white,
            is_capture,
            compute_knights_moves,
        )
    }

    pub fn move_pawn(&mut self, from: u64, to: u64, parsed_move: ParsedMove) -> bool {
        let is_white = self.is_white();
        let pawns = if is_white {
            self.board.white_pawns
        } else {
            self.board.black_pawns
        };

        let moved = self.move_piece(
            from,
            to,
            pawns,
            is_white,
            parsed_move.is_capture,
            compute_pawns_moves,
        );
        if let Some(SpecialMove::Promotion(piece)) = parsed_move.special_move {
            if moved {
                self.board.replace_pawn(to, is_white, piece);
            }
        }
        moved
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
    use super::*;
    use crate::board::{bitboard_single, Board, PositionBuilder};

    #[test]
    fn test_validate_pawn_move() {
        let game = Game::default();

        // check for diagonal capture
        assert!(game.validate_pawn_move(
            bitboard_single('e', 2).unwrap(),
            bitboard_single('d', 3).unwrap(),
            &parse_move("exd3").unwrap(),
            true
        ));

        assert!(!game.validate_pawn_move(
            bitboard_single('e', 2).unwrap(),
            bitboard_single('e', 3).unwrap(),
            &parse_move("exe3").unwrap(),
            true
        ));

        // check for promotion
        assert!(game.validate_pawn_move(
            bitboard_single('e', 7).unwrap(),
            bitboard_single('e', 8).unwrap(),
            &parse_move("e8=Q").unwrap(),
            true
        ));
        assert!(!game.validate_pawn_move(
            bitboard_single('e', 6).unwrap(),
            bitboard_single('e', 7).unwrap(),
            &parse_move("e7=Q").unwrap(),
            true
        ));

        assert!(game.validate_pawn_move(
            bitboard_single('e', 2).unwrap(),
            bitboard_single('e', 1).unwrap(),
            &parse_move("e1=Q").unwrap(),
            false
        ));
        assert!(!game.validate_pawn_move(
            bitboard_single('e', 3).unwrap(),
            bitboard_single('e', 2).unwrap(),
            &parse_move("e2=Q").unwrap(),
            false
        ));
    }

    #[test]
    fn test_pawn_move() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('e', 2)
            .add_piece('e', 3)
            .add_piece('a', 2)
            .add_piece('g', 2) // blocked
            .add_piece('h', 2)
            .build();
        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 7)
            .add_piece('d', 4)
            .add_piece('g', 3)
            .build();
        let board = Board::new(white_pawns, 0, 0, 0, 0, 0, black_pawns, 0, 0, 0, 0, 0);
        let mut game = Game::new(board);

        // e3 is blocked by white piece
        assert!(game.process_move("e3").is_err());
        // g3 is blocked by black piece
        assert!(game.process_move("g3").is_err());
        // can't skip g3 because there's a black piece
        assert!(game.process_move("g4").is_err());
        assert!(game.process_move("h3").is_ok());

        // black can move a5 next
        assert!(game.process_move("a5").is_ok());
    }

    #[test]
    fn test_parse_pawn_capture() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('e', 2)
            .add_piece('e', 3)
            .add_piece('a', 2)
            .add_piece('g', 2) // blocked
            .add_piece('h', 2)
            .build();
        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 7)
            .add_piece('d', 4)
            .add_piece('g', 3)
            .build();
        let board = Board::new(white_pawns, 0, 0, 0, 0, 0, black_pawns, 0, 0, 0, 0, 0);

        let mut game = Game::new(board);
        // g3 can only be captured diagonally from h2
        assert!(game.process_move("gxg3").is_err());
        assert!(game.process_move("fxg3").is_err());
        assert!(game.process_move("exd4").is_ok());
    }

    #[test]
    fn test_parse_pawn_promotion() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('e', 2)
            .add_piece('e', 3)
            .add_piece('a', 2)
            .add_piece('g', 2) // blocked
            .add_piece('h', 7)
            .build();
        let white_knights: u64 = PositionBuilder::new()
            .add_piece('b', 1)
            .add_piece('g', 1)
            .build();
        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 7)
            .add_piece('c', 2)
            .add_piece('d', 3)
            .add_piece('g', 3)
            .build();
        let black_knights: u64 = PositionBuilder::new()
            .add_piece('b', 8)
            .add_piece('g', 8)
            .build();
        let board = Board::new(
            white_pawns,
            white_knights,
            0,
            0,
            0,
            0,
            black_pawns,
            black_knights,
            0,
            0,
            0,
            0,
        );

        let mut game = Game::new(board);
        assert_eq!(0, game.board.white_queens); // no queen before
        assert!(game.process_move("hxg8=Q").is_ok());
        assert_eq!(bitboard_single('g', 8).unwrap(), game.board.white_queens);

        // one black knight captured
        assert_eq!(bitboard_single('b', 8).unwrap(), game.board.black_knights);
        assert!(game.process_move("c1=N").is_ok());
        assert_eq!(
            PositionBuilder::new()
                .add_piece('b', 8)
                .add_piece('c', 1)
                .build(),
            game.board.black_knights
        );

        // promotion doesn't work if not rank 8 for white
        assert!(game.process_move("a3=R").is_err());
        game.turn = 4; // switch to black
                       // promotion doesn't work if not rank 1 for black
        assert!(game.process_move("a6=R").is_err());
    }

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
