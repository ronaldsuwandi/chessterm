use crate::board::{is_file, is_rank, render_bitboard, Board};
use crate::moves::{
    compute_bishops_moves, compute_king_moves, compute_knights_moves, compute_pawns_moves,
    compute_queens_moves, compute_rooks_moves, resolve_bishop_source, resolve_king_source,
    resolve_knight_source, resolve_pawn_source, resolve_queen_source, resolve_rook_source,
    QUEEN_RAYS,
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
    pub check: bool,

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

            check: false,
            pinned_white: 0,
            pinned_black: 0,
        }
    }

    fn is_white(&self) -> bool {
        self.turn & 1 == 1
    }

    pub fn process_move(&mut self, cmd: &str) -> Result<(), MoveError> {
        if let Ok(parsed_move) = parse_move(cmd) {
            let is_white = self.is_white();
            let pieces;
            match parsed_move.piece {
                Piece::Pawn => {
                    pieces = if is_white {
                        self.board.white_pawns
                    } else {
                        self.board.black_pawns
                    };
                    // special case for pawns
                    self.process_pawn(parsed_move, pieces, is_white)?
                }
                Piece::Knight => {
                    pieces = if is_white {
                        self.board.white_knights
                    } else {
                        self.board.black_knights
                    };
                    self.process_piece(parsed_move, pieces, is_white, resolve_knight_source, compute_knights_moves)?
                }
                Piece::Bishop => {
                    pieces = if is_white {
                        self.board.white_bishops
                    } else {
                        self.board.black_bishops
                    };
                    self.process_piece(parsed_move, pieces, is_white, resolve_bishop_source, compute_bishops_moves)?
                }
                Piece::Queen => {
                    pieces = if is_white {
                        self.board.white_queens
                    } else {
                        self.board.black_queens
                    };
                    self.process_piece(parsed_move, pieces, is_white, resolve_queen_source, compute_queens_moves)?
                }
                Piece::Rook => {
                    pieces = if is_white {
                        self.board.white_rooks
                    } else {
                        self.board.black_rooks
                    };
                    self.process_piece(parsed_move, pieces, is_white, resolve_rook_source, compute_rooks_moves)?
                }
                Piece::King => {
                    pieces = if is_white {
                        self.board.white_king
                    } else {
                        self.board.black_king
                    };
                    self.process_piece(parsed_move, pieces, is_white, resolve_king_source, compute_king_moves)?
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
            self.update_check_state();

            Ok(())
        } else {
            Err(MoveError::ParseError)
        }
    }

    fn process_pawn(&mut self, mv: ParsedMove, pawns: u64, is_white: bool) -> Result<(), MoveError> {
        let to = mv.to;
        let from = resolve_pawn_source(&self.board, &mv, self.is_white());

        if !self.validate_pawn_move(from, to, &mv, self.is_white()) {
            return Err(MoveError::InvalidMove);
        }

        self.validate_move_piece(from, to, pawns, is_white, mv.is_capture, &compute_pawns_moves)?;
        self.move_piece(
            from,
            to,
            is_white,
            mv.is_capture
        )?;

        if let Some(SpecialMove::Promotion(piece)) = mv.special_move {
            self.board.replace_pawn(to, is_white, piece);
        }
        Ok(())
    }

    fn process_piece<F, G>(
        &mut self,
        mv: ParsedMove,
        pieces: u64,
        is_white: bool,
        source_resolver_fn: F,
        compute_move_fn: G,
    ) -> Result<(), MoveError>
    where
        F: Fn(&Board, &ParsedMove, bool) -> u64,
        G: Fn(&Board, bool) -> u64,
    {
        let to = mv.to;
        let from = source_resolver_fn(&self.board, &mv, self.is_white());

        self.validate_move_piece(from, to, pieces, is_white, mv.is_capture, compute_move_fn)?;
        self.move_piece(from, to, is_white, mv.is_capture)
    }

    fn process_promotion(board: &mut Board, mv: ParsedMove, is_white: bool) {
       if let Some(SpecialMove::Promotion(piece)) = mv.special_move {
            board.replace_pawn(mv.to, is_white, piece);
       }
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

    fn validate_move_pinned_piece(
        &self,
        from: u64,
        to: u64,
        pinned_pieces: u64,
        is_white: bool,
    ) -> bool {
        let king = if is_white {
            self.board.white_king
        } else {
            self.board.black_king
        };

        if from & pinned_pieces == 0 {
            return true; // no pin, all good
        }

        let king_idx = king.trailing_zeros() as usize;
        for direction in 0..8 {
            let ray = QUEEN_RAYS[king_idx][direction];

            // check only if from and to within the same ray
            if ray & from != 0 && ray & to != 0 {
                return true;
            }
        }
        false
    }

    fn validate_move_check() -> bool {
        // TODO do this
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
            if !self.validate_move_pinned_piece(from, to, pinned, is_white) {
                return Err(MoveError::Pinned);
            }
        }

        let target_must_be_captured = self.board.is_capture(to, is_white);
        if is_capture != target_must_be_captured {
            println!("Target must be captured, not moved");
            return Err(MoveError::InvalidMove);
        }

        // test for check
        if Self::is_in_check(&self.board, is_white) {

        }

        Ok(())
    }

    fn move_piece(
        &mut self,
        from: u64,
        to: u64,
        is_white: bool,
        is_capture: bool,
    ) -> Result<(), MoveError>
    where
    {
        if is_capture {
            self.board.move_piece(from, to, is_white);
            self.board.remove_piece(to, !is_white);
        } else {
            // Normal move
            self.board.move_piece(from, to, is_white);
        }
        Ok(())
    }

    fn move_bishop(
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
            is_white,
            parsed_move.is_capture,
        )
    }

    fn move_king(&mut self, from: u64, to: u64, parsed_move: ParsedMove) -> Result<(), MoveError> {
        let is_white = self.is_white();
        let king = if is_white {
            self.board.white_king
        } else {
            self.board.black_king
        };

        self.move_piece(
            from,
            to,
            is_white,
            parsed_move.is_capture,
        )?;

        // TODO castling state update
        Ok(())
    }

    fn move_rook(&mut self, from: u64, to: u64, parsed_move: ParsedMove) -> Result<(), MoveError> {
        let is_white = self.is_white();
        let rooks = if is_white {
            self.board.white_rooks
        } else {
            self.board.black_rooks
        };

        self.move_piece(
            from,
            to,
            is_white,
            parsed_move.is_capture,
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

    fn move_queen(&mut self, from: u64, to: u64, parsed_move: ParsedMove) -> Result<(), MoveError> {
        let is_white = self.is_white();
        let queens = if is_white {
            self.board.white_queens
        } else {
            self.board.black_queens
        };

        self.move_piece(
            from,
            to,
            is_white,
            parsed_move.is_capture,
        )
    }

    fn move_knight(
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
            is_white,
            parsed_move.is_capture,
        )
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
            | compute_bishops_moves(board, !is_white)
            | compute_queens_moves(board, !is_white);

        let opponent_sliding_pieces = if is_white {
            board.black_rooks | board.black_bishops | board.black_queens
        } else {
            board.white_rooks | board.white_bishops | board.white_queens
        };

        let mut pinned_pieces: u64 = 0;
        // pin only happened through sliding pieces, check all sliding directions

        for direction_from_king in 0..8 {
            // opposite direction of the ray (add by 4 and modulo 8)
            let direction_to_king = (direction_from_king + 4) % 8;

            let ray = QUEEN_RAYS[king_idx as usize][direction_from_king];
            let blockers = ray & own_pieces;

            // pin only happens there is only 1 piece blocking a ray
            if blockers.count_ones() != 1 {
                continue;
            }

            let blocker = blockers.trailing_zeros();
            let blocker_bit = 1u64 << blocker;

            // found potential pin that can be attacked
            if opponent_sliding_moves & blocker_bit != 0 {
                // find the attacker piece
                let mut pinned = false;

                let mut pieces = opponent_sliding_pieces;
                while pieces != 0 {
                    let piece_idx = pieces.trailing_zeros() as usize;

                    let opponent_ray = QUEEN_RAYS[piece_idx][direction_to_king];

                    // ray targeting king will hit the king
                    if opponent_ray & king != 0 {
                        // check opponent moves only at the ray direction
                        let opponent_ray_to_blocker;
                        // includes blocker bit in the ray to the blocker
                        if direction_to_king == 0
                            || direction_to_king == 1
                            || direction_to_king == 2
                            || direction_to_king == 7
                        {
                            opponent_ray_to_blocker =
                                blocker_bit | opponent_ray & !(u64::MAX << blocker);
                        } else {
                            opponent_ray_to_blocker =
                                blocker_bit | opponent_ray & (u64::MAX << blocker + 1);
                        };

                        /*
                        conditions:
                        1. only 1 piece blocking ray of attack FROM king
                        2. blocking piece can be hit from sliding pieces from opponents
                        3. opponent RAY of attack can reach king
                        4. opponent legal move can hit blocking piece and share the same oppoenent RAY line
                         */

                        // pin only if the moves from the piece overlap to the opponent ray to the blocker
                        pinned = opponent_sliding_moves & opponent_ray_to_blocker
                            == opponent_ray_to_blocker;
                        if pinned {
                            break;
                        }
                    }

                    // Remove the processed piece (use lsb approach)
                    pieces &= pieces - 1;
                }

                if pinned {
                    pinned_pieces |= blocker_bit;
                }
            }
        }
        pinned_pieces
    }

    fn update_check_state(&mut self) {
        self.check = Self::is_in_check(&self.board, self.is_white());
    }

    // check if king is in check
    fn is_in_check(board: &Board, is_white: bool) -> bool {
        let king = if is_white {
            board.white_king
        } else {
            board.black_king
        };

        let opponent_attacks = compute_knights_moves(&board, !is_white)
            | compute_rooks_moves(&board, !is_white)
            | compute_bishops_moves(&board, !is_white)
            | compute_queens_moves(&board, !is_white)
            | compute_pawns_moves(&board, !is_white);

        king & opponent_attacks != 0
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
    use crate::board::{bit_pos, bitboard_single, render_bitboard, Board, PositionBuilder};

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
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        assert!(game.process_move("Re2").is_ok());
        // black knight should be pinned
        assert_eq!(bitboard_single('e', 6).unwrap(), game.pinned_black);
        assert_eq!(0, game.pinned_white);
        // pinned knight should not be able to move
        assert_eq!(Err(MoveError::Pinned), game.process_move("Ng5"));
        assert!(game.process_move("Kd8").is_ok());
        // black no longer pinned
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
    }

    #[test]
    fn test_pinned_sliding_both() {
        let board = Board::new(
            0,
            0,
            0,
            bitboard_single('d', 2).unwrap(),
            0,
            bitboard_single('e', 1).unwrap(),
            0,
            0,
            0,
            0,
            bitboard_single('b', 5).unwrap(),
            bitboard_single('d', 8).unwrap(),
        );

        let mut game = Game::new(board);

        // no pin at start
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        assert!(game.process_move("Bc3").is_ok());
        assert!(game.process_move("Qa5").is_ok());
        // only white bishop should be pinned
        assert_eq!(bitboard_single('c', 3).unwrap(), game.pinned_white);
        assert_eq!(0, game.pinned_black);

        let board = Board::new(
            0,
            0,
            bitboard_single('b', 4).unwrap(),
            bitboard_single('a', 3).unwrap(),
            bitboard_single('h', 3).unwrap(),
            bitboard_single('e', 1).unwrap(),
            0,
            0,
            0,
            0,
            bitboard_single('d', 6).unwrap(),
            bitboard_single('f', 8).unwrap(),
        );

        let mut game = Game::new(board);

        // no pin at start
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        assert!(game.process_move("Qg3").is_ok());
        // nothing should be pinned, only white queen can attack black queen but it's not a pin
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        assert!(game.process_move("Qc5").is_ok());
        // still no pin
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        assert!(game.process_move("Ra4").is_ok());

        // black queen is now pinned
        assert_eq!(0, game.pinned_white);
        assert_eq!(bitboard_single('c', 5).unwrap(), game.pinned_black);
    }

    #[test]
    fn test_pinned_advance() {
        let board = Board::new(
            bitboard_single('e', 2).unwrap(),
            0,
            0,
            0,
            0,
            bitboard_single('e', 1).unwrap(),
            0,
            0,
            bitboard_single('f', 4).unwrap(),
            0,
            bitboard_single('d', 6).unwrap(),
            bitboard_single('e', 8).unwrap(),
        );

        let mut game = Game::new(board);

        // no pin at start
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        assert!(game.process_move("e3").is_ok());
        assert!(game.process_move("Qe6").is_ok());
        // white pawn is pinned
        assert_eq!(bitboard_single('e', 3).unwrap(), game.pinned_white);
        assert_eq!(0, game.pinned_black);

        // pinned pawn can't capture rook at f4
        assert_eq!(Err(MoveError::Pinned), game.process_move("exf4"));
        // but pinned pawn can advance
        assert!(game.process_move("e4").is_ok());
        // pawn now advanced to e4 and still pinned
        assert_eq!(bitboard_single('e', 4).unwrap(), game.pinned_white);
        assert_eq!(0, game.pinned_black);
    }

    #[test]
    fn test_pinned_advance_capture() {
        let board = Board::new(
            0,
            0,
            0,
            bitboard_single('d', 2).unwrap(),
            0,
            bitboard_single('e', 1).unwrap(),
            bitboard_single('d', 7).unwrap(),
            0,
            0,
            0,
            bitboard_single('b', 5).unwrap(),
            bitboard_single('d', 8).unwrap(),
        );

        let mut game = Game::new(board);

        // no pin at start
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        assert!(game.process_move("Bc3").is_ok());
        assert!(game.process_move("Qa5").is_ok());

        // // white bishop is pinned
        assert_eq!(bitboard_single('c', 3).unwrap(), game.pinned_white);
        assert_eq!(0, game.pinned_black);

        // pinned bishop captures queen
        assert!(game.process_move("Bxa5").is_ok());

        // no more pin
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
    }

    #[test]
    fn test_check_state() {
        let board = Board::new(
            0,
            0,
            0,
            bitboard_single('d', 2).unwrap(),
            0,
            bitboard_single('e', 1).unwrap(),
            0,
            0,
            bitboard_single('e', 6).unwrap(),
            bitboard_single('e', 5).unwrap(),
            0,
            bitboard_single('d', 8).unwrap(),
        );

        let mut game = Game::new(board);
        game.turn = 2; // black's turn
        assert!(!Game::is_in_check(&game.board, game.is_white()));
        // discovered check
        assert!(game.process_move("Bg3").is_ok());
        // white is checked
        assert!(Game::is_in_check(&game.board, game.is_white()));
        // white move
        assert!(game.process_move("Kd1").is_ok());
        // black is not checked
        assert!(!Game::is_in_check(&game.board, game.is_white()));
        // black move
        assert!(game.process_move("Kd7").is_ok());
        // white is not checked
        assert!(!Game::is_in_check(&game.board, game.is_white()));
    }

    // #[test]
    // fn test_check_attackers() {
    //     let black_knights: u64 = PositionBuilder::new().add_piece('f', 6).build();
    //     let board = Board::new(
    //         0,
    //         0,
    //         0,
    //         0,
    //         0,
    //         bitboard_single('e', 4).unwrap(),
    //         0,
    //         black_knights,
    //         0,
    //         0,
    //         0,
    //         bitboard_single('d', 7).unwrap(),
    //     );
    //
    //     let mut game = Game::new(board);
    //     game.turn = 3; // white's turn
    //     assert_eq!(
    //         bitboard_single('f', 6).unwrap(),
    //         game.check_attackers());
    // }

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
