use crate::board::{bitboard_single, is_rank, render_bitboard, Board, MASK_FILE_A, MASK_FILE_B, MASK_FILE_G, MASK_FILE_H, MASK_RANK_2, MASK_RANK_7};
use crate::parser::ParsedMove;
use crate::precompute_moves;
// move generation related, only generate pseudo-legal moves

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
    let moves: u64;

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

pub const KNIGHT_MOVES: [u64; 64] = precompute_moves!(precompute_knight_moves);

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

pub const ROOK_RAYS: [[u64; 4]; 64] = precompute_moves!(4, precompute_rook_rays);
// clockwise direction
const fn precompute_rook_rays(index: u8) -> [u64; 4] {
    let mut top: u64 = 0;
    let mut right: u64 = 0;
    let mut bottom: u64 = 0;
    let mut left: u64 = 0;

    let file = index % 8;
    let rank = index / 8;
    // println!("file={} rank={}", file, rank);

    let mut r: u8;
    let mut f: u8;

    r = rank + 1;
    while r < 8 {
        top |= 1u64 << (r * 8 + file);
        r += 1;
    }

    f = file + 1;
    while f < 8 {
        right |= 1u64 << (rank * 8 + f);
        f += 1;
    }

    r = 0;
    while r < rank {
        bottom |= 1u64 << (r * 8 + file);
        r += 1;
    }

    f = 0;
    while f < file {
        left |= 1u64 << (rank * 8 + f);
        f += 1;
    }

    [top, right, bottom, left]
}

pub fn compute_rooks_moves(board: &Board, is_white: bool) -> u64 {
    let mut moves = 0u64;
    let own_pieces: u64;
    let mut rooks: u64;
    let occupied = board.occupied;
    if is_white {
        rooks = board.white_rooks;
        own_pieces = board.white_pieces;
    } else {
        rooks = board.black_rooks;
        own_pieces = board.black_pieces;
    };

    while rooks != 0 {
        let index = rooks.trailing_zeros();
        // let rays = precompute_rook_rays(index as u8);
        let rays = ROOK_RAYS[index as usize];

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

pub const BISHOP_RAYS: [[u64; 4]; 64] = precompute_moves!(4, precompute_bishop_rays);
const fn precompute_bishop_rays(index: u8) -> [u64; 4] {
    let mut top_right: u64 = 0;
    let mut bottom_right: u64 = 0;
    let mut bottom_left: u64 = 0;
    let mut top_left: u64 = 0;

    let file = index % 8;
    let rank = index / 8;
    // println!("file={} rank={}", file, rank);

    let mut f: u8;
    let mut r: u8;

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

pub fn compute_bishops_moves(board: &Board, is_white: bool) -> u64 {
    let mut moves = 0u64;
    let own_pieces: u64;
    let mut bishops: u64;
    let occupied = board.occupied;
    if is_white {
        bishops = board.white_bishops;
        own_pieces = board.white_pieces;
    } else {
        bishops = board.black_bishops;
        own_pieces = board.black_pieces;
    };

    while bishops != 0 {
        let index = bishops.trailing_zeros();
        let rays = BISHOP_RAYS[index as usize];

        for dir in 0..4 {
            let ray = rays[dir];
            let blockers: u64 = ray & occupied;
            if blockers == 0 {
                // not occupied, add the whole move
                moves |= ray
            } else {
                let blocked_idx: u32;
                let blocked_bit: u64;

                if dir == 0 || dir == 3 {
                    // top_right and top_left ray, we find the index using trailing zeros position
                    blocked_idx = blockers.trailing_zeros();
                } else {
                    // for bottom_right and bottom_left, we find the index using leading ones position
                    // 63 minus X is required because we are shifting to the left
                    blocked_idx = 63 - blockers.leading_zeros();
                }

                blocked_bit = 1 << blocked_idx;
                if blocked_bit & own_pieces == 0 {
                    // opponent piece, we can move here
                    moves |= blocked_bit;
                }

                // for top_right      do ray & !(u64::MAX << blocked_idx) CONFIRM (exclusive)
                // for bottom_right   do ray & (u64::MAX << blocked_idx + 1) CONFIRM (exclusive)
                // for bottom_left    do ray & (u64::MAX << blocked_idx + 1)
                // for top_left       do ray & !(u64::MAX << blocked_idx) CONFIRM (exclusive)
                if dir == 0 || dir == 3 {
                    // top_right and top_left
                    // moves |= ray & !blocked_bitboard;
                    moves |= ray & !(u64::MAX << blocked_idx);
                } else {
                    // bottom_right and bottom_left
                    moves |= ray & (u64::MAX << blocked_idx + 1);
                }
            }
        }

        // Remove the processed rooks (use lsb approach)
        bishops &= bishops - 1;
    }

    moves
}

pub const QUEEN_RAYS: [[u64; 8]; 64] = precompute_moves!(8, precompute_queen_rays);
// clockwise direction
const fn precompute_queen_rays(index: u8) -> [u64; 8] {
    let rook_rays = ROOK_RAYS[index as usize];
    let bishop_rays = BISHOP_RAYS[index as usize];
    let mut rays: [u64; 8] = [0; 8];
    let mut i: usize = 0;
    while i < 4 {
        rays[i * 2] = rook_rays[i];
        rays[i * 2 + 1] = bishop_rays[i];
        i += 1;
    }
    rays
}

pub fn compute_queens_moves(board: &Board, is_white: bool) -> u64 {
    let mut moves = 0u64;
    let own_pieces: u64;
    let mut queens: u64;
    let occupied = board.occupied;
    if is_white {
        queens = board.white_queens;
        own_pieces = board.white_pieces;
    } else {
        queens = board.black_queens;
        own_pieces = board.black_pieces;
    };

    while queens != 0 {
        let index = queens.trailing_zeros();
        let rays = QUEEN_RAYS[index as usize];

        for dir in 0..8 {
            let ray = rays[dir];
            let blockers: u64 = ray & occupied;
            if blockers == 0 {
                // not occupied, add the whole move
                moves |= ray
            } else {
                let blocked_idx: u32;
                let blocked_bit: u64;

                // combination of rooks and bishops check
                if dir == 0 || dir == 1 || dir == 2 || dir == 7 {
                    // top, top_right, right, top_left, we find the index using trailing zeros position
                    blocked_idx = blockers.trailing_zeros();
                } else {
                    // bottom_right, bottom, bottom_left, left, we find the index using leading ones
                    // position
                    // 63 minus X is required because we are shifting to the left
                    blocked_idx = 63 - blockers.leading_zeros();
                }

                blocked_bit = 1 << blocked_idx;
                if blocked_bit & own_pieces == 0 {
                    // opponent piece, we can move here
                    moves |= blocked_bit;
                }

                if dir == 0 || dir == 1 || dir == 2 || dir == 7 {
                    moves |= ray & !(u64::MAX << blocked_idx);
                } else {
                    moves |= ray & (u64::MAX << blocked_idx + 1);
                }
            }
        }

        // Remove the processed rooks (use lsb approach)
        queens &= queens - 1;
    }

    moves
}

pub fn compute_king_moves(board: &Board, is_white: bool) -> u64 {
    let mut moves = 0u64;
    let own_pieces: u64;
    let mut king: u64;
    let occupied = board.occupied;
    if is_white {
        king = board.white_king;
        own_pieces = board.white_pieces;
    } else {
        king = board.black_king;
        own_pieces = board.black_pieces;
    };

    let index = king.trailing_zeros();
    let file = index % 8;
    let rank = index / 8;

    // iterate over ranks [-1, 0, 1] relative to the current rank
    for dr in -1..=1 {
        let r = rank as i8 + dr;
        // skip out of bounds
        if r < 0 || r > 7 {
            continue;
        }

        // iterate over files [-1, 0, 1] relative to current file
        for df in -1..=1 {
            let f = file as i8 + df;

            // skip out of bounds only
            if f < 0 || f > 7 {
                continue;
            }

            moves |= 1u64 << (r as u8 * 8 + f as u8);
        }
    }

    // find the current own_pieces that blocked by the king moves
    let own_pieces_blocked_moves = moves & own_pieces;
    // xor with own piece to remove blocked pieces for kings moves
    moves ^= own_pieces_blocked_moves;
    moves
}

// pawn source will always be resolvable
pub fn resolve_pawn_source(board: &Board,
                           parsed_move: ParsedMove,
                           is_white: bool) -> u64 {

    let target_rank: u64 = (parsed_move.to.trailing_zeros() / 8) as u64;

    // determine from
    if is_white {
        if parsed_move.is_capture {
            // find the target rank, move 1 step backward
            let rank = target_rank - 1;
            bitboard_single(parsed_move.from_file.unwrap(), rank).unwrap() & board.white_pawns
        } else {
            // figure out from either 1 step or 2 steps backwards
            parsed_move.to >> 8 & board.white_pawns | parsed_move.to >> 16 & board.white_pawns
        }
    } else {
        if parsed_move.is_capture {
            // find the target rank, move 1 step backward
            let rank = target_rank + 1;
            bitboard_single(parsed_move.from_file.unwrap(), rank).unwrap() & board.black_pawns
        } else {
            // figure out from either 1 step or 2 steps backwards
            parsed_move.to << 8 & board.black_pawns | parsed_move.to << 16 & board.black_pawns
        }
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::board::{bit_pos, bitboard_single, render_bitboard, Board, PositionBuilder};

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
            .add_piece('h', 8)
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
            .add_piece('c', 3) // can capture c3
            .add_piece('d', 2)
            .add_piece('f', 2)
            .add_piece('g', 3)
            .add_piece('g', 5)
            // for h8 knight
            .add_piece('f', 7)
            .build();

        assert_eq!(
            expected_white_knights_moves,
            compute_knights_moves(&board, true)
        );
        assert_eq!(
            expected_black_knights_moves,
            compute_knights_moves(&board, false)
        );
    }

    #[test]
    fn test_precompute_rook_rays() {
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

        assert_eq!(
            [
                expected_top_moves,
                expected_right_moves,
                expected_bottom_moves,
                expected_left_moves
            ],
            precompute_rook_rays(bit_pos('e', 4).unwrap() as u8)
        );
    }

    #[test]
    fn test_compute_rook_moves() {
        let white_pawns = PositionBuilder::new()
            .add_piece('a', 2)
            .add_piece('b', 4)
            .add_piece('e', 2)
            .build();
        let white_knights = PositionBuilder::new().add_piece('e', 1).build();

        let white_rooks = PositionBuilder::new()
            .add_piece('a', 1)
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

        assert_eq!(
            expected_white_rooks_moves,
            compute_rooks_moves(&board, true)
        );
        assert_eq!(
            expected_black_rooks_moves,
            compute_rooks_moves(&board, false)
        );
    }

    #[test]
    fn test_precompute_bishop_rays() {
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

        assert_eq!(
            [
                expected_top_right_moves,
                expected_bottom_right_moves,
                expected_bottom_left_moves,
                expected_top_left_moves
            ],
            precompute_bishop_rays(bit_pos('e', 4).unwrap() as u8)
        );
    }

    #[test]
    fn test_compute_bishops_moves() {
        let white_pawns = PositionBuilder::new()
            .add_piece('a', 2)
            .add_piece('c', 2)
            .add_piece('e', 2)
            .add_piece('g', 2)
            .build();

        let white_knights = PositionBuilder::new().add_piece('c', 6).build();

        let white_bishops = PositionBuilder::new().add_piece('e', 4).build();

        let black_pawns = PositionBuilder::new()
            // .add_piece('e', 5)
            .add_piece('e', 8)
            .add_piece('f', 6)
            .add_piece('g', 6)
            .build();

        let black_bishops = PositionBuilder::new().add_piece('b', 8).build();

        let board = Board::new(
            white_pawns,
            white_knights,
            0,
            white_bishops,
            0,
            0,
            black_pawns,
            0,
            0,
            black_bishops,
            0,
            0,
        );

        let expected_white_bishops_moves = PositionBuilder::new()
            .add_piece('f', 5)
            .add_piece('g', 6) // can capture pawn
            .add_piece('f', 3)
            .add_piece('d', 3)
            .add_piece('d', 5)
            .build();

        let expected_black_bishops_moves = PositionBuilder::new()
            // a8 rook
            .add_piece('c', 7)
            .add_piece('d', 6)
            .add_piece('e', 5)
            .add_piece('f', 4)
            .add_piece('g', 3)
            .add_piece('h', 2)
            .add_piece('a', 7)
            .build();

        assert_eq!(
            expected_white_bishops_moves,
            compute_bishops_moves(&board, true)
        );
        assert_eq!(
            expected_black_bishops_moves,
            compute_bishops_moves(&board, false)
        );
    }

    #[test]
    fn test_precompute_queen_rays() {
        let expected_top_moves = PositionBuilder::new()
            .add_piece('e', 5)
            .add_piece('e', 6)
            .add_piece('e', 7)
            .add_piece('e', 8)
            .build();
        let expected_top_right_moves = PositionBuilder::new()
            .add_piece('f', 5)
            .add_piece('g', 6)
            .add_piece('h', 7)
            .build();
        let expected_right_moves = PositionBuilder::new()
            .add_piece('f', 4)
            .add_piece('g', 4)
            .add_piece('h', 4)
            .build();
        let expected_bottom_right_moves = PositionBuilder::new()
            .add_piece('f', 3)
            .add_piece('g', 2)
            .add_piece('h', 1)
            .build();
        let expected_bottom_moves = PositionBuilder::new()
            .add_piece('e', 3)
            .add_piece('e', 2)
            .add_piece('e', 1)
            .build();
        let expected_bottom_left_moves = PositionBuilder::new()
            .add_piece('d', 3)
            .add_piece('c', 2)
            .add_piece('b', 1)
            .build();
        let expected_left_moves = PositionBuilder::new()
            .add_piece('d', 4)
            .add_piece('c', 4)
            .add_piece('b', 4)
            .add_piece('a', 4)
            .build();
        let expected_top_left_moves = PositionBuilder::new()
            .add_piece('d', 5)
            .add_piece('c', 6)
            .add_piece('b', 7)
            .add_piece('a', 8)
            .build();

        assert_eq!(
            [
                expected_top_moves,
                expected_top_right_moves,
                expected_right_moves,
                expected_bottom_right_moves,
                expected_bottom_moves,
                expected_bottom_left_moves,
                expected_left_moves,
                expected_top_left_moves
            ],
            precompute_queen_rays(bit_pos('e', 4).unwrap() as u8)
        );
    }

    #[test]
    fn test_compute_queen_moves() {
        let white_pawns = PositionBuilder::new()
            .add_piece('a', 2)
            .add_piece('b', 4)
            .add_piece('e', 2)
            .build();
        let white_knights = PositionBuilder::new().add_piece('e', 1).build();

        let white_rooks = PositionBuilder::new().add_piece('a', 1).build();

        let white_queens = PositionBuilder::new().add_piece('e', 4).build();

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

        let black_queens = PositionBuilder::new().add_piece('h', 5).build();

        let board = Board::new(
            white_pawns,
            white_knights,
            white_rooks,
            0,
            white_queens,
            0,
            black_pawns,
            0,
            black_rooks,
            0,
            black_queens,
            0,
        );

        let expected_white_queen_moves = PositionBuilder::new()
            // top
            .add_piece('e', 5)
            .add_piece('e', 6)
            .add_piece('e', 7)
            .add_piece('e', 8)
            // top right
            .add_piece('f', 5)
            .add_piece('g', 6)
            // right
            .add_piece('f', 4)
            .add_piece('g', 4)
            .add_piece('h', 4)
            // bottom right
            .add_piece('f', 3)
            .add_piece('g', 2)
            .add_piece('h', 1)
            // bottom
            .add_piece('e', 3)
            // bottom left
            .add_piece('d', 3)
            .add_piece('c', 2)
            .add_piece('b', 1)
            // left
            .add_piece('d', 4)
            .add_piece('c', 4)
            // top left
            .add_piece('d', 5)
            .add_piece('c', 6)
            .add_piece('b', 7)
            .add_piece('a', 8)
            .build();

        let expected_black_queen_moves = PositionBuilder::new()
            // top
            .add_piece('h', 6)
            .add_piece('h', 7)
            // top right
            // right
            // bottom right
            // bottom
            .add_piece('h', 4)
            .add_piece('h', 3)
            .add_piece('h', 2)
            .add_piece('h', 1)
            // bottom left
            .add_piece('g', 4)
            .add_piece('f', 3)
            .add_piece('e', 2)
            // left
            .add_piece('g', 5)
            .add_piece('f', 5)
            .add_piece('e', 5)
            .add_piece('d', 5)
            .add_piece('c', 5)
            .add_piece('b', 5)
            .add_piece('a', 5)
            // top left
            .build();

        assert_eq!(
            expected_white_queen_moves,
            compute_queens_moves(&board, true)
        );
        assert_eq!(
            expected_black_queen_moves,
            compute_queens_moves(&board, false)
        );
    }

    #[test]
    fn test_compute_king_moves() {
        let white_pawns = PositionBuilder::new()
            .add_piece('a', 2)
            .add_piece('b', 4)
            .add_piece('e', 2)
            .build();
        let white_knights = PositionBuilder::new().add_piece('e', 1).build();

        let white_rooks = PositionBuilder::new().add_piece('a', 1).build();

        let white_queens = PositionBuilder::new().add_piece('e', 4).build();

        let white_king = PositionBuilder::new().add_piece('f', 5).build();

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

        let black_queens = PositionBuilder::new().add_piece('h', 5).build();
        let black_king = PositionBuilder::new().add_piece('h', 6).build();

        let board = Board::new(
            white_pawns,
            white_knights,
            white_rooks,
            0,
            white_queens,
            white_king,
            black_pawns,
            0,
            black_rooks,
            0,
            black_queens,
            black_king,
        );

        let expected_white_king_moves = PositionBuilder::new()
            .add_piece('f', 6)
            .add_piece('g', 6)
            .add_piece('g', 5)
            .add_piece('g', 4)
            .add_piece('f', 4)
            // e4 blocked by queen
            .add_piece('e', 5)
            .add_piece('e', 6)
            .build();

        let expected_black_king_moves = PositionBuilder::new()
            .add_piece('h', 7)
            // h5 and g6 blocked
            .add_piece('g', 5)
            .add_piece('g', 7)
            .build();

        assert_eq!(expected_white_king_moves, compute_king_moves(&board, true));
        assert_eq!(expected_black_king_moves, compute_king_moves(&board, false));
    }
}
