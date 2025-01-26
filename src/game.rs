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
    pub status: Status,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MoveError {
    AmbiguousSource,
    InvalidMove,
    Pinned,
    Checked,
    ParseError,
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Ongoing,
    Draw,
    Checkmate,
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

            status: Status::Ongoing,
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
                    self.process_piece(
                        parsed_move,
                        pieces,
                        is_white,
                        resolve_knight_source,
                        compute_knights_moves,
                    )?
                }
                Piece::Bishop => {
                    pieces = if is_white {
                        self.board.white_bishops
                    } else {
                        self.board.black_bishops
                    };
                    self.process_piece(
                        parsed_move,
                        pieces,
                        is_white,
                        resolve_bishop_source,
                        compute_bishops_moves,
                    )?
                }
                Piece::Queen => {
                    pieces = if is_white {
                        self.board.white_queens
                    } else {
                        self.board.black_queens
                    };
                    self.process_piece(
                        parsed_move,
                        pieces,
                        is_white,
                        resolve_queen_source,
                        compute_queens_moves,
                    )?
                }
                Piece::Rook => {
                    pieces = if is_white {
                        self.board.white_rooks
                    } else {
                        self.board.black_rooks
                    };
                    self.process_piece(
                        parsed_move,
                        pieces,
                        is_white,
                        resolve_rook_source,
                        compute_rooks_moves,
                    )?
                }
                Piece::King => {
                    pieces = if is_white {
                        self.board.white_king
                    } else {
                        self.board.black_king
                    };
                    self.process_piece(
                        parsed_move,
                        pieces,
                        is_white,
                        resolve_king_source,
                        compute_king_moves,
                    )?
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

            self.evaluate_game_status();
            Ok(())
        } else {
            Err(MoveError::ParseError)
        }
    }

    fn process_pawn(
        &mut self,
        mv: ParsedMove,
        pawns: u64,
        is_white: bool,
    ) -> Result<(), MoveError> {
        let to = mv.to;
        let from = resolve_pawn_source(&self.board, &mv, self.is_white());

        if !self.validate_pawn_move(from, to, &mv, self.is_white()) {
            return Err(MoveError::InvalidMove);
        }

        self.validate_move_piece(
            from,
            to,
            pawns,
            is_white,
            mv.is_capture,
            &compute_pawns_moves,
        )?;
        self.move_piece(from, to, is_white, mv.is_capture)?;

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

    fn validate_move_check(&self, from: u64, to: u64, is_white: bool) -> bool {
        let mut simulated_board = self.board.clone();
        simulated_board.move_piece(from, to, is_white);
        Self::is_in_check(&simulated_board, is_white)
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

        let opponent_king = if is_white {
            self.board.black_king
        } else {
            self.board.white_king
        };
        if is_capture && (to & opponent_king != 0) {
            println!("King cannot be captured");
            return Err(MoveError::InvalidMove);
        }

        // test for check
        if Self::is_in_check(&self.board, is_white) {
            if self.validate_move_check(from, to, is_white) {
                println!("King is still checked");
                return Err(MoveError::Checked);
            }
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
where {
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

        self.move_piece(from, to, is_white, parsed_move.is_capture)
    }

    fn move_king(&mut self, from: u64, to: u64, parsed_move: ParsedMove) -> Result<(), MoveError> {
        let is_white = self.is_white();
        let king = if is_white {
            self.board.white_king
        } else {
            self.board.black_king
        };

        self.move_piece(from, to, is_white, parsed_move.is_capture)?;

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

        self.move_piece(from, to, is_white, parsed_move.is_capture)?;

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

        self.move_piece(from, to, is_white, parsed_move.is_capture)
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

        self.move_piece(from, to, is_white, parsed_move.is_capture)
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

    fn has_valid_move(
        &self,
        mut pieces: u64,
        mut pseudolegal_moves: u64,
        is_white: bool,
        opponent_pieces: u64,
        opponent_king: u64,
    ) -> bool {
        while pieces != 0 {
            let piece_idx = pieces.trailing_zeros() as u64;
            let piece_pos = 1 << piece_idx;

            while pseudolegal_moves != 0 {
                let move_idx = pseudolegal_moves.trailing_zeros() as u64;
                let single_move = 1 << move_idx;

                let is_capture = move_idx & opponent_pieces != 0;

                if self
                    .validate_move_piece(
                        piece_pos,
                        single_move,
                        piece_pos,
                        is_white,
                        is_capture,
                        |_: &Board, _: bool| -> u64 { single_move },
                    )
                    .is_ok()
                {
                    return true;
                }

                // remove processed move
                pseudolegal_moves &= pseudolegal_moves - 1;
            }
            // remove the processed piece
            pieces &= pieces - 1;
        }

        // TODO implement
        false
    }

    fn has_sufficient_materials(board: &Board) -> bool {
        // if pawn/rook/queen still around return true
        if board.white_pawns > 0
            || board.black_pawns > 0
            || board.white_queens > 0
            || board.black_queens > 0
            || board.white_rooks > 0
            || board.black_rooks > 0
        {
            return true;
        }

        let white_knights = board.white_knights.count_ones();
        let black_knights = board.black_knights.count_ones();
        let white_bishops = board.white_bishops.count_ones();
        let black_bishops = board.black_bishops.count_ones();

        let insufficient = matches!(
            (white_knights, black_knights, white_bishops, black_bishops),
            (0, 0, 0, 0)
                | (1, 0, 0, 0)
                | (0, 1, 0, 0)
                | (0, 0, 1, 0)
                | (0, 0, 0, 1)
                | (1, 1, 0, 0)
                | (0, 0, 1, 1)
                | (1, 0, 0, 1)
                | (0, 1, 1, 0)
                | (0, 2, 0, 0)
                | (2, 0, 0, 0)
        );

        !insufficient
    }

    fn evaluate_game_status(&mut self) {
        // check for sufficient material
        if !Self::has_sufficient_materials(&self.board) {
            self.status = Status::Draw;
            return;
        }

        let is_white = self.is_white();

        let knights_moves = compute_knights_moves(&self.board, is_white);
        let rooks_moves = compute_rooks_moves(&self.board, is_white);
        let bishops_moves = compute_bishops_moves(&self.board, is_white);
        let queens_moves = compute_queens_moves(&self.board, is_white);
        let pawns_moves = compute_pawns_moves(&self.board, is_white);
        let king_moves = compute_knights_moves(&self.board, is_white);

        let knights = if is_white {
            self.board.white_knights
        } else {
            self.board.black_knights
        };
        let rooks = if is_white {
            self.board.white_rooks
        } else {
            self.board.black_rooks
        };
        let bishops = if is_white {
            self.board.white_bishops
        } else {
            self.board.black_bishops
        };
        let queens = if is_white {
            self.board.white_queens
        } else {
            self.board.black_queens
        };
        let pawns = if is_white {
            self.board.white_pawns
        } else {
            self.board.black_pawns
        };
        let king = if is_white {
            self.board.white_king
        } else {
            self.board.black_king
        };

        let opponent_king = if is_white {
            self.board.black_king
        } else {
            self.board.white_king
        };
        let opponent_pieces = if is_white {
            self.board.black_pieces
        } else {
            self.board.white_pieces
        };

        let found_legal_move =
            self.has_valid_move(
                knights,
                knights_moves,
                is_white,
                opponent_pieces,
                opponent_king,
            ) || self.has_valid_move(rooks, rooks_moves, is_white, opponent_pieces, opponent_king)
                || self.has_valid_move(
                    bishops,
                    bishops_moves,
                    is_white,
                    opponent_pieces,
                    opponent_king,
                )
                || self.has_valid_move(
                    queens,
                    queens_moves,
                    is_white,
                    opponent_pieces,
                    opponent_king,
                )
                || self.has_valid_move(
                    pawns,
                    pawns_moves,
                    is_white,
                    opponent_pieces,
                    opponent_king,
                )
                || self.has_valid_move(king, king_moves, is_white, opponent_pieces, opponent_king);

        if found_legal_move {
            self.status = Status::Ongoing
        } else {
            if self.check {
                // check for checkmate
                self.status = Status::Checkmate;
            } else {
                // check for stalemate
                self.status = Status::Draw;
            }
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
    use super::*;
    use crate::board::{bit_pos, bitboard_single, render_bitboard, Board, PositionBuilder};

    fn process_moves(game: &mut Game, moves: &[&str]) {
        for &mv in moves {
            assert!(game.process_move(mv).is_ok());
        }
    }

    fn process_moves_error(game: &mut Game, moves: &[(&str, MoveError)]) {
        for &(mv, move_error) in moves {
            assert_eq!(Err(move_error), game.process_move(mv));
        }
    }

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
        let board = Board::from_fen("7k/p7/8/8/3p4/4P1p1/P3P1PP/K7");
        let mut game = Game::new(board);

        process_moves_error(
            &mut game,
            &[
                // e3 is blocked by white piece
                ("e3", MoveError::InvalidMove),
                // g3 is blocked by black piece
                ("g3", MoveError::InvalidMove),
                // can't skip g3 because there's a black piece
                ("g4", MoveError::InvalidMove),
            ],
        );
        process_moves(&mut game, &["h3", "a5"]);
    }

    #[test]
    fn test_pawn_capture() {
        let board = Board::from_fen("7k/p7/8/8/3p4/4P1p1/P3P1PP/K7");
        let mut game = Game::new(board);
        // g3 can only be captured diagonally from h2
        process_moves_error(&mut game, &[
            ("gxg3", MoveError::InvalidMove),
            ("fxg3", MoveError::InvalidMove),
        ]);
        process_moves(&mut game, &["exd4"]);
    }

    #[test]
    fn test_pawn_promotion() {
        let board = Board::from_fen("1n4n1/p3k2P/8/8/8/3pP1p1/P1p1P1P1/1N2K1N1");
        let mut game = Game::new(board);
        assert_eq!(0, game.board.white_queens); // no queen before
        process_moves(&mut game, &["hxg8=Q"]);
        assert_eq!(bitboard_single('g', 8).unwrap(), game.board.white_queens);

        // one black knight captured
        assert_eq!(bitboard_single('b', 8).unwrap(), game.board.black_knights);
        process_moves(&mut game, &["c1=N"]);
        assert_eq!(
            PositionBuilder::new()
                .add_piece('b', 8)
                .add_piece('c', 1)
                .build(),
            game.board.black_knights
        );

        // promotion doesn't work if not rank 8 for white
        process_moves_error(&mut game, &[("a3=R", MoveError::InvalidMove)]);
        game.turn = 4; // switch to black
        // promotion doesn't work if not rank 1 for black
        process_moves_error(&mut game, &[("a6=R", MoveError::InvalidMove)]);
    }

    #[test]
    fn test_knight() {
        let board = Board::from_fen("kn6/8/1n6/8/2P5/4pp2/4P3/K3N1N1");
        let mut game = Game::new(board);

        process_moves_error(&mut game, &[
            // blocked by own piece
            ("Ne2", MoveError::InvalidMove),
            ("Nxf3", MoveError::AmbiguousSource),
            // must capture
            ("Ngf3", MoveError::InvalidMove),
        ]);
        process_moves(&mut game, &[
            "Ngxf3",
            "Nxc4", // black capture c4,
            // additional detailed selectors
            "N3e5",
            "Nb8a6",
            "Ne5xc4",
        ]);
    }

    #[test]
    fn test_basic_moves() {
        let mut game = Game::default();
        process_moves(&mut game, &["e4", "e5"]);
        process_moves_error(
            &mut game,
            &[
                ("e5", MoveError::InvalidMove), // blocked by opponent
            ],
        );
        process_moves(&mut game, &["Bb5", "Nf6"]);

        process_moves_error(
            &mut game,
            &[
                ("Rb1", MoveError::AmbiguousSource), // ambiguous
                ("Rab1", MoveError::InvalidMove),    // blocked by own piece and ambiguous
            ],
        );
        process_moves(
            &mut game,
            &[
                "Qe2", "Nxe4", "d4", "exd4", "Qe3", "dxe3", "Nf3", "exf2", "Kd1", "f1=Q",
            ],
        )
    }

    #[test]
    fn test_detect_pins() {
        let board = Board::from_fen("8/8/8/4q3/7b/k7/1r1PPPP1/r1B1K3");

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
        let board = Board::from_fen("4k3/8/4n3/8/8/8/R7/4K3");

        let mut game = Game::new(board);

        // no pin at start
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        process_moves(&mut game, &["Re2"]);
        // black knight should be pinned
        assert_eq!(bitboard_single('e', 6).unwrap(), game.pinned_black);
        assert_eq!(0, game.pinned_white);
        // pinned knight should not be able to move
        process_moves_error(&mut game, &[("Ng5", MoveError::Pinned)]);
        process_moves(&mut game, &["Kd8"]);
        // black no longer pinned
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
    }

    #[test]
    fn test_pinned_sliding_both() {
        let board = Board::from_fen("3k4/8/8/1q6/8/8/3B4/4K3");
        let mut game = Game::new(board);

        // no pin at start
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        process_moves(&mut game, &["Bc3", "Qa5"]);
        // only white bishop should be pinned
        assert_eq!(bitboard_single('c', 3).unwrap(), game.pinned_white);
        assert_eq!(0, game.pinned_black);

        let board = Board::from_fen("5k2/8/3q4/8/1R6/B6Q/8/4K3");
        let mut game = Game::new(board);

        // no pin at start
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        process_moves(&mut game, &["Qg3"]);
        // nothing should be pinned, only white queen can attack black queen but it's not a pin
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        process_moves(&mut game, &["Qc5"]);
        // still no pin
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        process_moves(&mut game, &["Ra4"]);

        // black queen is now pinned
        assert_eq!(0, game.pinned_white);
        assert_eq!(bitboard_single('c', 5).unwrap(), game.pinned_black);
    }

    #[test]
    fn test_pinned_advance() {
        let board = Board::from_fen("4k3/8/3q4/8/5r2/8/4P3/4K3");
        let mut game = Game::new(board);

        // no pin at start
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        process_moves(&mut game, &["e3", "Qe6"]);
        // white pawn is pinned
        assert_eq!(bitboard_single('e', 3).unwrap(), game.pinned_white);
        assert_eq!(0, game.pinned_black);

        // pinned pawn can't capture rook at f4
        process_moves_error(&mut game, &[("exf4", MoveError::Pinned)]);
        // but pinned pawn can advance
        process_moves(&mut game, &["e4"]);
        // pawn now advanced to e4 and still pinned
        assert_eq!(bitboard_single('e', 4).unwrap(), game.pinned_white);
        assert_eq!(0, game.pinned_black);
    }

    #[test]
    fn test_pinned_advance_capture() {
        let board = Board::from_fen("4k3/3p4/8/2q5/8/8/3B4/4K3");
        let mut game = Game::new(board);

        // no pin at start
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
        process_moves(&mut game, &["Bc3", "Qa5"]);

        // // white bishop is pinned
        assert_eq!(bitboard_single('c', 3).unwrap(), game.pinned_white);
        assert_eq!(0, game.pinned_black);

        // pinned bishop captures queen
        process_moves(&mut game, &["Bxa5"]);

        // no more pin
        assert_eq!(0, game.pinned_white);
        assert_eq!(0, game.pinned_black);
    }

    #[test]
    fn test_capture_king() {
        // king cant be captured, this is already in checked state
        // this state is not achievable by the goal but the purpose for
        // this test is purely to ensure that when validating move, capturing
        // king will returns invalid move
        let board = Board::from_fen("4k3/3P4/8/8/8/8/8/4K3");

        let mut game = Game::new(board);
        process_moves_error(&mut game, &[("dxe8", MoveError::InvalidMove)]);
    }

    #[test]
    fn test_check_state() {
        let board = Board::from_fen("4k3/8/4r3/4b3/8/8/3B4/4K3");
        let mut game = Game::new(board);
        game.turn = 2; // black's turn
        assert!(!Game::is_in_check(&game.board, game.is_white()));
        // discovered check
        process_moves(&mut game, &["Bg3"]);
        // white is checked
        assert!(Game::is_in_check(&game.board, game.is_white()));
        // white move
        process_moves(&mut game, &["Kd1"]);
        // black is not checked
        assert!(!Game::is_in_check(&game.board, game.is_white()));
        // black move
        process_moves(&mut game, &["Kd7"]);
        // white is not checked
        assert!(!Game::is_in_check(&game.board, game.is_white()));
    }

    #[test]
    fn test_check_move_restriction() {
        let board = Board::from_fen("3k4/8/3Nr3/8/8/8/3R4/K7");
        let mut game = Game::new(board);
        // neither in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));

        // discovered check
        process_moves(&mut game, &["Nb7"]);

        // black is checked
        assert!(!Game::is_in_check(&game.board, true));
        assert!(Game::is_in_check(&game.board, false));

        // black can't move rook, king is still checked
        process_moves_error(&mut game, &[
            ("Ra6", MoveError::Checked),
            ("Re1", MoveError::Checked),
            // can't move king to position that's still being checked
            ("Kd7", MoveError::Checked),
        ]);

        // black is checked
        assert!(!Game::is_in_check(&game.board, true));
        assert!(Game::is_in_check(&game.board, false));

        // black move king to uncheck
        process_moves(&mut game, &["Ke8"]);
        // neither in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));

        // move white king
        process_moves(&mut game, &["Kb1", "Rb6"]);
        // white is now checked
        assert!(Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));

        // move white rook to block check
        process_moves(&mut game, &["Rb2"]);
        // neither in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
    }

    #[test]
    fn test_checkmate() {
        let board = Board::from_fen("3k4/R6R/6r1/8/8/8/8/K7");

        let mut game = Game::new(board);

        // neither in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        process_moves(&mut game, &["Rh8"]);

        // black in check but not mate
        assert!(!Game::is_in_check(&game.board, true));
        assert!(Game::is_in_check(&game.board, false));
        assert_eq!(Status::Ongoing, game.status);
        process_moves(&mut game, &["Rg8"]);
        // // blocked the check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        process_moves(&mut game, &["Rxg8"]);
        // black is lost
        assert!(!Game::is_in_check(&game.board, true));
        assert!(Game::is_in_check(&game.board, false));
        assert_eq!(Status::Checkmate, game.status);
    }

    #[test]
    fn test_draw_insufficient_materials() {
        // 2 kings
        let board = Board::from_fen("3k4/8/8/8/8/8/1r6/K7");
        let mut game = Game::new(board);

        // neither in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Ongoing, game.status);
        process_moves(&mut game, &["Kxb2"]);

        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Draw, game.status);

        // knight and bishop
        let board = Board::from_fen("3k4/7b/8/8/8/2r5/8/K2N4");
        let mut game = Game::new(board);

        // neither in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Ongoing, game.status);
        process_moves(&mut game, &["Nxc3"]);

        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Draw, game.status);

        // 2 knights
        let board = Board::from_fen("3k4/8/8/8/8/8/7q/K2N1N2");
        let mut game = Game::new(board);

        // neither in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Ongoing, game.status);
        process_moves(&mut game, &["Nxh2"]);

        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Draw, game.status);
    }

    #[test]
    fn test_draw_no_legal_move() {
        let board = Board::from_fen("7k/8/7K/7Q/8/8/8/8");
        let mut game = Game::new(board);

        // neither in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Ongoing, game.status);
        process_moves(&mut game, &["Qg5"]);

        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Draw, game.status);
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
