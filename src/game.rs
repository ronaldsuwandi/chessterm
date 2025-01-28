use crate::board::{
    is_file, is_rank, render_bitboard, Board, MASK_FILE_A, MASK_FILE_B, MASK_FILE_C, MASK_FILE_D,
    MASK_FILE_F, MASK_FILE_G, MASK_FILE_H, MASK_RANK_1, MASK_RANK_8,
};
use crate::moves::{
    compute_bishops_moves, compute_king_moves, compute_knights_moves, compute_pawns_moves,
    compute_queens_moves, compute_rooks_moves, resolve_bishop_source, resolve_king_source,
    resolve_knight_source, resolve_pawn_source, resolve_queen_source, resolve_rook_source,
    QUEEN_RAYS,
};
use crate::parser::{parse_move, ParsedMove, Piece, SpecialMove};

const MASK_CASTLING_PATH_KINGSIDE: u64 = (MASK_FILE_F | MASK_FILE_G) & (MASK_RANK_1 | MASK_RANK_8);
const MASK_CASTLING_PATH_QUEENSIDE: u64 =
    (MASK_FILE_B | MASK_FILE_C | MASK_FILE_D) & (MASK_RANK_1 | MASK_RANK_8);

const MASK_CASTLING_KINGSIDE_PIECE: u64 = MASK_FILE_H & (MASK_RANK_1 | MASK_RANK_8);
const MASK_CASTLING_QUEENSIDE_PIECE: u64 = MASK_FILE_A & (MASK_RANK_1 | MASK_RANK_8);

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
pub enum InvalidMoveReason {
    NoSourceOrTarget,
    InvalidSourceOrTarget,
    MultipleTargets,
    NotCaptureMove,
    KingCaptureMove,
    PawnNonDiagonalCapture,
    PawnInvalidPromotion,
    NoCastlingRight,
    CastlingPathBlocked,
    NoCastlingRook,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MoveError {
    AmbiguousSource,
    InvalidMove(InvalidMoveReason),
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

    fn get_pieces(board: &Board, piece_type: &Piece, is_white: bool) -> u64 {
        match piece_type {
            Piece::Pawn => {
                if is_white {
                    board.white_pawns
                } else {
                    board.black_pawns
                }
            }
            Piece::Knight => {
                if is_white {
                    board.white_knights
                } else {
                    board.black_knights
                }
            }
            Piece::Rook => {
                if is_white {
                    board.white_rooks
                } else {
                    board.black_rooks
                }
            }
            Piece::Bishop => {
                if is_white {
                    board.white_bishops
                } else {
                    board.black_bishops
                }
            }
            Piece::Queen => {
                if is_white {
                    board.white_queens
                } else {
                    board.black_queens
                }
            }
            Piece::King | Piece::Castling => {
                if is_white {
                    board.white_king
                } else {
                    board.black_king
                }
            }
        }
    }

    fn get_computed_pseudolegal_moves(&self, piece_type: &Piece, is_white: bool) -> u64 {
        match piece_type {
            Piece::Pawn => {
                if is_white {
                    self.board.white_pawns_pseudolegal_moves
                } else {
                    self.board.black_pawns_pseudolegal_moves
                }
            }
            Piece::Knight => {
                if is_white {
                    self.board.white_knights_pseudolegal_moves
                } else {
                    self.board.black_knights_pseudolegal_moves
                }
            }
            Piece::Rook => {
                if is_white {
                    self.board.white_rooks_pseudolegal_moves
                } else {
                    self.board.black_rooks_pseudolegal_moves
                }
            }
            Piece::Bishop => {
                if is_white {
                    self.board.white_bishops_pseudolegal_moves
                } else {
                    self.board.black_bishops_pseudolegal_moves
                }
            }
            Piece::Queen => {
                if is_white {
                    self.board.white_queens_pseudolegal_moves
                } else {
                    self.board.black_queens_pseudolegal_moves
                }
            }
            Piece::King | Piece::Castling => {
                if is_white {
                    self.board.white_king_pseudolegal_moves
                } else {
                    self.board.black_king_pseudolegal_moves
                }
            }
        }
    }

    pub fn process_move(&mut self, cmd: &str) -> Result<(), MoveError> {
        if let Ok(parsed_move) = parse_move(cmd) {
            let is_white = self.is_white();
            let pieces = Self::get_pieces(&self.board, &parsed_move.piece, is_white);
            let pseudolegal_moves =
                self.get_computed_pseudolegal_moves(&parsed_move.piece, is_white);

            let pinned_pieces = if is_white {
                self.pinned_white
            } else {
                self.pinned_black
            };

            match parsed_move.piece {
                Piece::Pawn => {
                    // special case for pawns
                    self.process_pawn(
                        parsed_move,
                        pieces,
                        is_white,
                        pseudolegal_moves,
                        pinned_pieces,
                        self.check,
                    )?
                }
                Piece::Knight => self.process_piece(
                    parsed_move,
                    pieces,
                    is_white,
                    resolve_knight_source,
                    pseudolegal_moves,
                    pinned_pieces,
                    self.check,
                )?,
                Piece::Bishop => self.process_piece(
                    parsed_move,
                    pieces,
                    is_white,
                    resolve_bishop_source,
                    pseudolegal_moves,
                    pinned_pieces,
                    self.check,
                )?,
                Piece::Queen => self.process_piece(
                    parsed_move,
                    pieces,
                    is_white,
                    resolve_queen_source,
                    pseudolegal_moves,
                    pinned_pieces,
                    self.check,
                )?,
                Piece::Rook => self.process_rook(
                    parsed_move,
                    pieces,
                    is_white,
                    pseudolegal_moves,
                    pinned_pieces,
                    self.check,
                )?,
                Piece::King => self.process_king(
                    parsed_move,
                    pieces,
                    is_white,
                    pseudolegal_moves,
                    pinned_pieces,
                    self.check,
                )?,
                Piece::Castling => {
                    self.process_castling(parsed_move, is_white, pinned_pieces, self.check)?
                }
            }
            // move successful, increment turn
            self.turn += 1;

            self.board.update_compute_moves();
            self.update_pinned_state();
            self.update_check_state();

            // TODO
            self.board.render();

            // final step is to update game status
            self.update_game_status();
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
        pseudolegal_moves: u64,
        pinned_pieces: u64,
        check: bool,
    ) -> Result<(), MoveError> {
        let to = mv.to;
        let from = resolve_pawn_source(&self.board, &mv, self.is_white());

        self.validate_pawn_move(from, to, &mv, self.is_white())?;
        Self::validate_move_piece(
            &self.board,
            from,
            to,
            pawns,
            is_white,
            mv.is_capture,
            pseudolegal_moves,
            pinned_pieces,
            check,
        )?;
        self.move_piece(from, to, is_white, mv.is_capture)?;

        if let Some(SpecialMove::Promotion(piece)) = mv.special_move {
            self.board.replace_pawn(to, is_white, piece);
        }
        Ok(())
    }

    fn process_king(
        &mut self,
        mv: ParsedMove,
        king: u64,
        is_white: bool,
        pseudolegal_moves: u64,
        pinned_pieces: u64,
        check: bool,
    ) -> Result<(), MoveError> {
        let to = mv.to;
        let from = resolve_king_source(&self.board, &mv, self.is_white());

        self.validate_king_move(to, self.is_white())?;
        Self::validate_move_piece(
            &self.board,
            from,
            to,
            king,
            is_white,
            mv.is_capture,
            pseudolegal_moves,
            pinned_pieces,
            check,
        )?;
        self.move_piece(from, to, is_white, mv.is_capture)?;

        // TODO extract this to another function
        // remove castling right
        if is_white {
            self.white_can_castle_kingside = false;
            self.white_can_castle_queenside = false;
        } else {
            self.black_can_castle_kingside = false;
            self.black_can_castle_queenside = false;
        }

        Ok(())
    }

    fn process_rook(
        &mut self,
        mv: ParsedMove,
        rook: u64,
        is_white: bool,
        pseudolegal_moves: u64,
        pinned_pieces: u64,
        check: bool,
    ) -> Result<(), MoveError> {
        let to = mv.to;
        let from = resolve_rook_source(&self.board, &mv, self.is_white());

        Self::validate_move_piece(
            &self.board,
            from,
            to,
            rook,
            is_white,
            mv.is_capture,
            pseudolegal_moves,
            pinned_pieces,
            check,
        )?;
        self.move_piece(from, to, is_white, mv.is_capture)?;

        // remove castling right
        if is_white {
            if is_file(from, 'a') {
                self.white_can_castle_queenside = false;
            } else if is_file(from, 'h') {
                self.white_can_castle_kingside = false;
            }
        } else {
            if is_file(from, 'a') {
                self.black_can_castle_queenside = false;
            } else if is_file(from, 'h') {
                self.black_can_castle_kingside = false;
            }
        }

        Ok(())
    }

    fn process_castling(
        &mut self,
        mv: ParsedMove,
        is_white: bool,
        pinned_pieces: u64,
        check: bool,
    ) -> Result<(), MoveError> {
        // TODO do this
        // 1. which side am I castling on, SpecialMove is optional enum that can be CastlingKing or CastlingQueen
        // 2. make sure I still have the right for castling
        // 3. make sure no pieces on the path to castling
        // 4. make sure no attack_moves on the path to castling
        // 5. once castling done, disable all castling rights for that color

        // TODO refactor the code
        if let Some(special_move) = mv.special_move {
            let king = Self::get_pieces(&self.board, &Piece::King, is_white);
            let rooks = Self::get_pieces(&self.board, &Piece::Rook, is_white);

            let rank = if is_white { MASK_RANK_1 } else { MASK_RANK_8 };

            if special_move == SpecialMove::CastlingKing {
                self.validate_castling(true, is_white)?;
                let rook = rooks & MASK_FILE_H;
                self.move_piece(king, rank & MASK_FILE_G, is_white, false)?;
                self.move_piece(rook, rank & MASK_FILE_F, is_white, false)?;
            } else if special_move == SpecialMove::CastlingQueen {
                self.validate_castling(false, is_white)?;
                let rook = rooks & MASK_FILE_A;
                self.move_piece(king, rank & MASK_FILE_C, is_white, false)?;
                self.move_piece(rook, rank & MASK_FILE_D, is_white, false)?;
            }
            return Ok(());
        }
        Err(MoveError::InvalidMove(
            InvalidMoveReason::InvalidSourceOrTarget,
        ))
    }

    fn process_piece<F>(
        &mut self,
        mv: ParsedMove,
        pieces: u64,
        is_white: bool,
        source_resolver_fn: F,
        pseudolegal_moves: u64,
        pinned_pieces: u64,
        check: bool,
    ) -> Result<(), MoveError>
    where
        F: Fn(&Board, &ParsedMove, bool) -> u64,
    {
        let to = mv.to;
        let from = source_resolver_fn(&self.board, &mv, self.is_white());

        Self::validate_move_piece(
            &self.board,
            from,
            to,
            pieces,
            is_white,
            mv.is_capture,
            pseudolegal_moves,
            pinned_pieces,
            check,
        )?;
        self.move_piece(from, to, is_white, mv.is_capture)
    }

    fn process_promotion(board: &mut Board, mv: ParsedMove, is_white: bool) {
        if let Some(SpecialMove::Promotion(piece)) = mv.special_move {
            board.replace_pawn(mv.to, is_white, piece);
        }
    }

    // pawn specific move validation (diagonal capture, promotion, etc)
    fn validate_pawn_move(
        &self,
        from: u64,
        to: u64,
        mv: &ParsedMove,
        is_white: bool,
    ) -> Result<(), MoveError> {
        if mv.is_capture {
            let diagonal_moves = if is_white {
                (from << 7) | (from << 9) // diagonal forward left/right
            } else {
                (from >> 7) | (from >> 9) // diagonal backward left/right
            };
            if to & diagonal_moves == 0 {
                return Err(MoveError::InvalidMove(
                    InvalidMoveReason::PawnNonDiagonalCapture,
                ));
            }

            // TODO en passant check
        }

        if let Some(SpecialMove::Promotion(_)) = mv.special_move {
            // promotion only allowed on rank 8 for white and rank 1 for black
            let correct_rank = if is_white {
                is_rank(to, 8)
            } else {
                is_rank(to, 1)
            };

            if !correct_rank {
                return Err(MoveError::InvalidMove(
                    InvalidMoveReason::PawnInvalidPromotion,
                ));
            }
        }
        Ok(())
    }

    // king specific move validation (can't enter into attack ray)
    fn validate_king_move(&self, to: u64, is_white: bool) -> Result<(), MoveError> {
        let opponent_attacks = Self::get_attack_moves(&self.board, is_white);

        if to & opponent_attacks != 0 {
            Err(MoveError::Checked)
        } else {
            Ok(())
        }
    }

    fn validate_castling(&self, is_kingside: bool, is_white: bool) -> Result<(), MoveError> {
        if self.check {
            return Err(MoveError::Checked);
        }

        // TODO refactor and tidy up
        if is_white {
            if is_kingside && !self.white_can_castle_kingside {
                return Err(MoveError::InvalidMove(InvalidMoveReason::NoCastlingRight));
            } else if !is_kingside && !self.white_can_castle_queenside {
                return Err(MoveError::InvalidMove(InvalidMoveReason::NoCastlingRight));
            } else if is_kingside
                && (self.board.white_rooks & MASK_CASTLING_KINGSIDE_PIECE & MASK_RANK_1 == 0)
            {
                return Err(MoveError::InvalidMove(InvalidMoveReason::NoCastlingRook));
            } else if !is_kingside
                && (self.board.white_rooks & MASK_CASTLING_QUEENSIDE_PIECE & MASK_RANK_1 == 0)
            {
                return Err(MoveError::InvalidMove(InvalidMoveReason::NoCastlingRook));
            }

            let castling_path = if is_kingside {
                MASK_CASTLING_PATH_KINGSIDE & MASK_RANK_1
            } else {
                MASK_CASTLING_PATH_QUEENSIDE & MASK_RANK_1
            };
            let castling_path_clear =
                castling_path & self.board.free & !self.board.black_attack_moves == castling_path;
            if !castling_path_clear {
                return Err(MoveError::InvalidMove(
                    InvalidMoveReason::CastlingPathBlocked,
                ));
            }
        } else {
            if is_kingside && !self.black_can_castle_kingside {
                return Err(MoveError::InvalidMove(InvalidMoveReason::NoCastlingRight));
            } else if !is_kingside && !self.black_can_castle_queenside {
                return Err(MoveError::InvalidMove(InvalidMoveReason::NoCastlingRight));
            } else if is_kingside
                && (self.board.black_rooks & MASK_CASTLING_KINGSIDE_PIECE & MASK_RANK_8 == 0)
            {
                return Err(MoveError::InvalidMove(InvalidMoveReason::NoCastlingRook));
            } else if !is_kingside
                && (self.board.black_rooks & MASK_CASTLING_QUEENSIDE_PIECE & MASK_RANK_8 == 0)
            {
                return Err(MoveError::InvalidMove(InvalidMoveReason::NoCastlingRook));
            }

            let castling_path = if is_kingside {
                MASK_CASTLING_PATH_KINGSIDE & MASK_RANK_8
            } else {
                MASK_CASTLING_PATH_QUEENSIDE & MASK_RANK_8
            };

            let castling_path_clear =
                (castling_path & self.board.free & !self.board.white_attack_moves) == castling_path;

            if !castling_path_clear {
                return Err(MoveError::InvalidMove(
                    InvalidMoveReason::CastlingPathBlocked,
                ));
            }
        }
        Ok(())
    }

    fn validate_move_pinned_piece(
        board: &Board,
        from: u64,
        to: u64,
        pinned_pieces: u64,
        is_white: bool,
    ) -> bool {
        let king = Self::get_pieces(board, &Piece::King, is_white);

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

    fn validate_move_check(board: &Board, from: u64, to: u64, is_white: bool) -> bool {
        let mut simulated_board = board.clone();
        let king;
        let opponent_king;
        let opponent_pieces;
        if is_white {
            king = simulated_board.white_king;
            opponent_king = simulated_board.black_king;
            opponent_pieces = simulated_board.black_pieces ^ opponent_king;
        } else {
            king = simulated_board.black_king;
            opponent_king = simulated_board.white_king;
            opponent_pieces = simulated_board.white_pieces ^ opponent_king;
        }

        // do not allow capturing king
        if to == opponent_king {
            return false;
        }

        simulated_board.move_piece(from, to, is_white);
        if opponent_pieces & to != 0 {
            // simulate capture
            simulated_board.remove_piece(to, !is_white);
        }

        // update the whole moves for simplicity, this helps with capture and
        // blocking move
        simulated_board.update_compute_moves();

        // if attack_moves & to
        Self::is_in_check(&simulated_board, is_white)
    }

    fn validate_move_piece(
        board: &Board,
        from: u64,
        to: u64,
        pieces: u64,
        is_white: bool,
        is_capture: bool,
        pseudolegal_moves: u64,
        pinned_pieces: u64,
        check: bool,
        // attack_moves: u64,
    ) -> Result<(), MoveError> {
        if from == to {
            return Err(MoveError::InvalidMove(
                InvalidMoveReason::InvalidSourceOrTarget,
            ));
        }

        if from == 0 || to == 0 {
            return Err(MoveError::InvalidMove(InvalidMoveReason::NoSourceOrTarget));
        }

        if to.count_ones() != 1 {
            return Err(MoveError::InvalidMove(InvalidMoveReason::MultipleTargets));
        }

        if from.count_ones() > 1 {
            return Err(MoveError::AmbiguousSource);
        }

        if (from & pieces) == 0 {
            return Err(MoveError::InvalidMove(
                InvalidMoveReason::InvalidSourceOrTarget,
            ));
        }

        if (to & pseudolegal_moves) == 0 {
            return Err(MoveError::InvalidMove(
                InvalidMoveReason::InvalidSourceOrTarget,
            ));
        }

        let target_must_be_captured = board.is_capture(to, is_white);
        if is_capture != target_must_be_captured {
            return Err(MoveError::InvalidMove(InvalidMoveReason::NotCaptureMove));
        }

        let opponent_king = if is_white {
            board.black_king
        } else {
            board.white_king
        };
        if is_capture && (to & opponent_king != 0) {
            return Err(MoveError::InvalidMove(InvalidMoveReason::KingCaptureMove));
        }

        if (from & pinned_pieces) != 0 {
            if !Self::validate_move_pinned_piece(board, from, to, pinned_pieces, is_white) {
                return Err(MoveError::Pinned);
            }
        }

        // test for check
        if check {
            // if Self::is_in_check(board, is_white) {
            if Self::validate_move_check(board, from, to, is_white) {
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
    ) -> Result<(), MoveError> {
        if is_capture {
            self.board.move_piece(from, to, is_white);
            self.board.remove_piece(to, !is_white);
        } else {
            // Normal move
            self.board.move_piece(from, to, is_white);
        }
        Ok(())
    }

    // pin handling
    fn update_pinned_state(&mut self) {
        self.pinned_white = self.detect_pins(true);
        self.pinned_black = self.detect_pins(false);
    }

    fn detect_pins(&self, is_white: bool) -> u64 {
        let king = Self::get_pieces(&self.board, &Piece::King, is_white);
        let king_idx = king.trailing_zeros();

        // own pieces exclude king
        let own_pieces = if is_white {
            self.board.white_pieces ^ king
        } else {
            self.board.black_pieces ^ king
        };

        let opponent_sliding_moves = self.get_computed_pseudolegal_moves(&Piece::Rook, !is_white)
            | self.get_computed_pseudolegal_moves(&Piece::Bishop, !is_white)
            | self.get_computed_pseudolegal_moves(&Piece::Queen, !is_white);

        let opponent_sliding_pieces = Self::get_pieces(&self.board, &Piece::Rook, !is_white)
            | Self::get_pieces(&self.board, &Piece::Bishop, !is_white)
            | Self::get_pieces(&self.board, &Piece::Queen, !is_white);

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

    fn get_attack_moves(board: &Board, is_white: bool) -> u64 {
        if is_white {
            board.black_attack_moves
        } else {
            board.white_attack_moves
        }
    }

    // check if king is in check
    fn is_in_check(board: &Board, is_white: bool) -> bool {
        let king = Self::get_pieces(board, &Piece::King, is_white);
        let opponent_attacks = Self::get_attack_moves(board, is_white);
        king & opponent_attacks != 0
    }

    /// perform is_in_check for simulated move. This is EXPENSIVE because
    /// it has to recompute all the opponent moves and this must be called
    /// because if simulated move completely alter attack moves, then
    /// cached version is no longer valid (ie. capturing attacking piece,
    /// blocking sliding piece, etc)
    /// delta attack move can be implemented but will be rather tricky...
    fn is_in_check_simulated_move(board: &Board, is_white: bool) -> bool {
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
        piece: &Piece,
        mut pseudolegal_moves: u64,
        is_white: bool,
        opponent_pieces: u64,
    ) -> bool {
        let pinned = if is_white {
            self.pinned_white
        } else {
            self.pinned_black
        };

        let mut pieces = Self::get_pieces(&self.board, piece, is_white);

        while pieces != 0 {
            let piece_idx = pieces.trailing_zeros() as u64;
            let piece_pos = 1 << piece_idx;

            while pseudolegal_moves != 0 {
                let move_idx = pseudolegal_moves.trailing_zeros() as u64;
                let single_move = 1 << move_idx;

                let is_capture = move_idx & opponent_pieces != 0;

                // remove processed move
                pseudolegal_moves &= pseudolegal_moves - 1;

                match piece {
                    Piece::Pawn => {
                        // if self.validate_pawn_move()
                    }
                    Piece::Knight => {}
                    Piece::Rook => {}
                    Piece::Bishop => {}
                    Piece::Queen => {}
                    Piece::King => {
                        if self.validate_king_move(single_move, is_white).is_err() {
                            // skip
                            continue;
                        }
                    }
                    _ => {}
                }

                if Self::validate_move_piece(
                    &self.board,
                    piece_pos,
                    single_move,
                    piece_pos,
                    is_white,
                    is_capture,
                    single_move,
                    pinned,
                    self.check,
                )
                .is_ok()
                {
                    return true;
                }
            }
            // remove the processed piece
            pieces &= pieces - 1;
        }
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

    fn update_game_status(&mut self) {
        // check for sufficient material
        if !Self::has_sufficient_materials(&self.board) {
            self.status = Status::Draw;
            return;
        }

        let is_white = self.is_white();

        let knights_moves = self.get_computed_pseudolegal_moves(&Piece::Knight, is_white);
        let rooks_moves = self.get_computed_pseudolegal_moves(&Piece::Rook, is_white);
        let bishops_moves = self.get_computed_pseudolegal_moves(&Piece::Bishop, is_white);
        let queens_moves = self.get_computed_pseudolegal_moves(&Piece::Queen, is_white);
        let pawns_moves = self.get_computed_pseudolegal_moves(&Piece::Pawn, is_white);
        let king_moves = self.get_computed_pseudolegal_moves(&Piece::King, is_white);

        render_bitboard(&king_moves, 'k');

        let opponent_pieces = if is_white {
            self.board.black_pieces
        } else {
            self.board.white_pieces
        };

        let found_legal_move =
            self.has_valid_move(&Piece::Knight, knights_moves, is_white, opponent_pieces)
                || self.has_valid_move(&Piece::Rook, rooks_moves, is_white, opponent_pieces)
                || self.has_valid_move(&Piece::Bishop, bishops_moves, is_white, opponent_pieces)
                || self.has_valid_move(&Piece::Queen, queens_moves, is_white, opponent_pieces)
                || self.has_valid_move(&Piece::Pawn, pawns_moves, is_white, opponent_pieces)
                || self.has_valid_move(&Piece::King, king_moves, is_white, opponent_pieces);

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
    use crate::board::{bitboard_single, Board, PositionBuilder};

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
        assert!(game
            .validate_pawn_move(
                bitboard_single('e', 2).unwrap(),
                bitboard_single('d', 3).unwrap(),
                &parse_move("exd3").unwrap(),
                true
            )
            .is_ok());

        assert!(!game
            .validate_pawn_move(
                bitboard_single('e', 2).unwrap(),
                bitboard_single('e', 3).unwrap(),
                &parse_move("exe3").unwrap(),
                true
            )
            .is_ok());

        // check for promotion
        assert!(game
            .validate_pawn_move(
                bitboard_single('e', 7).unwrap(),
                bitboard_single('e', 8).unwrap(),
                &parse_move("e8=Q").unwrap(),
                true
            )
            .is_ok());
        assert!(!game
            .validate_pawn_move(
                bitboard_single('e', 6).unwrap(),
                bitboard_single('e', 7).unwrap(),
                &parse_move("e7=Q").unwrap(),
                true
            )
            .is_ok());

        assert!(game
            .validate_pawn_move(
                bitboard_single('e', 2).unwrap(),
                bitboard_single('e', 1).unwrap(),
                &parse_move("e1=Q").unwrap(),
                false
            )
            .is_ok());
        assert!(!game
            .validate_pawn_move(
                bitboard_single('e', 3).unwrap(),
                bitboard_single('e', 2).unwrap(),
                &parse_move("e2=Q").unwrap(),
                false
            )
            .is_ok());
    }

    #[test]
    fn test_pawn_move() {
        let board = Board::from_fen("7k/p7/8/8/3p4/4P1p1/P3P1PP/K7");
        let mut game = Game::new(board);

        process_moves_error(
            &mut game,
            &[
                // e3 is blocked by white piece
                (
                    "e3",
                    MoveError::InvalidMove(InvalidMoveReason::InvalidSourceOrTarget),
                ),
                // g3 is blocked by black piece
                (
                    "g3",
                    MoveError::InvalidMove(InvalidMoveReason::NotCaptureMove),
                ),
                // can't skip g3 because there's a black piece
                (
                    "g4",
                    MoveError::InvalidMove(InvalidMoveReason::InvalidSourceOrTarget),
                ),
            ],
        );
        process_moves(&mut game, &["h3", "a5"]);
    }

    #[test]
    fn test_pawn_capture() {
        let board = Board::from_fen("7k/p7/8/8/3p4/4P1p1/P3P1PP/K7");
        let mut game = Game::new(board);
        // g3 can only be captured diagonally from h2
        process_moves_error(
            &mut game,
            &[
                (
                    "gxg3",
                    MoveError::InvalidMove(InvalidMoveReason::PawnNonDiagonalCapture),
                ),
                (
                    "fxg3",
                    MoveError::InvalidMove(InvalidMoveReason::PawnNonDiagonalCapture),
                ),
            ],
        );
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
        process_moves_error(
            &mut game,
            &[(
                "a3=R",
                MoveError::InvalidMove(InvalidMoveReason::PawnInvalidPromotion),
            )],
        );
        game.turn = 4; // switch to black
                       // promotion doesn't work if not rank 1 for black
        process_moves_error(
            &mut game,
            &[(
                "a6=R",
                MoveError::InvalidMove(InvalidMoveReason::PawnInvalidPromotion),
            )],
        );
    }

    #[test]
    fn test_knight() {
        let board = Board::from_fen("kn6/8/1n6/8/2P5/4pp2/4P3/K3N1N1");
        let mut game = Game::new(board);

        process_moves_error(
            &mut game,
            &[
                // blocked by own piece
                (
                    "Ne2",
                    MoveError::InvalidMove(InvalidMoveReason::InvalidSourceOrTarget),
                ),
                ("Nxf3", MoveError::AmbiguousSource),
                // must capture
                (
                    "Ngf3",
                    MoveError::InvalidMove(InvalidMoveReason::NotCaptureMove),
                ),
            ],
        );
        process_moves(
            &mut game,
            &[
                "Ngxf3", "Nxc4", // black capture c4,
                // additional detailed selectors
                "N3e5", "Nb8a6", "Ne5xc4",
            ],
        );
    }

    #[test]
    fn test_basic_moves() {
        let mut game = Game::default();
        process_moves(&mut game, &["e4", "e5"]);
        process_moves_error(
            &mut game,
            &[
                (
                    "e5",
                    MoveError::InvalidMove(InvalidMoveReason::InvalidSourceOrTarget),
                ), // blocked by opponent
            ],
        );
        process_moves(&mut game, &["Bb5", "Nf6"]);

        process_moves_error(
            &mut game,
            &[
                ("Rb1", MoveError::AmbiguousSource), // ambiguous
                (
                    "Rab1",
                    MoveError::InvalidMove(InvalidMoveReason::InvalidSourceOrTarget),
                ), // blocked by own piece and ambiguous
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
    fn test_valid_move_remove_check() {
        let board = Board::from_fen("8/4q1k1/8/5P2/8/8/8/3K4");
        let mut game = Game::new(board);
        // neither in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        process_moves(&mut game, &["f6"]);
        // black in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(Game::is_in_check(&game.board, false));
        process_moves(&mut game, &["Qxf6"]);
        // neither in check after attacking piece is captured
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
    }

    #[test]
    fn test_detect_pins() {
        let board = Board::from_fen("8/8/8/4q3/7b/k7/1r1PPPP1/r1B1K3");
        let game = Game::new(board);

        assert_eq!(
            PositionBuilder::new()
                .add_piece('c', 1)
                .add_piece('e', 2)
                .add_piece('f', 2)
                .build(),
            game.detect_pins(true)
        );

        assert_eq!(bitboard_single('b', 2).unwrap(), game.detect_pins(false));
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
        process_moves_error(
            &mut game,
            &[(
                "dxe8",
                MoveError::InvalidMove(InvalidMoveReason::KingCaptureMove),
            )],
        );
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
        process_moves_error(
            &mut game,
            &[
                ("Ra6", MoveError::Checked),
                ("Re1", MoveError::Checked),
                // can't move king to position that's still being checked
                ("Kd7", MoveError::Checked),
            ],
        );

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
    fn test_draw_no_legal_move_king_blocking() {
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

    #[test]
    fn test_draw_no_legal_move() {
        let board = Board::from_fen("7k/8/8/6Q1/8/8/8/K7");
        let mut game = Game::new(board);

        // neither in check
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Ongoing, game.status);
        process_moves(&mut game, &["Qg6"]);

        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Draw, game.status);
    }

    #[test]
    fn test_draw_no_legal_move_pawn() {
        let board = Board::from_fen("7k/8/6KP/8/8/8/8/8");
        let mut game = Game::new(board);
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Ongoing, game.status);
        process_moves(&mut game, &["h7"]);

        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Draw, game.status);
    }

    #[test]
    fn test_draw_no_legal_move_pinned() {
        let board = Board::from_fen("7k/5P1n/8/6N1/8/1R6/8/K6R");
        let mut game = Game::new(board);
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Ongoing, game.status);
        process_moves(&mut game, &["Ne6"]);

        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        assert_eq!(Status::Draw, game.status);
    }

    #[test]
    fn test_validate_castling() {
        let basic_castlings = [
            (true, true, true, true, Ok(())),
            (true, true, false, true, Ok(())),
            (true, true, true, false, Ok(())),
            (true, true, false, false, Ok(())),
            (
                false,
                true,
                true,
                true,
                Err(MoveError::InvalidMove(InvalidMoveReason::NoCastlingRight)),
            ),
            (
                true,
                false,
                false,
                true,
                Err(MoveError::InvalidMove(InvalidMoveReason::NoCastlingRight)),
            ),
        ];
        for (can_castle_kingside, can_castle_queenside, is_kingside, is_white, result) in
            basic_castlings
        {
            let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R");
            let mut game = Game::new(board);
            if is_white {
                game.white_can_castle_kingside = can_castle_kingside;
                game.white_can_castle_queenside = can_castle_queenside;
            } else {
                game.black_can_castle_kingside = can_castle_kingside;
                game.black_can_castle_queenside = can_castle_queenside;
            }
            assert_eq!(result, game.validate_castling(is_kingside, is_white));
        }
    }

    #[test]
    fn test_castling() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R");
        let mut game = Game::new(board);
        process_moves(&mut game, &["O-O"]);
        assert_eq!(bitboard_single('g', 1).unwrap(), game.board.white_king);
        assert_eq!(
            PositionBuilder::new()
                .add_piece('a', 1)
                .add_piece('f', 1)
                .build(),
            game.board.white_rooks
        );
        process_moves_error(
            &mut game,
            &[(
                "O-O",
                MoveError::InvalidMove(InvalidMoveReason::CastlingPathBlocked),
            )],
        );
        process_moves(&mut game, &["O-O-O"]);
        assert_eq!(bitboard_single('c', 8).unwrap(), game.board.black_king);
        assert_eq!(
            PositionBuilder::new()
                .add_piece('d', 8)
                .add_piece('h', 8)
                .build(),
            game.board.black_rooks
        );

        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R2QK2R");
        let mut game = Game::new(board);
        process_moves(&mut game, &["Qd8"]);
        // black should be in check
        assert!(Game::is_in_check(&game.board, false));
        process_moves_error(
            &mut game,
            &[("O-O-O", MoveError::Checked), ("O-O", MoveError::Checked)],
        );
        process_moves(&mut game, &["Kxd8", "Kf1", "Ke8"]);
        process_moves_error(
            &mut game,
            &[
                (
                    "O-O-O",
                    MoveError::InvalidMove(InvalidMoveReason::NoCastlingRight),
                ),
                (
                    "O-O",
                    MoveError::InvalidMove(InvalidMoveReason::NoCastlingRight),
                ),
            ],
        );

        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R");
        let mut game = Game::new(board);
        process_moves(&mut game, &["Rab1", "Ra7", "Rba1"]);
        // no check but rook on queen side has moved
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        process_moves_error(
            &mut game,
            &[(
                "O-O-O",
                MoveError::InvalidMove(InvalidMoveReason::NoCastlingRight),
            )],
        );
        process_moves(&mut game, &["O-O"]);

        let board = Board::from_fen("r3k2r/6B1/8/8/8/8/8/R3K2R");
        let mut game = Game::new(board);
        process_moves(&mut game, &["Bxh8"]);
        // no check but rook on queen side has moved
        assert!(!Game::is_in_check(&game.board, true));
        assert!(!Game::is_in_check(&game.board, false));
        process_moves_error(
            &mut game,
            &[(
                "O-O",
                MoveError::InvalidMove(InvalidMoveReason::NoCastlingRook),
            )],
        );
    }
}
