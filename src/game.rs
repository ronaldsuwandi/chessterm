use crate::board::{is_file, is_rank, render_bitboard, Board};
use crate::moves::{
    compute_bishops_moves, compute_king_moves, compute_knights_moves, compute_pawns_moves,
    compute_queens_moves, compute_rooks_moves, resolve_bishop_source, resolve_king_source,
    resolve_knight_source, resolve_pawn_source, resolve_queen_source, resolve_rook_source,
    BISHOP_RAYS, QUEEN_RAYS, ROOK_RAYS,
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
    pub pinned_white: u64,
    pub pinned_black: u64,
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

            pinned_white: 0,
            pinned_black: 0,
        }
    }

    fn is_white(&self) -> bool {
        self.turn % 2 == 1
    }

    pub fn process_move(&mut self, cmd: &str) -> Result<(), MoveError> {
        if let Ok(parsed_move) = parse_move(cmd) {
            match parsed_move.piece {
                Piece::Pawn => {
                    // special case for pawns
                    self.process_pawn(parsed_move)?
                }
                Piece::Knight => {
                    self.process_piece(parsed_move, resolve_knight_source, Self::move_knight)?
                }
                Piece::Bishop => {
                    self.process_piece(parsed_move, resolve_bishop_source, Self::move_bishop)?
                }
                Piece::Queen => {
                    self.process_piece(parsed_move, resolve_queen_source, Self::move_queen)?
                }
                Piece::Rook => {
                    self.process_piece(parsed_move, resolve_rook_source, Self::move_rook)?
                }
                Piece::King => {
                    self.process_piece(parsed_move, resolve_king_source, Self::move_king)?
                }
                Piece::Castling => {
                    // self.process_piece(parsed_move, resolve_queen_source, Self::move_queen)
                    // Ok(())
                }
            }
            // move successful, increment turn
            self.turn += 1;

            // update pins for the opponent
            self.update_pinned_state();

            Ok(())
        } else {
            Err(MoveError::ParseError)
        }
    }

    fn process_pawn(&mut self, mv: ParsedMove) -> Result<(), MoveError> {
        let to = mv.to;
        let from = resolve_pawn_source(&self.board, &mv, self.is_white());

        if !self.validate_pawn_move(from, to, &mv, self.is_white()) {
            return Err(MoveError::InvalidMove);
        }
        self.move_pawn(from, to, mv)
    }

    fn process_piece<F, G>(
        &mut self,
        mv: ParsedMove,
        source_resolver_fn: F,
        move_fn: G,
    ) -> Result<(), MoveError>
    where
        F: Fn(&Board, &ParsedMove, bool) -> u64,
        G: Fn(&mut Self, u64, u64, ParsedMove) -> Result<(), MoveError>,
    {
        let to = mv.to;
        let from = source_resolver_fn(&self.board, &mv, self.is_white());

        move_fn(self, from, to, mv)
    }

    // pawn specific move validation (diagonal capture, promotion, etc)
    fn validate_pawn_move(&self, from: u64, to: u64, mv: &ParsedMove, is_white: bool) -> bool {
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
    ) -> Result<(), MoveError>
    where
        F: Fn(&Board, bool) -> u64,
    {
        let pseudolegal_moves = compute_moves(&self.board, is_white);
        if from == to {
            println!("Invalid from and to square");
            return Err(MoveError::InvalidMove);
        }

        if from == 0 || to == 0 {
            println!("No source or target square");
            return Err(MoveError::InvalidMove);
        }

        if to.count_ones() != 1 {
            println!("Target must only have 1 position");
            return Err(MoveError::InvalidMove);
        }

        if from.count_ones() > 1 {
            println!("Ambiguous source");
            return Err(MoveError::AmbiguousSource);
        }

        if (from & pieces) == 0 {
            println!("Invalid from square");
            return Err(MoveError::InvalidMove);
        }

        if (to & pseudolegal_moves) == 0 {
            println!("Invalid target square");
            return Err(MoveError::InvalidMove);
        }

        let pinned = if is_white {
            self.pinned_white
        } else {
            self.pinned_black
        };
        if (from & pinned) != 0 {
            return Err(MoveError::Pinned);
        }

        let target_must_be_captured = self.board.is_capture(to, is_white);
        if is_capture != target_must_be_captured {
            println!("Target must be captured, not moved");
            return Err(MoveError::InvalidMove);
        }
        Ok(())
    }

    fn move_piece<F>(
        &mut self,
        from: u64,
        to: u64,
        pieces: u64,
        is_white: bool,
        is_capture: bool,
        compute_moves: F,
    ) -> Result<(), MoveError>
    where
        F: Fn(&Board, bool) -> u64,
    {
        self.validate_move_piece(from, to, pieces, is_white, is_capture, &compute_moves)?;

        if is_capture {
            self.board.move_piece(from, to, is_white);
            self.board.remove_piece(to, !is_white);
        } else {
            // Normal move
            self.board.move_piece(from, to, is_white);
        }
        Ok(())
    }

    pub fn move_bishop(
        &mut self,
        from: u64,
        to: u64,
        parsed_move: ParsedMove,
    ) -> Result<(), MoveError> {
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
            parsed_move.is_capture,
            compute_bishops_moves,
        )
    }

    pub fn move_king(
        &mut self,
        from: u64,
        to: u64,
        parsed_move: ParsedMove,
    ) -> Result<(), MoveError> {
        let is_white = self.is_white();
        let king = if is_white {
            self.board.white_king
        } else {
            self.board.black_king
        };

        self.move_piece(
            from,
            to,
            king,
            is_white,
            parsed_move.is_capture,
            compute_king_moves,
        )?;

        // TODO castling state update
        Ok(())
    }

    pub fn move_rook(
        &mut self,
        from: u64,
        to: u64,
        parsed_move: ParsedMove,
    ) -> Result<(), MoveError> {
        let is_white = self.is_white();
        let rooks = if is_white {
            self.board.white_rooks
        } else {
            self.board.black_rooks
        };

        self.move_piece(
            from,
            to,
            rooks,
            is_white,
            parsed_move.is_capture,
            compute_rooks_moves,
        )?;

        // TODO castling state update
        // let

        if is_white && (self.white_can_castle_kingside || self.white_can_castle_queenside) {
            // disable castling if needed
            if is_file(from, 'a') {
            } else if is_file(from, 'h') {
            }
        } else if !is_white && (self.black_can_castle_kingside || self.black_can_castle_queenside) {
        }

        Ok(())
    }

    pub fn move_queen(
        &mut self,
        from: u64,
        to: u64,
        parsed_move: ParsedMove,
    ) -> Result<(), MoveError> {
        let is_white = self.is_white();
        let queens = if is_white {
            self.board.white_queens
        } else {
            self.board.black_queens
        };

        self.move_piece(
            from,
            to,
            queens,
            is_white,
            parsed_move.is_capture,
            compute_queens_moves,
        )
    }

    pub fn move_knight(
        &mut self,
        from: u64,
        to: u64,
        parsed_move: ParsedMove,
    ) -> Result<(), MoveError> {
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
            parsed_move.is_capture,
            compute_knights_moves,
        )
    }

    pub fn move_pawn(
        &mut self,
        from: u64,
        to: u64,
        parsed_move: ParsedMove,
    ) -> Result<(), MoveError> {
        let is_white = self.is_white();
        let pawns = if is_white {
            self.board.white_pawns
        } else {
            self.board.black_pawns
        };

        self.move_piece(
            from,
            to,
            pawns,
            is_white,
            parsed_move.is_capture,
            compute_pawns_moves,
        )?;

        if let Some(SpecialMove::Promotion(piece)) = parsed_move.special_move {
            self.board.replace_pawn(to, is_white, piece);
        }
        Ok(())
    }

    // TODO implement parse move and game logic for check
    // fn parse_move(&self, cmd: &str)

    // pin handling
    fn update_pinned_state(&mut self) {
        self.pinned_white = Self::detect_pins(&self.board, true);
        self.pinned_black = Self::detect_pins(&self.board, false);
    }

    fn detect_pins(board: &Board, is_white: bool) -> u64 {
        let king = if is_white {
            board.white_king
        } else {
            board.black_king
        };
        let king_idx = king.trailing_zeros();

        // own pieces exclude king
        let own_pieces = if is_white {
            board.white_pieces ^ king
        } else {
            board.black_pieces ^ king
        };

        let opponent_sliding_moves = compute_rooks_moves(board, !is_white)
            | compute_queens_moves(board, !is_white)
            | compute_bishops_moves(board, !is_white);

        let mut pinned_pieces: u64 = 0;
        // pin only happened through sliding pieces, check all sliding directions
        for direction in 0..8 {
            let ray = QUEEN_RAYS[king_idx as usize][direction];
            let blockers = ray & own_pieces;

            // pin only happen if there's 1 own piece blocking a ray
            if blockers.count_ones() == 1 {
                let blocker = blockers.trailing_zeros();
                let blocker_bit = 1u64 << blocker;

                if opponent_sliding_moves & blocker_bit != 0 {
                    pinned_pieces |= blocker_bit;
                }
            }
        }

        pinned_pieces
    }
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
        let board = Board::new(
            white_pawns,
            0,
            0,
            0,
            0,
            bitboard_single('a', 1).unwrap(),
            black_pawns,
            0,
            0,
            0,
            0,
            bitboard_single('h', 8).unwrap(),
        );
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
    fn test_pawn_capture() {
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
        let board = Board::new(
            white_pawns,
            0,
            0,
            0,
            0,
            bitboard_single('a', 1).unwrap(),
            black_pawns,
            0,
            0,
            0,
            0,
            bitboard_single('h', 8).unwrap(),
        );

        let mut game = Game::new(board);
        // g3 can only be captured diagonally from h2
        assert!(game.process_move("gxg3").is_err());
        assert!(game.process_move("fxg3").is_err());
        assert!(game.process_move("exd4").is_ok());
    }

    #[test]
    fn test_pawn_promotion() {
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
            bitboard_single('e', 1).unwrap(),
            black_pawns,
            black_knights,
            0,
            0,
            0,
            bitboard_single('e', 8).unwrap(),
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

    #[test]
    fn test_knight() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('e', 2)
            .add_piece('c', 4)
            .build();
        let white_knights: u64 = PositionBuilder::new()
            .add_piece('e', 1)
            .add_piece('g', 1)
            .build();
        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('e', 3)
            .add_piece('f', 3)
            .build();
        let black_knights: u64 = PositionBuilder::new()
            .add_piece('b', 8)
            .add_piece('b', 6)
            .build();
        let board = Board::new(
            white_pawns,
            white_knights,
            0,
            0,
            0,
            bitboard_single('a', 1).unwrap(),
            black_pawns,
            black_knights,
            0,
            0,
            0,
            bitboard_single('a', 8).unwrap(),
        );

        let mut game = Game::new(board);

        // blocked by own piece
        assert!(game.process_move("Ne2").is_err());
        // ambiguous
        assert!(game.process_move("Nxf3").is_err());
        // must capture
        assert!(game.process_move("Ngf3").is_err());
        assert!(game.process_move("Ngxf3").is_ok());

        // black then capture c4
        assert!(game.process_move("Nxc4").is_ok());

        // additional more detailed selector works
        assert!(game.process_move("N3e5").is_ok());
        assert!(game.process_move("Nb8a6").is_ok());
        assert!(game.process_move("Ne5xc4").is_ok());
    }

    #[test]
    fn test_basic_moves() {
        let mut game = Game::default();
        assert!(game.process_move("e4").is_ok());
        assert!(game.process_move("e5").is_ok());
        assert!(game.process_move("e5").is_err()); // blocked by opponent
        assert!(game.process_move("Bb5").is_ok());
        assert!(game.process_move("Nf6").is_ok());
        assert!(game.process_move("Rb1").is_err()); // blocked by own piece
        assert!(game.process_move("Qe2").is_ok());
        assert!(game.process_move("Nxe4").is_ok());
        assert!(game.process_move("d4").is_ok());
        assert!(game.process_move("exd4").is_ok());
        assert!(game.process_move("Qe3").is_ok());
        assert!(game.process_move("dxe3").is_ok());
        assert!(game.process_move("Nf3").is_ok());
        assert!(game.process_move("exf2").is_ok());
        assert!(game.process_move("Kd1").is_ok());
        assert!(game.process_move("f1=Q").is_ok());
    }

    #[test]
    fn test_detect_pins() {
        let white_pawns = PositionBuilder::new()
            .add_piece('d', 2)
            .add_piece('e', 2)
            .add_piece('f', 2)
            .add_piece('g', 2)
            .build();

        let white_bishops = PositionBuilder::new().add_piece('c', 1).build();

        let black_queens = PositionBuilder::new().add_piece('e', 5).build();

        let black_rooks = PositionBuilder::new()
            .add_piece('a', 1)
            .add_piece('b', 2)
            .build();

        let black_bishops = PositionBuilder::new().add_piece('h', 4).build();

        let board = Board::new(
            white_pawns,
            0,
            0,
            white_bishops,
            0,
            bitboard_single('e', 1).unwrap(),
            0,
            0,
            black_rooks,
            black_bishops,
            black_queens,
            bitboard_single('a', 3).unwrap(),
        );

        assert_eq!(
            PositionBuilder::new()
                .add_piece('c', 1)
                .add_piece('d', 2)
                .add_piece('e', 2)
                .add_piece('f', 2)
                .build(),
            Game::detect_pins(&board, true)
        );

        assert_eq!(
            bitboard_single('b', 2).unwrap(),
            Game::detect_pins(&board, false)
        );
    }

    #[test]
    fn test_pinned() {
        let board = Board::new(
            0,
            0,
            bitboard_single('a', 2).unwrap(),
            0,
            0,
            bitboard_single('e', 1).unwrap(),
            0,
            bitboard_single('e', 6).unwrap(),
            0,
            0,
            0,
            bitboard_single('e', 8).unwrap(),
        );

        let mut game = Game::new(board);

        // no pin at start
        assert_eq!(0, game.pinned_black);
        assert_eq!(0, game.pinned_white);
        assert!(game.process_move("Re2").is_ok());
        // black knight should be pinned
        assert_eq!(bitboard_single('e', 6).unwrap(), game.pinned_black);
        assert_eq!(0, game.pinned_white);
        // pinned knight should not be able to move
        assert_eq!(Err(MoveError::Pinned), game.process_move("Ng5"));
        assert!(game.process_move("Kd8").is_ok());
        // black no longer pinned
        assert_eq!(0, game.pinned_black);
        assert_eq!(0, game.pinned_white);
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
