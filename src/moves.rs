use crate::board::{
    is_rank, render_bitboard, Board, MASK_FILE_A, MASK_FILE_B, MASK_FILE_G, MASK_FILE_H,
    MASK_RANK_2, MASK_RANK_7,
};

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

pub const KNIGHT_MOVES: [u64; 64] = [
    precompute_knight_moves(0),
    precompute_knight_moves(1),
    precompute_knight_moves(2),
    precompute_knight_moves(3),
    precompute_knight_moves(4),
    precompute_knight_moves(5),
    precompute_knight_moves(6),
    precompute_knight_moves(7),
    precompute_knight_moves(8),
    precompute_knight_moves(9),
    precompute_knight_moves(10),
    precompute_knight_moves(11),
    precompute_knight_moves(12),
    precompute_knight_moves(13),
    precompute_knight_moves(14),
    precompute_knight_moves(15),
    precompute_knight_moves(16),
    precompute_knight_moves(17),
    precompute_knight_moves(18),
    precompute_knight_moves(19),
    precompute_knight_moves(20),
    precompute_knight_moves(21),
    precompute_knight_moves(22),
    precompute_knight_moves(23),
    precompute_knight_moves(24),
    precompute_knight_moves(25),
    precompute_knight_moves(26),
    precompute_knight_moves(27),
    precompute_knight_moves(28),
    precompute_knight_moves(29),
    precompute_knight_moves(30),
    precompute_knight_moves(31),
    precompute_knight_moves(32),
    precompute_knight_moves(33),
    precompute_knight_moves(34),
    precompute_knight_moves(35),
    precompute_knight_moves(36),
    precompute_knight_moves(37),
    precompute_knight_moves(38),
    precompute_knight_moves(39),
    precompute_knight_moves(40),
    precompute_knight_moves(41),
    precompute_knight_moves(42),
    precompute_knight_moves(43),
    precompute_knight_moves(44),
    precompute_knight_moves(45),
    precompute_knight_moves(46),
    precompute_knight_moves(47),
    precompute_knight_moves(48),
    precompute_knight_moves(49),
    precompute_knight_moves(50),
    precompute_knight_moves(51),
    precompute_knight_moves(52),
    precompute_knight_moves(53),
    precompute_knight_moves(54),
    precompute_knight_moves(55),
    precompute_knight_moves(56),
    precompute_knight_moves(57),
    precompute_knight_moves(58),
    precompute_knight_moves(59),
    precompute_knight_moves(60),
    precompute_knight_moves(61),
    precompute_knight_moves(62),
    precompute_knight_moves(63),
];

// precompute all the moves available for knights at each bit index in the bitboard
const fn precompute_knight_moves(index: u8) -> u64 {
    let bitboard = 1u64 << index;
    // use mask to avoid wrap around
    ((bitboard << 17) & !MASK_FILE_A) // UP 2 + LEFT 1
    | ((bitboard << 15) & !MASK_FILE_H) // UP 2 + RIGHT 1
    | ((bitboard << 10) & !(MASK_FILE_A | MASK_FILE_B)) // UP 1 + RIGHT 2
    | ((bitboard << 6) & !(MASK_FILE_G | MASK_FILE_H)) // UP 1 + LEFT 2
    | ((bitboard >> 17) & !MASK_FILE_H) // DOWN 2 + RIGHT 1
    | ((bitboard >> 15) & !MASK_FILE_A) // DOWN 2 + LEFT 1
    | ((bitboard >> 10) & !(MASK_FILE_G | MASK_FILE_H)) // DOWN 1 + LEFT 2
    | ((bitboard >> 6) & !(MASK_FILE_A | MASK_FILE_B)) // DOWN 1 + RIGHT 2
}

pub fn compute_knights_moves(board: &Board, is_white: bool) -> u64 {
    let mut moves = 0u64;
    let own_pieces: u64;
    let mut knights: u64;
    if is_white {
        knights = board.white_knights;
        own_pieces = board.white_pieces;
    } else {
        knights = board.black_knights;
        own_pieces = board.black_pieces;
    };

    while knights != 0 {
        let index = knights.trailing_zeros();

        // Add the knight's precomputed moves, excluding occupied by own
        moves |= KNIGHT_MOVES[index as usize] & !own_pieces;

        // Remove the processed knight (use lsb approach)
        knights &= knights - 1;
    }

    moves
}

// TODO use macro to store in const
// clockwise direction
fn precompute_rook_rays(index: u8) -> [u64; 4] {
    let mut top: u64 = 0;
    let mut right: u64 = 0;
    let mut bottom: u64 = 0;
    let mut left: u64 = 0;
    let bitboard = 1u64 << index;

    let file = index % 8;
    let rank = index / 8;
    println!("file={} rank={}", file, rank);

    for r in (rank + 1)..8 {
        top |= 1u64 << (r * 8 + file);
    }
    for f in (file + 1)..8 {
        right |= 1u64 << (rank * 8 + f);
    }
    for r in (0..rank) {
        bottom |= 1u64 << (r * 8 + file);
    }
    for f in (0..file) {
        left |= 1u64 << (rank * 8 + f);
    }

    [top, right, bottom, left]
}

pub fn compute_rooks_moves(board: &Board, is_white: bool) -> u64 {
    let mut moves = 0u64;
    let own_pieces: u64;
    let opponents: u64;
    let mut rooks: u64;
    let occupied = board.occupied;
    if is_white {
        rooks = board.white_rooks;
        own_pieces = board.white_pieces;
        opponents = board.black_pieces;
    } else {
        rooks = board.black_rooks;
        own_pieces = board.black_pieces;
        opponents = board.white_pieces;
    };


    while rooks != 0 {
        let index = rooks.trailing_zeros();
        let rays = precompute_rook_rays(index as u8);

        for dir in 0..4 {
            let ray = rays[dir];
            let blockers: u64 = ray & occupied;
            if blockers == 0 {
                // not occupied, add the whole move
                moves |= ray
            } else {
                let blocked_idx: u32;
                let blocked_bit: u64;

                if dir == 0 || dir == 1 {
                   // for top/right ray, we find the index using trailing zeros position
                   blocked_idx = blockers.trailing_zeros();
                } else {
                    // for bottom/left ray, we find the index using leading ones position
                    // 63 minus X is required because we are shifting to the left
                    blocked_idx = 63 - blockers.leading_zeros();
                }

                println!("direction={dir} - blocked idx={blocked_idx}");

                blocked_bit = 1 << blocked_idx;
                if blocked_bit & own_pieces == 0 {
                    // opponent piece, we can move here
                    moves |= blocked_bit;
                }


                // for top    do ray & !(u64::MAX << blocked_idx) CONFIRM (exclusive)
                // for left   do ray &  (u64::MAX << blocked_idx + 1) CONFIRM (exclusive)
                // for bottom do ray &  (u64::MAX << blocked_idx + 1) CONFIRM (exclusive)
                // for right  do ray & !(u64::MAX << blocked_idx) CONFIRM (exclusive)

                if dir == 0 || dir == 1 {
                    // top/right
                    // moves |= ray & !blocked_bitboard;
                    moves |= ray & !(u64::MAX << blocked_idx);

                } else {
                    // bottom/left
                    moves |= ray & (u64::MAX << blocked_idx + 1);
                }
            }
        }


        // Remove the processed rooks (use lsb approach)
        rooks &= rooks - 1;
    }

    moves
}

fn precompute_bishop_rays(index: u8) -> [u64; 4] {
    let mut top_right: u64 = 0;
    let mut bottom_right: u64 = 0;
    let mut bottom_left: u64 = 0;
    let mut top_left: u64 = 0;

    let file = index % 8;
    let rank = index / 8;
    println!("file={} rank={}", file, rank);


    let mut f = file;
    let mut r = rank;

    f = file + 1;
    r = rank + 1;
    while f < 8 && r < 8 {
        top_right |= 1u64 << (r * 8 + f);
        f = f + 1;
        r = r + 1;
    }

    f = file + 1;
    r = rank.wrapping_sub(1);
    while f < 8 && r < 8 {
        bottom_right |= 1u64 << (r * 8 + f);
        f = f + 1;
        r = r.wrapping_sub(1); // when out of bound this will go back to 255
    }

    f = file.wrapping_sub(1);
    r = rank.wrapping_sub(1);
    while f < 8 && r < 8 {
        bottom_left |= 1u64 << (r * 8 + f);
        f = f.wrapping_sub(1);
        r = r.wrapping_sub(1); // when out of bound this will go back to 255
    }

    f = file.wrapping_sub(1);
    r = rank + 1;
    while f < 8 && r < 8 {
        top_left |= 1u64 << (r * 8 + f);
        f = f.wrapping_sub(1);
        r = r + 1; // when out of bound this will go back to 255
    }

    [top_right, bottom_right, bottom_left, top_left]
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

fn detect_pawns_source_for_target_single_move(
    board: &Board,
    target: u64,
    is_white: bool,
) -> Vec<u64> {
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

    if is_white && !is_rank(target, 1) {
        // only process if target is rank 2 and above
        let possible_source = target >> 8;
        println!("possible source {} ", possible_source);
        if (pawns | possible_source) == pawns {
            sources.push(possible_source);
        }
    } else if !is_white && !is_rank(target, 8) {
        // only process if target is rank 7 and below
        let possible_source = target << 8;
        if (pawns | possible_source) == pawns {
            sources.push(possible_source);
        }
    }
    sources
}

fn detect_pawns_source_for_target_double_move(
    board: &Board,
    target: u64,
    is_white: bool,
) -> Vec<u64> {
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

    if is_white && !is_rank(target, 1) && !is_rank(target, 2) {
        // only process if target is rank 3 and above
        let possible_source = target >> 16;
        println!("possible source {} ", possible_source);
        if (pawns & possible_source) != 0 {
            sources.push(possible_source);
        }
    } else if !is_white && !is_rank(target, 8) && !is_rank(target, 7) {
        // only process if target is rank 6 and below
        let possible_source = target << 16;
        if (pawns & possible_source) != 0 {
            sources.push(possible_source);
        }
    }
    sources
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::board::{bit_pos, bitboard_single, render_bitboard, Board, PositionBuilder};
    use crossterm::queue;
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
        let board = Board::new(white_pawns, 0, 0, 0, 0, 0, black_pawns, 0, 0, 0, 0, 0);

        let expected_e4_white = vec![bitboard_single('e', 3).unwrap()];
        let expected_g3_white: Vec<u64> = Vec::new();
        let expected_h3_white = vec![bitboard_single('h', 2).unwrap()];
        let expected_g2_black: Vec<u64> = Vec::new();
        let expected_a6_black = vec![bitboard_single('a', 7).unwrap()];

        assert_eq!(
            expected_e4_white,
            detect_pawns_source_for_target_single_move(
                &board,
                bitboard_single('e', 4).unwrap(),
                true
            )
        );
        assert_eq!(
            expected_g3_white,
            detect_pawns_source_for_target_single_move(
                &board,
                bitboard_single('g', 3).unwrap(),
                true
            )
        );
        assert_eq!(
            expected_h3_white,
            detect_pawns_source_for_target_single_move(
                &board,
                bitboard_single('h', 3).unwrap(),
                true
            )
        );
        assert_eq!(
            expected_g2_black,
            detect_pawns_source_for_target_single_move(
                &board,
                bitboard_single('g', 2).unwrap(),
                false
            )
        );
        assert_eq!(
            expected_a6_black,
            detect_pawns_source_for_target_single_move(
                &board,
                bitboard_single('a', 6).unwrap(),
                false
            )
        );
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
        let board = Board::new(white_pawns, 0, 0, 0, 0, 0, black_pawns, 0, 0, 0, 0, 0);

        let expected_a4_white = vec![bitboard_single('a', 2).unwrap()];
        let expected_e4_white: Vec<u64> = Vec::new();
        let expected_g4_white: Vec<u64> = Vec::new();
        let expected_h4_white = vec![bitboard_single('h', 2).unwrap()];
        let expected_a5_black = vec![bitboard_single('a', 7).unwrap()];
        let expected_d5_black: Vec<u64> = Vec::new();

        assert_eq!(
            expected_a4_white,
            detect_pawns_source_for_target_double_move(
                &board,
                bitboard_single('a', 4).unwrap(),
                true
            )
        );
        assert_eq!(
            expected_e4_white,
            detect_pawns_source_for_target_double_move(
                &board,
                bitboard_single('e', 4).unwrap(),
                true
            )
        );
        assert_eq!(
            expected_g4_white,
            detect_pawns_source_for_target_double_move(
                &board,
                bitboard_single('g', 4).unwrap(),
                true
            )
        );
        assert_eq!(
            expected_h4_white,
            detect_pawns_source_for_target_double_move(
                &board,
                bitboard_single('h', 4).unwrap(),
                true
            )
        );
        assert_eq!(
            expected_a5_black,
            detect_pawns_source_for_target_double_move(
                &board,
                bitboard_single('a', 5).unwrap(),
                false
            )
        );
        assert_eq!(
            expected_d5_black,
            detect_pawns_source_for_target_double_move(
                &board,
                bitboard_single('d', 5).unwrap(),
                false
            )
        );
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

        let board = Board::new(white_pawns, 0, 0, 0, 0, 0, black_pawns, 0, 0, 0, 0, 0);

        let expected_white_moves: u64 = PositionBuilder::new()
            .add_piece('d', 4)
            .add_piece('g', 3)
            .build();
        let expected_black_moves: u64 = PositionBuilder::new()
            .add_piece('e', 3)
            .add_piece('h', 2)
            .build();

        assert_eq!(
            expected_white_moves,
            compute_pawns_diagonal_captures(&board, true)
        );
        assert_eq!(
            expected_black_moves,
            compute_pawns_diagonal_captures(&board, false)
        );
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

        let board = Board::new(white_pawns, 0, 0, 0, 0, 0, black_pawns, 0, 0, 0, 0, 0);
        render_bitboard(&compute_pawns_double_moves(&board, true), 'm');

        assert_eq!(
            expected_white_moves,
            compute_pawns_double_moves(&board, true)
        );
        assert_eq!(
            expected_black_moves,
            compute_pawns_double_moves(&board, false)
        );
    }

    #[test]
    fn test_pawns_single_moves() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 2) // blocked
            .add_piece('b', 8) // can't move
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

        let expected_black_moves: u64 = PositionBuilder::new().add_piece('h', 6).build();

        let board = Board::new(white_pawns, 0, 0, 0, 0, 0, black_pawns, 0, 0, 0, 0, 0);

        render_bitboard(&compute_pawns_single_moves(&board, false), 'W');

        assert_eq!(
            expected_white_moves,
            compute_pawns_single_moves(&board, true)
        );
        assert_eq!(
            expected_black_moves,
            compute_pawns_single_moves(&board, false)
        );
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

        let board = Board::new(white_pawns, 0, 0, 0, 0, 0, black_pawns, 0, 0, 0, 0, 0);

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

        let board = Board::new(0, 0, 0, 0, 0, 0, black_pawns, 0, 0, 0, 0, 0);

        assert_eq!(expected, compute_pawns_moves(&board, false));
    }

    #[test]
    fn test_precompute_knight_moves() {
        let expected_knights_normal_f3: u64 = PositionBuilder::new()
            .add_piece('e', 5)
            .add_piece('g', 5)
            .add_piece('d', 4)
            .add_piece('h', 4)
            .add_piece('d', 2)
            .add_piece('h', 2)
            .add_piece('e', 1)
            .add_piece('g', 1)
            .build();

        let expected_knights_edge_a8: u64 = PositionBuilder::new()
            .add_piece('c', 7)
            .add_piece('b', 6)
            .build();

        assert_eq!(
            expected_knights_normal_f3,
            precompute_knight_moves(bit_pos('f', 3).unwrap() as u8)
        );
        assert_eq!(
            expected_knights_edge_a8,
            precompute_knight_moves(bit_pos('a', 8).unwrap() as u8)
        );
    }

    #[test]
    fn test_compute_knights_moves() {
        let white_pawns = PositionBuilder::new()
            .add_piece('a', 2)
            .add_piece('b', 5)
            .build();

        let white_knights = PositionBuilder::new().add_piece('c', 3).build();

        let black_pawns = PositionBuilder::new()
            .add_piece('d', 5)
            .add_piece('f', 6)
            .add_piece('g', 6)
            .build();

        let black_knights = PositionBuilder::new()
            .add_piece('e', 4)
            .add_piece('h',8)
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

        // should be blocked on a2 and b5
        let expected_white_knights_moves = PositionBuilder::new()
            .add_piece('a', 4)
            .add_piece('b', 1)
            .add_piece('e', 2)
            .add_piece('e', 4) // can captue e4
            .add_piece('d', 1)
            .add_piece('d', 5) // can capture d5
            .build();

        // should be blocked on f6 (for e4 knight) and g6 (for h8 knight)
        let expected_black_knights_moves = PositionBuilder::new()
            // for e4 knight
            .add_piece('c', 5)
            .add_piece('d', 6)
            .add_piece('c', 3)// can capture c3
            .add_piece('d', 2)
            .add_piece('f', 2)
            .add_piece('g', 3)
            .add_piece('g', 5)
            // for h8 knight
            .add_piece('f', 7)
            .build();

        board.render();

        assert_eq!(expected_white_knights_moves, compute_knights_moves(&board, true));
        assert_eq!(expected_black_knights_moves, compute_knights_moves(&board, false));
    }

    #[test]
    fn test_precompute_rook_rays() {
        let rays = precompute_rook_rays(bit_pos('e',4).unwrap() as u8);

        let expected_top_moves = PositionBuilder::new()
            .add_piece('e', 5)
            .add_piece('e', 6)
            .add_piece('e', 7)
            .add_piece('e', 8)
            .build();
        let expected_right_moves = PositionBuilder::new()
            .add_piece('f', 4)
            .add_piece('g', 4)
            .add_piece('h', 4)
            .build();
        let expected_bottom_moves = PositionBuilder::new()
            .add_piece('e', 3)
            .add_piece('e', 2)
            .add_piece('e', 1)
            .build();
        let expected_left_moves = PositionBuilder::new()
            .add_piece('d', 4)
            .add_piece('c', 4)
            .add_piece('b', 4)
            .add_piece('a', 4)
            .build();

        assert_eq!([expected_top_moves, expected_right_moves, expected_bottom_moves, expected_left_moves], precompute_rook_rays(bit_pos('e',4).unwrap() as u8));
    }

    #[test]
    fn test_compute_rook_moves() {
        let white_pawns = PositionBuilder::new()
            .add_piece('a', 2)
            .add_piece('b', 4)
            .add_piece('e', 2)
            .build();
        let white_knights = PositionBuilder::new()
            .add_piece('e', 1)
            .build();

        let white_rooks = PositionBuilder::new()
            .add_piece('a',1)
            .add_piece('e', 4)
            .build();

        let black_pawns = PositionBuilder::new()
            // .add_piece('e', 5)
            .add_piece('e', 8)
            .add_piece('f', 6)
            .add_piece('g', 6)
            .build();

        let black_rooks = PositionBuilder::new()
            .add_piece('a', 8)
            .add_piece('h', 8)
            .build();

        let board = Board::new(
            white_pawns,
            white_knights,
            white_rooks,
            0,
            0,
            0,
            black_pawns,
            0,
            black_rooks,
            0,
            0,
            0,
        );

        board.render();

        let expected_white_rooks_moves = PositionBuilder::new()
            // a1 rook
            .add_piece('b', 1)
            .add_piece('c', 1)
            .add_piece('d', 1)

            // e4 rook
            .add_piece('e', 5)
            .add_piece('e', 6)
            .add_piece('e', 7)
            // e8 can be captured
            .add_piece('e', 8)
            // bottom only up to e3 (e2 blocked)
            .add_piece('e', 3)
            .add_piece('c', 4)
            .add_piece('d', 4)
            .add_piece('f', 4)
            .add_piece('g', 4)
            .add_piece('h', 4)
            .build();

        let expected_black_rooks_moves = PositionBuilder::new()
            // a8 rook
            .add_piece('a', 7)
            .add_piece('a', 6)
            .add_piece('a', 5)
            .add_piece('a', 4)
            .add_piece('a', 3)
            // a2 can be captured
            .add_piece('a', 2)
            .add_piece('b', 8)
            .add_piece('c', 8)
            // e8 is blocked
            .add_piece('d', 8)

            // h8 rook
            .add_piece('f', 8)
            .add_piece('g', 8)
            .add_piece('h', 7)
            .add_piece('h', 6)
            .add_piece('h', 5)
            .add_piece('h', 4)
            .add_piece('h', 3)
            .add_piece('h', 2)
            .add_piece('h', 1)
            .build();

        let white = compute_rooks_moves(&board, false);
        assert_eq!(expected_white_rooks_moves, compute_rooks_moves(&board, true));
        assert_eq!(expected_black_rooks_moves, compute_rooks_moves(&board, false));
    }

    // TODO remove the belo test later

    #[test]
    fn test_top() {
        let rook = PositionBuilder::new()
            .add_piece('e', 3)
            .build();

        let blocked = PositionBuilder::new()
            .add_piece('e', 7)
            .build();

        let blocked_idx = bit_pos('e',7).unwrap();

        let ray = PositionBuilder::new()
            .add_piece('e', 4)
            .add_piece('e', 5)
            .add_piece('e', 6)
            .add_piece('e', 7)
            .add_piece('e', 8)
            .build();

        render_bitboard(&rook, 'R');
        render_bitboard(&blocked, 'B');
        render_bitboard(&ray, 'x');

        let blocked_ray = u64::MAX << blocked_idx;
        println!("block ray");
        render_bitboard(&blocked_ray, '*');
        println!("inverted block ray");
        render_bitboard(&!blocked_ray, '~');
        println!("AVAILABLE");
        render_bitboard(&(ray & !blocked_ray), '+');


    }

    #[test]
    fn test_left() {
        let rook = PositionBuilder::new()
            .add_piece('e', 3)
            .build();

        let blocked = PositionBuilder::new()
            .add_piece('b', 3)
            .build();

        let blocked_idx = bit_pos('b',3).unwrap();

        let ray = PositionBuilder::new()
            .add_piece('d', 3)
            .add_piece('c', 3)
            .add_piece('b', 3)
            .add_piece('a', 3)
            .build();

        render_bitboard(&rook, 'R');
        render_bitboard(&blocked, 'B');
        render_bitboard(&ray, 'x');

        let blocked_ray = u64::MAX << blocked_idx + 1;
        println!("block ray");
        render_bitboard(&blocked_ray, '*');
        println!("inverted block ray");
        render_bitboard(&!blocked_ray, '~');
        println!("AVAILABLE");
        render_bitboard(&(ray & blocked_ray), '+');



        // for bottom do ray &  (u64::MAX << blocked_idx) CONFIRM (inclusive block)
        // for right  do ray & !(u64::MAX << blocked_idx + 1) CONFIRM
    }

    #[test]
    fn test_bottom() {
        let rook = PositionBuilder::new()
            .add_piece('e', 4)
            .build();

        let blocked = PositionBuilder::new()
            .add_piece('e', 2)
            .build();

        let blocked_idx = bit_pos('e',2).unwrap();

        let ray = PositionBuilder::new()
            .add_piece('e', 3)
            .add_piece('e', 2)
            .add_piece('e', 1)
            .build();

        render_bitboard(&rook, 'R');
        render_bitboard(&blocked, 'B');
        render_bitboard(&ray, 'x');

        let blocked_ray = u64::MAX << blocked_idx + 1;
        println!("block ray");
        render_bitboard(&blocked_ray, '*');
        println!("inverted block ray");
        render_bitboard(&!blocked_ray, '~');
        println!("AVAILABLE");
        render_bitboard(&(ray & blocked_ray), '+');

        // for right  do ray & !(u64::MAX << blocked_idx + 1) CONFIRM
    }


    #[test]
    fn test_right() {
        let rook = PositionBuilder::new()
            .add_piece('e', 4)
            .build();

        let blocked = PositionBuilder::new()
            .add_piece('g', 4)
            .build();

        let blocked_idx = bit_pos('g',4).unwrap();

        let ray = PositionBuilder::new()
            .add_piece('f', 4)
            .add_piece('g', 4)
            .add_piece('h', 4)
            .build();

        render_bitboard(&rook, 'R');
        render_bitboard(&blocked, 'B');
        render_bitboard(&ray, 'x');

        let blocked_ray = u64::MAX << blocked_idx;
        println!("block ray");
        render_bitboard(&blocked_ray, '*');
        println!("inverted block ray");
        render_bitboard(&!blocked_ray, '~');
        println!("AVAILABLE");
        render_bitboard(&(ray & !blocked_ray), '+');

    }

    #[test]
    fn test_precompute_bishop_rays() {
        let rays = precompute_bishop_rays(bit_pos('e', 4).unwrap() as u8);
        render_bitboard(&rays[0], '⬈');
        render_bitboard(&rays[1], '⬊');
        render_bitboard(&rays[2], '⬋');
        render_bitboard(&rays[3], '⬉');

        let expected_top_right_moves = PositionBuilder::new()
            .add_piece('f', 5)
            .add_piece('g', 6)
            .add_piece('h', 7)
            .build();
        let expected_bottom_right_moves = PositionBuilder::new()
            .add_piece('f', 3)
            .add_piece('g', 2)
            .add_piece('h', 1)
            .build();
        let expected_bottom_left_moves = PositionBuilder::new()
            .add_piece('d', 3)
            .add_piece('c', 2)
            .add_piece('b', 1)
            .build();
        let expected_top_left_moves = PositionBuilder::new()
            .add_piece('d', 5)
            .add_piece('c', 6)
            .add_piece('b', 7)
            .add_piece('a', 8)
            .build();

        assert_eq!([expected_top_right_moves, expected_bottom_right_moves, expected_bottom_left_moves, expected_top_left_moves], precompute_bishop_rays(bit_pos('e', 4).unwrap() as u8));

    }
}

// for top    do ray & !(u64::MAX << blocked_idx) CONFIRM (exclusive)
// for left   do ray &  (u64::MAX << blocked_idx + 1) CONFIRM (exclusive)
// for bottom do ray &  (u64::MAX << blocked_idx + 1) CONFIRM (exclusive)
// for right  do ray & !(u64::MAX << blocked_idx) CONFIRM (exclusive)
