use crate::board::bitboard_single;
use std::str::Chars;

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
pub struct ParsedMove {
    pub piece: Piece,
    /// from file and rank is optional (e.g. Nf3)
    pub from_file: Option<char>,
    pub from_rank: Option<u64>,
    pub to: u64,
    pub is_capture: bool,
    pub special_move: Option<SpecialMove>,
}

/// parses PGN moves, there is no validation of the move. All validations are
/// done on game.rs (this includes promotion logic)
/// It is only responsible to make sure the string is a correct PGN format
pub fn parse_move(cmd: &str) -> Result<ParsedMove, ParseError> {
    if cmd.len() <= 1 {
        // invalid
        return Err(ParseError::InvalidLength);
    }

    let mut chars = cmd.chars();
    let source = chars.next().unwrap();
    let piece = parse_source(source)?;

    match piece {
        Piece::Pawn => parse_pawn(source, chars),

        Piece::Knight | Piece::Rook | Piece::Bishop | Piece::Queen | Piece::King => {
            parse_piece(piece, chars)
        }

        Piece::Castling => parse_castling(cmd),
    }
}

fn parse_piece(piece: Piece, mut chars: Chars) -> Result<ParsedMove, ParseError> {
    let mut is_capture = false;
    let mut to: u64 = 0;

    #[derive(Debug, PartialEq)]
    enum PieceParserState {
        Initial,
        PotentialTargetFileParsed,
        PotentialTargetRankParsed,
        PotentialTargetParsed,
        SourceParsed,
        TargetFileParsed,
        TargetParsed,
    }

    let mut state = PieceParserState::Initial;

    let mut potential_target_rank: u64 = 0;
    let mut potential_target_file: char = ' ';

    let mut source_file: Option<char> = None;
    let mut source_rank: Option<u64> = None;

    while let Some(c) = chars.next() {
        match state {
            PieceParserState::Initial => match c {
                file @ 'a'..='h' => {
                    potential_target_file = file;
                    state = PieceParserState::PotentialTargetFileParsed;
                }
                rank @ '0'..='8' => {
                    potential_target_rank = rank.to_digit(10).unwrap() as u64;
                    state = PieceParserState::PotentialTargetRankParsed;
                }
                'x' => {
                    state = PieceParserState::SourceParsed;
                    is_capture = true;
                }
                _ => {
                    return Err(ParseError::InvalidTarget);
                }
            },

            PieceParserState::PotentialTargetFileParsed => match c {
                rank @ '0'..='8' => {
                    potential_target_rank = rank.to_digit(10).unwrap() as u64;
                    state = PieceParserState::PotentialTargetParsed;
                }
                'x' if piece != Piece::King => {
                    source_file = Some(potential_target_file);
                    potential_target_file = ' ';
                    state = PieceParserState::SourceParsed;
                    is_capture = true;
                }
                // handling ambiguous (exclude king)
                file @ 'a'..='h' if piece != Piece::King => {
                    source_file = Some(potential_target_file);
                    potential_target_file = file;
                    state = PieceParserState::TargetFileParsed;
                }
                _ => {
                    return Err(ParseError::InvalidTarget);
                }
            },
            PieceParserState::PotentialTargetRankParsed => match c {
                'x' => {
                    source_rank = Some(potential_target_rank);
                    potential_target_rank = 0;
                    state = PieceParserState::SourceParsed;
                    is_capture = true;
                }
                // handling ambiguous (exclude king)
                file @ 'a'..='h' if piece != Piece::King => {
                    source_rank = Some(potential_target_rank);
                    potential_target_file = file;
                    to = bitboard_single(potential_target_file, potential_target_rank).unwrap();
                    state = PieceParserState::TargetFileParsed;
                }
                _ => {
                    return Err(ParseError::InvalidTarget);
                }

            }
            PieceParserState::PotentialTargetParsed => match c {
                'x' if piece != Piece::King => {
                    source_file = Some(potential_target_file);
                    source_rank = Some(potential_target_rank);
                    potential_target_file= ' ';
                    potential_target_rank = 0;
                    state = PieceParserState::SourceParsed;
                    is_capture = true;
                }
                file@ 'a'..='h' if piece != Piece::King => {
                    source_file = Some(potential_target_file);
                    source_rank = Some(potential_target_rank);
                    potential_target_file = file;
                    to = bitboard_single(potential_target_file, potential_target_rank).unwrap();
                    state = PieceParserState::TargetFileParsed;
                }
                _ => {
                    return Err(ParseError::InvalidTarget);
                }
            }

            PieceParserState::SourceParsed => match c {
                file @ 'a'..='h' => {
                    potential_target_file = file;
                    state = PieceParserState::PotentialTargetFileParsed;
                }
                _ => {
                    return Err(ParseError::InvalidTarget);
                }
            },
            PieceParserState::TargetFileParsed => match c {
                rank @ '0'..='8' => {
                    potential_target_rank = rank.to_digit(10).unwrap() as u64;
                    to = bitboard_single(potential_target_file, potential_target_rank).unwrap();
                    state = PieceParserState::TargetParsed;
                }
                _ => {
                    return Err(ParseError::InvalidTarget);
                }
            }
            PieceParserState::TargetParsed => return match c {
                _ => {
                    Err(ParseError::InvalidTarget)
                }
            }
        }
    }

    // final checks
    if state == PieceParserState::PotentialTargetParsed {
        to = bitboard_single(potential_target_file, potential_target_rank).unwrap();
        state = PieceParserState::TargetParsed;
    }

    if state != PieceParserState::TargetParsed || to == 0 {
        return Err(ParseError::InvalidTarget);
    }

    Ok(ParsedMove {
        piece,
        from_file: source_file,
        from_rank: source_rank,
        to,
        is_capture,
        special_move: None,
    })
}

fn parse_castling(cmd: &str) -> Result<ParsedMove, ParseError> {
    let special_move: Option<SpecialMove>;
    if cmd.eq("O-O") {
        special_move = Some(SpecialMove::CastlingKing);
    } else if cmd.eq("O-O-O") {
        special_move = Some(SpecialMove::CastlingQueen);
    } else {
        return Err(ParseError::InvalidCastling);
    }

    Ok(ParsedMove {
        piece: Piece::Castling,
        from_file: None,
        from_rank: None,
        to: 0,
        is_capture: false,
        special_move,
    })
}

fn parse_pawn(source: char, mut chars: Chars) -> Result<ParsedMove, ParseError> {
    let mut is_capture = false;
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
    let mut target_rank: u64 = 0;

    while let Some(c) = chars.next() {
        match state {
            PawnParserState::Initial => match c {
                rank @ '1'..='8' => {
                    target_rank = rank.to_digit(10).unwrap() as u64;
                    to = bitboard_single(source, target_rank).unwrap();
                    state = PawnParserState::TargetParsed;
                }
                'x' => {
                    state = PawnParserState::Capturing;
                    is_capture = true;
                }
                _ => {
                    return Err(ParseError::InvalidTarget);
                }
            },
            PawnParserState::Capturing => match c {
                file @ 'a'..='h' => {
                    if let Some(c) = chars.next() {
                        match c {
                            rank @ '1'..='8' => {
                                target_rank = rank.to_digit(10).unwrap() as u64;
                                to = bitboard_single(file, target_rank).unwrap();
                                state = PawnParserState::TargetParsed;
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
            },
            PawnParserState::TargetParsed => match c {
                '=' => {
                    state = PawnParserState::PromotionPiece;
                }
                _ => {
                    return Err(ParseError::InvalidTarget);
                }
            },
            PawnParserState::PromotionPiece => {
                let promotion = match c {
                    'N' => Piece::Knight,
                    'R' => Piece::Rook,
                    'B' => Piece::Bishop,
                    'Q' => Piece::Queen,
                    _ => {
                        return Err(ParseError::InvalidTarget);
                    }
                };
                special_move = Some(SpecialMove::Promotion(promotion));
            }
        }
    }

    // final checks
    if to == 0 {
        return Err(ParseError::InvalidTarget);
    }
    if state == PawnParserState::PromotionPiece && special_move == None {
        return Err(ParseError::InvalidTarget);
    }

    Ok(ParsedMove {
        piece: Piece::Pawn,
        from_file: Some(source),
        from_rank: None,
        to,
        is_capture,
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
    use super::*;

    #[test]
    fn test_parse_pawn_basic_moves() {
        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('e'),
                from_rank: None,
                to: bitboard_single('e', 4).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("e4").unwrap()
        );

        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('f'),
                from_rank: None,
                to: bitboard_single('f', 3).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("f3").unwrap()
        );

        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('e'),
                from_rank: None,
                to: bitboard_single('e', 5).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("e5").unwrap()
        );

        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('h'),
                from_rank: None,
                to: bitboard_single('h', 6).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("h6").unwrap()
        );

        assert_eq!(Err(ParseError::InvalidSource), parse_move("x5"));

        assert_eq!(Err(ParseError::InvalidTarget), parse_move("e9"));

        assert_eq!(Err(ParseError::InvalidTarget), parse_move("e0"));

        assert_eq!(Err(ParseError::InvalidLength), parse_move("a"));
    }

    #[test]
    fn test_parse_pawn_capture() {
        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('e'),
                from_rank: None,
                to: bitboard_single('d', 4).unwrap(),
                is_capture: true,
                special_move: None,
            },
            parse_move("exd4").unwrap()
        );

        assert_eq!(Err(ParseError::InvalidTarget), parse_move("exd"));

        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('g'),
                from_rank: None,
                to: bitboard_single('h', 2).unwrap(),
                is_capture: true,
                special_move: None,
            },
            parse_move("gxh2").unwrap()
        );
    }

    #[test]
    fn test_parse_pawn_promotion() {
        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('h'),
                from_rank: None,
                to: bitboard_single('g', 8).unwrap(),
                is_capture: true,
                special_move: Some(SpecialMove::Promotion(Piece::Rook)),
            },
            parse_move("hxg8=R").unwrap()
        );

        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('d'),
                from_rank: None,
                to: bitboard_single('d', 1).unwrap(),
                is_capture: false,
                special_move: Some(SpecialMove::Promotion(Piece::Queen)),
            },
            parse_move("d1=Q").unwrap()
        );

        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('d'),
                from_rank: None,
                to: bitboard_single('d', 1).unwrap(),
                is_capture: false,
                special_move: Some(SpecialMove::Promotion(Piece::Knight)),
            },
            parse_move("d1=N").unwrap()
        );

        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('d'),
                from_rank: None,
                to: bitboard_single('d', 1).unwrap(),
                is_capture: false,
                special_move: Some(SpecialMove::Promotion(Piece::Bishop)),
            },
            parse_move("d1=B").unwrap()
        );

        // no check if promotion is valid (target rank is invalid)
        assert_eq!(
            ParsedMove {
                piece: Piece::Pawn,
                from_file: Some('a'),
                from_rank: None,
                to: bitboard_single('a', 3).unwrap(),
                is_capture: false,
                special_move: Some(SpecialMove::Promotion(Piece::Queen)),
            },
            parse_move("a3=Q").unwrap()
        );

        assert_eq!(Err(ParseError::InvalidTarget), parse_move("h8="));

        assert_eq!(Err(ParseError::InvalidTarget), parse_move("h8=O"));
    }

    #[test]
    fn test_parse_castling() {
        assert_eq!(
            ParsedMove {
                piece: Piece::Castling,
                from_file: None,
                from_rank: None,
                to: 0,
                is_capture: false,
                special_move: Some(SpecialMove::CastlingKing),
            },
            parse_move("O-O").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::Castling,
                from_file: None,
                from_rank: None,
                to: 0,
                is_capture: false,
                special_move: Some(SpecialMove::CastlingQueen),
            },
            parse_move("O-O-O").unwrap()
        );
        assert_eq!(Err(ParseError::InvalidCastling), parse_move("O-"));
    }

    #[test]
    fn test_parse_pieces_simple_moves() {
        assert_eq!(
            ParsedMove {
                piece: Piece::Knight,
                from_file: None,
                from_rank: None,
                to: bitboard_single('f', 3).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("Nf3").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::Queen,
                from_file: None,
                from_rank: None,
                to: bitboard_single('f', 3).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("Qf3").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::Bishop,
                from_file: None,
                from_rank: None,
                to: bitboard_single('a', 2).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("Ba2").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::Rook,
                from_file: None,
                from_rank: None,
                to: bitboard_single('h', 7).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("Rh7").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::King,
                from_file: None,
                from_rank: None,
                to: bitboard_single('e', 1).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("Ke1").unwrap()
        );
        assert_eq!(
            Err(ParseError::InvalidSource),
            parse_move("Je1")
        );
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move("Nz9")
        );
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move("Ne")
        );
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move("N1")
        );
    }

    #[test]
    fn test_parse_pieces_capture() {
        assert_eq!(
            ParsedMove {
                piece: Piece::Queen,
                from_file: None,
                from_rank: None,
                to: bitboard_single('b', 2).unwrap(),
                is_capture: true,
                special_move: None,
            },
            parse_move("Qxb2").unwrap()
        );
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move("Qxxb2")
        );
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move("Qx2")
        );
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move("Qxe")
        );
    }

    #[test]
    fn test_parse_pieces_ambiguous() {
        assert_eq!(
            ParsedMove {
                piece: Piece::Queen,
                from_file: Some('e'),
                from_rank: None,
                to: bitboard_single('b', 2).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("Qeb2").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::Queen,
                from_file: None,
                from_rank: Some(1),
                to: bitboard_single('b', 2).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("Q1b2").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::Queen,
                from_file: Some('h'),
                from_rank: Some(8),
                to: bitboard_single('b', 2).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("Qh8b2").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::Queen,
                from_file: Some('e'),
                from_rank: None,
                to: bitboard_single('b', 2).unwrap(),
                is_capture: true,
                special_move: None,
            },
            parse_move("Qexb2").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::Queen,
                from_file: None,
                from_rank: Some(1),
                to: bitboard_single('b', 2).unwrap(),
                is_capture: true,
                special_move: None,
            },
            parse_move("Q1xb2").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::Queen,
                from_file: Some('h'),
                from_rank: Some(8),
                to: bitboard_single('b', 2).unwrap(),
                is_capture: true,
                special_move: None,
            },
            parse_move("Qh8xb2").unwrap()
        );
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move("Qh8b2b")
        );
    }

    #[test]
    fn test_parse_king() {
        assert_eq!(
            ParsedMove {
                piece: Piece::King,
                from_file: None,
                from_rank: None,
                to: bitboard_single('e', 2).unwrap(),
                is_capture: false,
                special_move: None,
            },
            parse_move("Ke2").unwrap()
        );
        assert_eq!(
            ParsedMove {
                piece: Piece::King,
                from_file: None,
                from_rank: None,
                to: bitboard_single('e', 2).unwrap(),
                is_capture: true,
                special_move: None,
            },
            parse_move("Kxe2").unwrap()
        );
        // king ambiguity resolution is not allowed in PGN
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move("Kef2")
        );
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move("Ke2e3")
        );
        assert_eq!(
            Err(ParseError::InvalidTarget),
            parse_move("Ke2xe3")
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
