use crate::board::{is_rank, render_bitboard, Board, MASK_FILE_A, MASK_FILE_H, MASK_RANK_2, MASK_RANK_7};

// move generation related, only generate pseudolegal moves

// PAWNS
pub fn compute_pawns_moves(board: &Board, is_white: bool) -> u64 {
    let moves: u64;

    let single_moves = compute_pawns_single_moves(&board, is_white);
    let capture_diagonals = compute_pawns_diagonal_captures(&board, is_white);
    let double_moves = compute_pawns_double_moves(&board, is_white);

    moves = single_moves | double_moves | capture_diagonals;

    // TODO - en passant
    moves
}

fn compute_pawns_single_moves(board: &Board, is_white: bool) -> u64 {
    let mut moves: u64;

    if is_white {
        moves = board.white_pawns << 8 & board.free;
    } else {
        moves = board.black_pawns >> 8 & board.free;
    }
    moves
}

fn compute_pawns_double_moves(board: &Board, is_white: bool) -> u64 {
    let mut moves: u64;

    // upward move
    if is_white {
        // only rank 2 can move twice
        // moves 1 step first and filter for blockage on rank 3
        moves = ((board.white_pawns & MASK_RANK_2) << 8) & board.free;
        moves = (moves << 8) & board.free; // move another step + only eligible spot
    } else {
        // only rank 7 can move twice
        // moves 1 step first and filter for blockage on rank 6
        moves = ((board.black_pawns & MASK_RANK_7) >> 8) & board.free;
        moves = (moves >> 8) & board.free; // move another step
    }

    moves
}

fn compute_pawns_diagonal_captures(board: &Board, is_white: bool) -> u64 {
    let opponents: u64;
    let mut moves: u64;
    let left_diagonal: u64;
    let right_diagonal: u64;

    if is_white {
        moves = board.white_pawns;
        opponents = board.black_pieces;
        left_diagonal = moves << 7 & opponents & !MASK_FILE_H; // prevent wrap-around on H file for left diagonal move
        right_diagonal = moves << 9 & opponents & !MASK_FILE_A; // prevent wrap-around on A file on right diagonal move

    } else {
        moves = board.black_pawns;
        opponents = board.white_pieces;
        left_diagonal = moves >> 9 & opponents & !MASK_FILE_H; // prevent wrap-around on H file for left diagonal move
        right_diagonal = moves >> 7 & opponents & !MASK_FILE_A; // prevent wrap-around on A file on right diagonal move
    }

    moves = left_diagonal | right_diagonal;
    moves
}

// REVERSE ENGINEER MOVES

pub fn detect_pawns_source_for_target(board: &Board, target: u64, is_white: bool) -> Vec<u64> {
    // Reverse shifts and bitwise checks to find the source square
    let mut sources = Vec::new();


    let legal_pawn_moves = compute_pawns_moves(&board, is_white);
    render_bitboard(&legal_pawn_moves, 'c');

    println!("legal to target{}", legal_pawn_moves >> target);
    if (legal_pawn_moves >> target) == 0 {
        println!("HA");
        return sources;
    }

    let single_move_source = detect_pawns_source_for_target_single_move(board, target, is_white);

    sources.extend(single_move_source);

    sources
}

fn detect_pawns_source_for_target_single_move(board: &Board, target: u64, is_white: bool) -> Vec<u64> {
    let mut sources = Vec::new();
        let pawns = if is_white {
        board.white_pawns
    } else {
        board.black_pawns
    };

    let legal_single_moves = compute_pawns_single_moves(&board, is_white);
    // only allow legal moves
    if (target & legal_single_moves) == 0 {
        return sources;
    }

    if is_white && !is_rank(target, 1) { // only process if target is rank 2 and above
        let possible_source = target >> 8;
        println!("possible source {} ", possible_source);
        if (pawns | possible_source) == pawns {
            sources.push(possible_source);
        }
    } else if !is_white && !is_rank(target, 8) { // only process if target is rank 7 and below
        let possible_source = target << 8;
        if (pawns | possible_source) == pawns {
            sources.push(possible_source);
        }
    }
    sources
}

fn detect_pawns_source_for_target_double_move(board: &Board, target: u64, is_white: bool) -> Vec<u64> {
    let mut sources = Vec::new();
        let pawns = if is_white {
        board.white_pawns
    } else {
        board.black_pawns
    };

    let legal_double_moves = compute_pawns_double_moves(&board, is_white);
    // only allow legal moves
    if (target & legal_double_moves) == 0 {
        return sources;
    }

    if is_white && !is_rank(target, 1) && !is_rank(target, 2) { // only process if target is rank 3 and above
        let possible_source = target >> 16;
        println!("possible source {} ", possible_source);
        if (pawns & possible_source) != 0 {
            sources.push(possible_source);
        }
    } else if !is_white && !is_rank(target, 8) && !is_rank(target, 7) { // only process if target is rank 6 and below
        let possible_source = target << 16;
        if (pawns & possible_source) != 0 {
            sources.push(possible_source);
        }
    }
    sources
}

#[cfg(test)]
pub mod tests {
    use crate::board::{bit_pos, bitboard_single, render_bitboard, Board, PositionBuilder};
    use super::*;
    #[test]
    fn test_detect_pawn_source_for_target_single_move() {
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
        let board = Board::new(white_pawns, black_pawns);

        let expected_e4_white = vec![bitboard_single('e', 3).unwrap()];
        let expected_g3_white: Vec<u64> = Vec::new();
        let expected_h3_white = vec![bitboard_single('h', 2).unwrap()];
        let expected_g2_black: Vec<u64> = Vec::new();
        let expected_a6_black = vec![bitboard_single('a', 7).unwrap()];

        assert_eq!(expected_e4_white, detect_pawns_source_for_target_single_move(&board, bitboard_single('e', 4).unwrap(), true));
        assert_eq!(expected_g3_white, detect_pawns_source_for_target_single_move(&board, bitboard_single('g', 3).unwrap(), true));
        assert_eq!(expected_h3_white, detect_pawns_source_for_target_single_move(&board, bitboard_single('h', 3).unwrap(), true));
        assert_eq!(expected_g2_black, detect_pawns_source_for_target_single_move(&board, bitboard_single('g', 2).unwrap(), false));
        assert_eq!(expected_a6_black, detect_pawns_source_for_target_single_move(&board, bitboard_single('a', 6).unwrap(), false));
    }
    
    #[test]
    fn test_detect_pawn_source_for_target_double_move() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('e', 2) //blocked by e3
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
        let board = Board::new(white_pawns, black_pawns);

        let expected_a4_white = vec![bitboard_single('a', 2).unwrap()];
        let expected_e4_white: Vec<u64> = Vec::new();
        let expected_g4_white: Vec<u64> = Vec::new();
        let expected_h4_white = vec![bitboard_single('h', 2).unwrap()];
        let expected_a5_black = vec![bitboard_single('a', 7).unwrap()];
        let expected_d5_black: Vec<u64> = Vec::new();

        assert_eq!(expected_a4_white, detect_pawns_source_for_target_double_move(&board, bitboard_single('a', 4).unwrap(), true));
        assert_eq!(expected_e4_white, detect_pawns_source_for_target_double_move(&board, bitboard_single('e', 4).unwrap(), true));
        assert_eq!(expected_g4_white, detect_pawns_source_for_target_double_move(&board, bitboard_single('g', 4).unwrap(), true));
        assert_eq!(expected_h4_white, detect_pawns_source_for_target_double_move(&board, bitboard_single('h', 4).unwrap(), true));
        assert_eq!(expected_a5_black, detect_pawns_source_for_target_double_move(&board, bitboard_single('a', 5).unwrap(), false));
        assert_eq!(expected_d5_black, detect_pawns_source_for_target_double_move(&board, bitboard_single('d', 5).unwrap(), false));
    }

    #[test]
    fn test_diagonal_captures_pawn() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('e', 3)
            .add_piece('a', 2)
            .add_piece('h', 2)
            .build();
        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 7)
            .add_piece('d', 4)
            .add_piece('g', 3)
            .build();

        let board = Board::new(white_pawns, black_pawns);

        let expected_white_moves: u64 = PositionBuilder::new()
            .add_piece('d', 4)
            .add_piece('g', 3)
            .build();
        let expected_black_moves: u64 = PositionBuilder::new()
            .add_piece('e', 3)
            .add_piece('h', 2)
            .build();

        // assert_eq!(expected_white_moves, compute_diagonal_captures_pawns(&board, true));
        assert_eq!(expected_black_moves, compute_pawns_diagonal_captures(&board, false));
    }

    #[test]
    fn test_pawns_double_moves() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 2)
            .add_piece('d', 2)
            .add_piece('e', 3)
            .add_piece('f', 2)
            .add_piece('g', 6) // block rank 6 for black
            .build();

        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 3) // block rank 2 for white
            .add_piece('b', 7)
            .add_piece('c', 6)
            .add_piece('g', 7)
            .add_piece('h', 7)
            .build();

        let expected_white_moves: u64 = PositionBuilder::new()
            // a is blocked
            .add_piece('d', 4)
            .add_piece('f', 4)
            .build();
        let expected_black_moves: u64 = PositionBuilder::new()
            .add_piece('b', 5)
            // g is blocked
            .add_piece('h', 5)
            .build();

        let board = Board::new(white_pawns, black_pawns);
        render_bitboard(&compute_pawns_double_moves(&board, true), 'm');

        assert_eq!(expected_white_moves, compute_pawns_double_moves(&board, true));
        assert_eq!(expected_black_moves, compute_pawns_double_moves(&board, false));
    }


    #[test]
    fn test_pawns_single_moves() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 2)// blocked
            .add_piece('b', 8)// can't move
            .add_piece('d', 2)
            .add_piece('e', 3)
            .add_piece('f', 2)
            .build();

        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 3) // block
            .add_piece('g', 1) // can't move
            .add_piece('h', 7)
            .build();

        let expected_white_moves: u64 = PositionBuilder::new()
            .add_piece('d', 3)
            .add_piece('e', 4)
            .add_piece('f', 3)
            .build();

        let expected_black_moves: u64 = PositionBuilder::new()
            .add_piece('h', 6)
            .build();

        let board = Board::new(white_pawns, black_pawns);

        render_bitboard(&compute_pawns_single_moves(&board, false), 'W');

        assert_eq!(expected_white_moves, compute_pawns_single_moves(&board, true));
        assert_eq!(expected_black_moves, compute_pawns_single_moves(&board, false));
    }



    #[test]
    fn test_white_pawns_moves() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 2)
            .add_piece('d', 2)
            .add_piece('e', 3)
            .add_piece('f', 2)
            .build();

        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 3) // block rank 3
            .build();

        let expected: u64 = PositionBuilder::new()
            .add_piece('d', 3)
            .add_piece('d', 4)
            .add_piece('e', 4)
            .add_piece('f', 3)
            .add_piece('f', 4)
            .build();

        let board = Board::new(white_pawns, black_pawns);

        assert_eq!(expected, compute_pawns_moves(&board, true));
    }

    #[test]
    fn test_black_pawns_moves() {
        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 7)
            .add_piece('d', 2)
            .add_piece('e', 3)
            .add_piece('f', 1) // can't move
            .build();

        let expected: u64 = PositionBuilder::new()
            .add_piece('a', 6)
            .add_piece('a', 5)
            .add_piece('d', 1)
            .add_piece('e', 2)
            .build();

        let board = Board::new(0, black_pawns);

        assert_eq!(expected, compute_pawns_moves(&board, false));
    }
}