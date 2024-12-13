use crate::board::{bit_pos, bitboard_single, render_bitboard, Board};
use std::str::Chars;
use crate::moves::detect_pawn_source;

#[derive(Debug, PartialEq)]
pub enum Piece {
    Pawn,
    Knight,
    Rook,
    Bishop,
    Queen,
    King,
    Castling,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidLength,
    InvalidSource,
    InvalidTarget,
    InvalidCastling,
}

#[derive(Debug, PartialEq)]
pub enum SpecialMove {
    Promotion(Piece),
    CastlingKing,
    CastlingQueen,
}

#[derive(Debug, PartialEq)]
pub struct Move {
    pub piece: Piece,
    pub from: u64,
    pub to: u64,
    pub is_capture: bool,
    pub is_white: bool,
    pub special_move: Option<SpecialMove>,
}

/// parses PGN moves, no validation of the actual move. The only validation involved is to ensure
/// the source piece exists
pub fn parse_move(board: &Board, cmd: &str, is_white: bool) -> Result<Move, ParseError> {
    if cmd.len() <= 1 {
        // invalid
        return Err(ParseError::InvalidLength);
    }

    let mut chars = cmd.chars();
    let mut is_capture = false;
    let mut from: u64 = 0;
    let mut to: u64 = 0;
    let mut special_move: Option<SpecialMove> = None;
    let source = chars.next().unwrap();
    let piece = parse_source(source)?;

    match piece {
        Piece::Pawn => return parse_pawn(board, source, chars, is_white),

        Piece::Knight | Piece::Rook | Piece::Bishop | Piece::Queen | Piece::King => {}

        Piece::Castling => {
            if cmd.eq("O-O") {
                special_move = Some(SpecialMove::CastlingKing);
            } else if cmd.eq("O-O-O") {
                special_move = Some(SpecialMove::CastlingQueen);
            } else {
                return Err(ParseError::InvalidCastling);
            }
        }
    }
    Ok(Move {
        piece,
        from,
        to,
        is_capture,
        is_white,
        special_move,
    })
}

fn parse_pawn(board: &Board, source: char, mut chars: Chars, is_white: bool) -> Result<Move, ParseError> {
    let mut is_capture = false;
    let mut from: u64 = 0;
    let mut to: u64 = 0;
    let mut special_move: Option<SpecialMove> = None;

    #[derive(Debug, PartialEq)]
    enum PawnParserState {
        Initial,
        TargetParsed,
        Capturing,
        PromotionPiece,
    }

    let mut state = PawnParserState::Initial;
    let mut can_promote = false;

    let mut target_rank: u64 = 0;

    while let Some(c) = chars.next() {
        match state {
            PawnParserState::Initial => {
                match c {
                    rank @ '0'..='8' => {
                        target_rank = rank.to_digit(10).unwrap() as u64;
                        to = bitboard_single(source, target_rank).unwrap();
                        state = PawnParserState::TargetParsed;
                        can_promote = if is_white {
                            rank == '8'
                        } else {
                            rank == '1'
                        } ;
                    }
                    'x' => {
                        state = PawnParserState::Capturing;
                        is_capture = true;
                    }
                    _ => {
                        return Err(ParseError::InvalidTarget);
                    }
                }
            }
            PawnParserState::Capturing => {
                match c {
                    file @ 'a'..='h' => {
                        if let Some(c) = chars.next() {
                            match c {
                                rank @ '0'..='8' => {
                                    target_rank = rank.to_digit(10).unwrap() as u64;
                                    to = bitboard_single(file, target_rank).unwrap();
                                    state = PawnParserState::TargetParsed;
                                    can_promote = if is_white {
                                        rank == '8'
                                    } else {
                                        rank == '1'
                                    } ;
                                }
                                _ => {
                                    return Err(ParseError::InvalidTarget);
                                }
                            }
                        } else {
                            return Err(ParseError::InvalidTarget);
                        }
                    }
                    _ => {
                        return Err(ParseError::InvalidTarget);
                    }
                }
            }
            PawnParserState::TargetParsed => {
                match c {
                    '=' if can_promote => {
                        state = PawnParserState::PromotionPiece;
                    }
                    _ => {
                        return Err(ParseError::InvalidTarget);
                    }
                }
            }
            PawnParserState::PromotionPiece => {
                let promotion = match c {
                    'N' => Piece::Knight,
                    'R' => Piece::Rook,
                    'B' => Piece::Bishop,
                    'Q' => Piece::Queen,
                    'K' => Piece::King,
                    _ => {
                        return Err(ParseError::InvalidTarget);
                    }
                };
                special_move = Some(SpecialMove::Promotion(promotion));
            }
        }
    }

    render_bitboard(&to, 'p');
    from = detect_pawn_source(board, source, target_rank, to, is_capture, is_white);

    render_bitboard(&from, 'f');

    // final checks
    if from == 0 {
        return Err(ParseError::InvalidSource);
    }
    if to == 0 {
        return Err(ParseError::InvalidTarget);
    }
    if state == PawnParserState::PromotionPiece && special_move == None {
         return Err(ParseError::InvalidTarget);
    }

    Ok(Move {
        piece: Piece::Pawn,
        from,
        to,
        is_capture,
        is_white,
        special_move,
    })
}

fn parse_source(c: char) -> Result<Piece, ParseError> {
    match c {
        'a'..='h' => Ok(Piece::Pawn),
        'N' => Ok(Piece::Knight),
        'R' => Ok(Piece::Rook),
        'B' => Ok(Piece::Bishop),
        'Q' => Ok(Piece::Queen),
        'K' => Ok(Piece::King),
        'O' => Ok(Piece::Castling),
        _ => Err(ParseError::InvalidSource),
    }
}

#[cfg(test)]
pub mod tests {
    use crate::board::PositionBuilder;
    use super::*;

    #[test]
    fn test_parse_move() {
        let board = Board::default();
    }

    #[test]
    fn test_parse_pawn_basic_moves() {
        let board = Board::default();
        assert_eq!(
            Move{
                piece: Piece::Pawn,
                from: bitboard_single('e', 2).unwrap(),
                to: bitboard_single('e', 4).unwrap(),
                is_capture: false,
                is_white: true,
                special_move: None,
            },
            parse_move(&board, "e4", true).unwrap()
        );

        assert_eq!(
            Move{
                piece: Piece::Pawn,
                from: bitboard_single('f', 2).unwrap(),
                to: bitboard_single('f', 3).unwrap(),
                is_capture: false,
                is_white: true,
                special_move: None,
            },
            parse_move(&board, "f3", true).unwrap()
        );

        assert_eq!(
            Move{
                piece: Piece::Pawn,
                from: bitboard_single('e', 7).unwrap(),
                to: bitboard_single('e', 5).unwrap(),
                is_capture: false,
                is_white: false,
                special_move: None,
            },
            parse_move(&board, "e5", false).unwrap()
        );

        assert_eq!(
            Move{
                piece: Piece::Pawn,
                from: bitboard_single('h', 7).unwrap(),
                to: bitboard_single('h', 6).unwrap(),
                is_capture: false,
                is_white: false,
                special_move: None,
            },
            parse_move(&board, "h6", false).unwrap()
        );

        // white can't go to e5
        assert_eq!(
            Err(ParseError::InvalidSource),
            parse_move(&board, "e5", true)
        );

        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move(&board, "e9", true)
        );

        assert_eq!(
            Err(ParseError::InvalidLength),
            parse_move(&board, "a", true)
        );
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

        assert_eq!(
            Move{
                piece: Piece::Pawn,
                from: bitboard_single('e', 3).unwrap(),
                to: bitboard_single('d', 4).unwrap(),
                is_capture: true,
                is_white: true,
                special_move: None,
            },
            parse_move(&board, "exd4", true).unwrap()
        );

        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move(&board, "exd", true)
        );

        assert_eq!(
            Move{
                piece: Piece::Pawn,
                from: bitboard_single('g', 3).unwrap(),
                to: bitboard_single('h', 2).unwrap(),
                is_capture: true,
                is_white: false,
                special_move: None,
            },
            parse_move(&board, "gxh2", false).unwrap()
        );
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
            .add_piece('d', 2)
            .add_piece('g', 3)
            .build();
        let black_knights: u64 = PositionBuilder::new()
            .add_piece('b', 8)
            .add_piece('g', 8)
            .build();
        let board = Board::new(white_pawns, white_knights, 0, 0, 0, 0, black_pawns, black_knights, 0, 0, 0, 0);
        board.render();

        assert_eq!(
            Move{
                piece: Piece::Pawn,
                from: bitboard_single('h', 7).unwrap(),
                to: bitboard_single('g', 8).unwrap(),
                is_capture: true,
                is_white: true,
                special_move: Some(SpecialMove::Promotion(Piece::Rook)),
            },
            parse_move(&board, "hxg8=R", true).unwrap()
        );

        assert_eq!(
            Move{
                piece: Piece::Pawn,
                from: bitboard_single('d', 2).unwrap(),
                to: bitboard_single('d', 1).unwrap(),
                is_capture: false,
                is_white: false,
                special_move: Some(SpecialMove::Promotion(Piece::Queen)),
            },
            parse_move(&board, "d1=Q", false).unwrap()
        );

        assert_eq!(
            Move{
                piece: Piece::Pawn,
                from: bitboard_single('d', 2).unwrap(),
                to: bitboard_single('d', 1).unwrap(),
                is_capture: false,
                is_white: false,
                special_move: Some(SpecialMove::Promotion(Piece::Knight)),
            },
            parse_move(&board, "d1=N", false).unwrap()
        );

        assert_eq!(
            Move{
                piece: Piece::Pawn,
                from: bitboard_single('d', 2).unwrap(),
                to: bitboard_single('d', 1).unwrap(),
                is_capture: false,
                is_white: false,
                special_move: Some(SpecialMove::Promotion(Piece::Bishop)),
            },
            parse_move(&board, "d1=B", false).unwrap()
        );

        // can't promote if not at the end
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move(&board, "a3=Q", true)
        );

        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move(&board, "h8=", true)
        );

        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move(&board, "h8=O", true)
        );
    }


    #[test]
    fn test_parse_source() {
        assert_eq!(Ok(Piece::Pawn), parse_source('a'));
        assert_eq!(Ok(Piece::Pawn), parse_source('b'));
        assert_eq!(Ok(Piece::Pawn), parse_source('c'));
        assert_eq!(Ok(Piece::Pawn), parse_source('d'));
        assert_eq!(Ok(Piece::Pawn), parse_source('e'));
        assert_eq!(Ok(Piece::Pawn), parse_source('f'));
        assert_eq!(Ok(Piece::Pawn), parse_source('g'));
        assert_eq!(Ok(Piece::Pawn), parse_source('h'));
        assert_eq!(Ok(Piece::Knight), parse_source('N'));
        assert_eq!(Ok(Piece::Bishop), parse_source('B'));
        assert_eq!(Ok(Piece::Rook), parse_source('R'));
        assert_eq!(Ok(Piece::Queen), parse_source('Q'));
        assert_eq!(Ok(Piece::King), parse_source('K'));
        assert_eq!(Ok(Piece::Castling), parse_source('O'));
        assert_eq!(Err(ParseError::InvalidSource), parse_source('Z'));
        assert_eq!(Err(ParseError::InvalidSource), parse_source('1'));
    }
}
