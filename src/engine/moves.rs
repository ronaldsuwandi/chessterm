use crate::engine::board::{
    bitboard_single, is_file, is_rank, Board, MASK_FILE_A, MASK_FILE_B, MASK_FILE_G, MASK_FILE_H,
    MASK_RANK_2, MASK_RANK_7,
};
use crate::engine::parser::ParsedMove;
use crate::precompute_moves;
/// move generation related, only generate pseudo-legal moves which ensure that
/// moves are within bounds, exclude friendly pieces and exclude blocked pieces

pub const UP: usize = 0;
pub const UP_RIGHT: usize = 1;
pub const RIGHT: usize = 2;
pub const DOWN_RIGHT: usize = 3;
pub const DOWN: usize = 4;
pub const DOWN_LEFT: usize = 5;
pub const LEFT: usize = 6;
pub const UP_LEFT: usize = 7;

pub const WHITE_PAWN_MOVES: [[u64; 2]; 64] = precompute_moves!(2, true, precompute_pawn_moves);
pub const BLACK_PAWN_MOVES: [[u64; 2]; 64] = precompute_moves!(2, false, precompute_pawn_moves);
const fn precompute_pawn_moves(index: u8, is_white: bool) -> [u64; 2] {
    let bitboard = 1u64 << index;

    // no valid move at first line
    if (is_white && index < 8) || (!is_white && index > 55) {
        return [0, 0];
    }

    let single_move: u64;
    let double_move: u64;
    let left_diagonal: u64;
    let right_diagonal: u64;

    if is_white {
        single_move = bitboard << 8;
        left_diagonal = bitboard << 7 & !MASK_FILE_H; // prevent wrap-around on H file for left diagonal move
        right_diagonal = bitboard << 9 & !MASK_FILE_A; // prevent wrap-around on A file on right diagonal move
    } else {
        single_move = bitboard >> 8;
        left_diagonal = bitboard >> 9 & !MASK_FILE_H; // prevent wrap-around on H file for left diagonal move
        right_diagonal = bitboard >> 7 & !MASK_FILE_A; // prevent wrap-around on A file on right diagonal move
    }

    // double moves only on rank 2 for white and rank 7 for black
    if is_white && index >= 8 && index <= 15 {
        double_move = bitboard << 16;
    } else if !is_white && index >= 48 && index <= 55 {
        double_move = bitboard >> 16;
    } else {
        double_move = 0;
    }

    let attacks = left_diagonal | right_diagonal;
    let moves = single_move | double_move | attacks;
    [moves, attacks]
}

// PAWNS
pub fn compute_pawns_moves(board: &Board, is_white: bool) -> (u64, u64) {
    let mut moves = 0u64;
    let mut attack_moves = 0u64;
    let own_pieces: u64;
    let mut pawns: u64;
    let precomputed_moves: [[u64; 2]; 64];

    if is_white {
        pawns = board.white_pawns;
        own_pieces = board.white_pieces;
        precomputed_moves = WHITE_PAWN_MOVES;
    } else {
        pawns = board.black_pawns;
        own_pieces = board.black_pieces;
        precomputed_moves = BLACK_PAWN_MOVES;
    };

    while pawns != 0 {
        let index = pawns.trailing_zeros() as usize;

        // add pawn's precomputed moves and exclude own piece
        moves |= precomputed_moves[index][0] & !own_pieces;
        attack_moves |= precomputed_moves[index][1] & !own_pieces;

        // additional check for double move only for rank 2 for white
        if is_white && index >= 8 && index <= 15 {
            // Check if both rank 3 and rank 4 squares are free
            let rank3_free = (1u64 << (index + 8)) & board.free;
            let rank4_free = (1u64 << (index + 16)) & board.free;
            if rank3_free == 0 {
                // if rank3 is blocked, remove rank 3 and rank 4
                moves &= !(1u64 << (index + 8));
                moves &= !(1u64 << (index + 16));
            } else if rank4_free == 0 {
                // if only rank 4 is blocked, remove rank 4
                moves &= !(1u64 << (index + 16));
            }
        } else if !is_white && index >= 48 && index <= 55 {
            // Check if both rank 6 and rank 5 squares are free
            let rank6_free = (1u64 << (index - 8)) & board.free;
            let rank5_free = (1u64 << (index - 16)) & board.free;
            if rank6_free == 0 {
                // if rank 6 is blocked, remove both rank 6 and 5
                moves &= !(1u64 << (index - 16));
                moves &= !(1u64 << (index - 8));
            } else if rank5_free == 0 {
                // If rank 5 is blocked, remove only the rank 5 move from precomputed moves
                moves &= !(1u64 << (index - 16));
            }
        }

        // Remove the processed pawns (use lsb approach)
        pawns &= pawns - 1;
    }

    (moves, attack_moves)
}

pub const KNIGHT_MOVES: [u64; 64] = precompute_moves!(precompute_knight_moves);
// precompute all the moves available for knights at each bit index in the bitboard
const fn precompute_knight_moves(index: u8) -> u64 {
    let bitboard = 1u64 << index;
    // use mask to avoid wrap around
    ((bitboard << 17) & !MASK_FILE_A) // UP 2 + RIGHT 1
        | ((bitboard << 15) & !MASK_FILE_H) // UP 2 + LEFT 1
        | ((bitboard << 10) & !(MASK_FILE_A | MASK_FILE_B)) // UP 1 + RIGHT 2
        | ((bitboard << 6) & !(MASK_FILE_G | MASK_FILE_H)) // UP 1 + LEFT 2
        | ((bitboard >> 17) & !MASK_FILE_H) // DOWN 2 + LEFT 1
        | ((bitboard >> 15) & !MASK_FILE_A) // DOWN 2 + RIGHT 1
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

/// Finds the blocker along the given ray for a given direction.
/// Once a blocker is found, all the remaining move for the ray is marked
/// as blocked and returns the tuple of first blocker and blocker mask.
/// Returns (0, 0) if no blocking found
/// Important: caller is responsible to pass the correct ray and direction
pub fn find_blocker_mask(ray: u64, occupied: u64, direction: usize) -> (u64, u64) {
    let blockers = ray & occupied;
    if blockers == 0 {
        (0, 0)
    } else {
        let blocker_idx;
        let available_moves;
        if matches!(direction, UP | UP_RIGHT | RIGHT | UP_LEFT) {
            blocker_idx = blockers.trailing_zeros();
            available_moves = ray & !(u64::MAX << blocker_idx);
        } else {
            // for directions down, left or down-left/down-right
            // 63 minus X is required because we are shifting to the left
            blocker_idx = 63 - blockers.leading_zeros();
            available_moves = ray & (u64::MAX << (blocker_idx + 1))
        };

        let blocker_pos = 1 << blocker_idx;

        // XOR with ray to get the blocked mask
        (blocker_pos, ray ^ available_moves)
    }
}

pub const ROOK_RAYS_DIRECTIONS: [usize; 4] = [UP, RIGHT, DOWN, LEFT];
pub const BISHOP_RAYS_DIRECTIONS: [usize; 4] = [UP_RIGHT, DOWN_RIGHT, DOWN_LEFT, UP_LEFT];
pub const QUEEN_RAYS_DIRECTIONS: [usize; 8] = [
    UP, UP_RIGHT, RIGHT, DOWN_RIGHT, DOWN, DOWN_LEFT, LEFT, UP_LEFT,
];

pub const ROOK_RAYS: [[u64; 4]; 64] = precompute_moves!(4, precompute_rook_rays);
pub const BISHOP_RAYS: [[u64; 4]; 64] = precompute_moves!(4, precompute_bishop_rays);
pub const QUEEN_RAYS: [[u64; 8]; 64] = precompute_moves!(8, precompute_queen_rays);

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

fn compute_sliding_moves(
    mut pieces: u64,
    directions: &[usize],
    own_pieces: u64,
    occupied: u64,
) -> u64 {
    let mut moves = 0u64;

    while pieces != 0 {
        let index = pieces.trailing_zeros();
        let rays = QUEEN_RAYS[index as usize];

        for &dir in directions {
            let ray = rays[dir];

            let (blocked_bit, blocked_mask) = find_blocker_mask(ray, occupied, dir);
            // ray & inverted block mask to show the available move in the ray
            moves |= ray & !blocked_mask;

            // if first blocked piece is an opponent, we can move here
            if blocked_bit & own_pieces == 0 {
                moves |= blocked_bit;
            }
        }

        // Remove the processed piece (use lsb approach)
        pieces &= pieces - 1;
    }
    moves
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

    compute_sliding_moves(rooks, &ROOK_RAYS_DIRECTIONS, own_pieces, occupied)
}

const fn precompute_bishop_rays(index: u8) -> [u64; 4] {
    let mut top_right: u64 = 0;
    let mut bottom_right: u64 = 0;
    let mut bottom_left: u64 = 0;
    let mut top_left: u64 = 0;

    let file = index % 8;
    let rank = index / 8;

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

    compute_sliding_moves(bishops, &BISHOP_RAYS_DIRECTIONS, own_pieces, occupied)
}

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

    compute_sliding_moves(queens, &QUEEN_RAYS_DIRECTIONS, own_pieces, occupied)
}

pub const KING_MOVES: [u64; 64] = precompute_moves!(precompute_king_moves);
// precompute all the moves available for knights at each bit index in the bitboard
const fn precompute_king_moves(index: u8) -> u64 {
    let bitboard = 1u64 << index;
    // use mask to avoid wrap around
    ((bitboard << 8))                       // up
        | ((bitboard >> 8))                     // down
        | ((bitboard << 1) & !MASK_FILE_A)      // right
        | ((bitboard >> 1) & !MASK_FILE_H)      // left
        | ((bitboard << 9) & !MASK_FILE_A)      // up-right
        | ((bitboard << 7) & !MASK_FILE_H)      // up-left
        | ((bitboard >> 9) & !MASK_FILE_H)      // down-left
        | ((bitboard >> 7) & !MASK_FILE_A) // down-right
}

pub fn compute_king_moves(board: &Board, is_white: bool) -> u64 {
    let mut moves = 0u64;
    let own_pieces: u64;
    let king: u64;
    if is_white {
        king = board.white_king;
        own_pieces = board.white_pieces;
    } else {
        king = board.black_king;
        own_pieces = board.black_pieces;
    };

    let index = king.trailing_zeros();
    // Add the king's precomputed moves, excluding occupied by own
    moves |= KING_MOVES[index as usize] & !own_pieces;

    moves
}

// pawn source will always be resolvable
pub fn resolve_pawn_source(board: &Board, parsed_move: &ParsedMove, is_white: bool) -> u64 {
    let target_rank: u64 = (parsed_move.to.trailing_zeros() / 8) as u64 + 1;
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

// returns bitboard of the potential source, this means there may be more than 1 knight
pub fn resolve_knight_source(board: &Board, parsed_move: &ParsedMove, is_white: bool) -> u64 {
    let mut knights = if is_white {
        board.white_knights
    } else {
        board.black_knights
    };

    let mut source = 0;
    while knights > 0 {
        // get current knight position using LSB in bitboard
        let knight_position = knights & !(knights - 1);
        let knight_idx = knight_position.trailing_zeros() as usize;

        // remove the knight from the bitboard
        knights ^= knight_position;

        if KNIGHT_MOVES[knight_idx] & parsed_move.to != 0 {
            if let Some(from_file) = parsed_move.from_file {
                if !is_file(knight_position, from_file) {
                    continue;
                }
            }
            if let Some(from_rank) = parsed_move.from_rank {
                if !is_rank(knight_position, from_rank) {
                    continue;
                }
            }
            source |= knight_position;
        }
    }
    source
}

// there is only 1 king for each side, it will always be resolvable
pub fn resolve_king_source(board: &Board, _: &ParsedMove, is_white: bool) -> u64 {
    if is_white {
        board.white_king
    } else {
        board.black_king
    }
}

// Rays trait used to generalise resolve_sliding_piece
pub trait Rays {
    // should return slice of bitboard for target rays
    fn get_rays(&self, piece_index: usize) -> &[u64];
}

// implement for bishop/rook
impl Rays for [[u64; 4]; 64] {
    fn get_rays(&self, piece_index: usize) -> &[u64] {
        &self[piece_index]
    }
}

// implement for queen
impl Rays for [[u64; 8]; 64] {
    fn get_rays(&self, piece_index: usize) -> &[u64] {
        &self[piece_index]
    }
}

// helper function to get piece source using rays, no validation done here
fn resolve_sliding_piece_source(
    board: &Board,
    mut pieces: u64,
    parsed_move: &ParsedMove,
    directions: &[usize],
) -> u64 {
    let mut source = 0;
    while pieces > 0 {
        // get current piece position using LSB in bitboard
        let piece_position = pieces & !(pieces - 1);
        let piece_idx = piece_position.trailing_zeros() as usize;

        // remove the piece from the bitboard use LSB
        pieces = pieces & pieces - 1;

        let rays = QUEEN_RAYS[piece_idx];

        // go through all ray direction
        for &dir in directions {
            let ray = rays[dir];
            if ray & parsed_move.to != 0 {
                if let Some(from_file) = parsed_move.from_file {
                    if !is_file(piece_position, from_file) {
                        continue;
                    }
                }
                if let Some(from_rank) = parsed_move.from_rank {
                    if !is_rank(piece_position, from_rank) {
                        continue;
                    }
                }

                // ray is found, ensure that the path from source to target is not blocked

                // if target box is in the occupied board (other piece), exclude them
                let occupied_without_target = if board.occupied & parsed_move.to != 0 {
                    board.occupied ^ parsed_move.to
                } else {
                    board.occupied
                };
                let (_, blocker_mask) = find_blocker_mask(ray, occupied_without_target, dir);
                let available_ray_move = ray & !blocker_mask;

                if available_ray_move & parsed_move.to != 0 {
                    // the ray can go to the target
                    // once we found one ray, no point of continuing the
                    source |= piece_position;
                    break;
                }
            }
        }
    }
    source
}

pub fn resolve_bishop_source(board: &Board, parsed_move: &ParsedMove, is_white: bool) -> u64 {
    let bishops = if is_white {
        board.white_bishops
    } else {
        board.black_bishops
    };
    resolve_sliding_piece_source(board, bishops, parsed_move, &BISHOP_RAYS_DIRECTIONS)
}

pub fn resolve_rook_source(board: &Board, parsed_move: &ParsedMove, is_white: bool) -> u64 {
    let rooks = if is_white {
        board.white_rooks
    } else {
        board.black_rooks
    };
    resolve_sliding_piece_source(board, rooks, parsed_move, &ROOK_RAYS_DIRECTIONS)
}

pub fn resolve_queen_source(board: &Board, parsed_move: &ParsedMove, is_white: bool) -> u64 {
    let queens = if is_white {
        board.white_queens
    } else {
        board.black_queens
    };
    resolve_sliding_piece_source(board, queens, parsed_move, &QUEEN_RAYS_DIRECTIONS)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::board::{bit_pos, Board, PositionBuilder};
    use crate::parser::parse_move;

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
            .add_piece('b', 3)
            .add_piece('c', 3)
            .add_piece('d', 3)
            .add_piece('d', 4)
            .add_piece('d', 4)
            .add_piece('e', 4)
            .add_piece('f', 4)
            .add_piece('f', 3)
            .add_piece('g', 3)
            .add_piece('f', 4)
            .build();

        let board = Board::new(
            white_pawns,
            0,
            0,
            0,
            0,
            bitboard_single('e', 1).unwrap(),
            black_pawns,
            0,
            0,
            0,
            0,
            bitboard_single('e', 8).unwrap(),
        );

        assert_eq!(expected, compute_pawns_moves(&board, true).0);
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
            .add_piece('b', 6)
            .add_piece('c', 1)
            .add_piece('d', 1)
            .add_piece('e', 1)
            .add_piece('e', 2)
            .add_piece('f', 2)
            .build();

        let board = Board::new(
            0,
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

        assert_eq!(expected, compute_pawns_moves(&board, false).0);
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
            bitboard_single('e', 1).unwrap(),
            black_pawns,
            black_knights,
            0,
            0,
            0,
            bitboard_single('e', 8).unwrap(),
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
            bitboard_single('e', 1).unwrap(),
            black_pawns,
            0,
            black_rooks,
            0,
            0,
            bitboard_single('e', 8).unwrap(),
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
            bitboard_single('e', 1).unwrap(),
            black_pawns,
            0,
            0,
            black_bishops,
            0,
            bitboard_single('h', 8).unwrap(),
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
            bitboard_single('e', 1).unwrap(),
            black_pawns,
            0,
            black_rooks,
            0,
            black_queens,
            bitboard_single('c', 8).unwrap(),
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
    fn test_precompute_king_moves() {
        let expected_king_normal_d4: u64 = PositionBuilder::new()
            .add_piece('d', 5)
            .add_piece('e', 5)
            .add_piece('e', 4)
            .add_piece('e', 3)
            .add_piece('d', 3)
            .add_piece('c', 3)
            .add_piece('c', 4)
            .add_piece('c', 5)
            .build();

        let expected_king_edge_a2: u64 = PositionBuilder::new()
            .add_piece('a', 3)
            .add_piece('b', 3)
            .add_piece('b', 2)
            .add_piece('b', 1)
            .add_piece('a', 1)
            .build();

        assert_eq!(
            expected_king_normal_d4,
            precompute_king_moves(bit_pos('d', 4).unwrap() as u8)
        );
        assert_eq!(
            expected_king_edge_a2,
            precompute_king_moves(bit_pos('a', 2).unwrap() as u8)
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

    #[test]
    fn test_resolve_pawn_source() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('e', 2)
            .add_piece('e', 3)
            .add_piece('h', 7)
            .build();
        let white_knights: u64 = PositionBuilder::new()
            .add_piece('b', 1)
            .add_piece('g', 1)
            .build();
        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 7)
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

        assert_eq!(
            bitboard_single('e', 2).unwrap(),
            resolve_pawn_source(&board, &parse_move("exd3").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('h', 7).unwrap(),
            resolve_pawn_source(&board, &parse_move("h8").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('h', 7).unwrap(),
            resolve_pawn_source(&board, &parse_move("hxg8").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('a', 7).unwrap(),
            resolve_pawn_source(&board, &parse_move("a5").unwrap(), false)
        );
    }

    #[test]
    fn test_resolve_knight_source() {
        let white_knights: u64 = PositionBuilder::new()
            .add_piece('e', 1)
            .add_piece('g', 1)
            .build();
        let black_knights: u64 = PositionBuilder::new()
            .add_piece('b', 8)
            .add_piece('b', 6)
            .build();
        let board = Board::new(
            0,
            white_knights,
            0,
            0,
            0,
            bitboard_single('a', 1).unwrap(),
            0,
            black_knights,
            0,
            0,
            0,
            bitboard_single('a', 8).unwrap(),
        );

        // non-ambiguous source
        assert_eq!(
            bitboard_single('e', 1).unwrap(),
            resolve_knight_source(&board, &parse_move("Nd3").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('b', 6).unwrap(),
            resolve_knight_source(&board, &parse_move("Na4").unwrap(), false)
        );

        // ambiguous moves
        assert_eq!(
            PositionBuilder::new()
                .add_piece('e', 1)
                .add_piece('g', 1)
                .build(),
            resolve_knight_source(&board, &parse_move("Nf3").unwrap(), true)
        );
        assert_eq!(
            PositionBuilder::new()
                .add_piece('b', 8)
                .add_piece('b', 6)
                .build(),
            resolve_knight_source(&board, &parse_move("Nd7").unwrap(), false)
        );

        // ambiguous move with more details
        assert_eq!(
            bitboard_single('e', 1).unwrap(),
            resolve_knight_source(&board, &parse_move("Nef3").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('b', 6).unwrap(),
            resolve_knight_source(&board, &parse_move("N6d7").unwrap(), false)
        );

        // ambiguous move with more details but still ambiguous
        assert_eq!(
            PositionBuilder::new()
                .add_piece('e', 1)
                .add_piece('g', 1)
                .build(),
            resolve_knight_source(&board, &parse_move("N1f3").unwrap(), true)
        );
        assert_eq!(
            PositionBuilder::new()
                .add_piece('b', 8)
                .add_piece('b', 6)
                .build(),
            resolve_knight_source(&board, &parse_move("Nbd7").unwrap(), false)
        );

        let black_knights: u64 = PositionBuilder::new()
            .add_piece('b', 8)
            .add_piece('b', 6)
            .build();
        let board = Board::new(
            0,
            0,
            0,
            0,
            0,
            bitboard_single('h', 1).unwrap(),
            0,
            black_knights,
            0,
            0,
            0,
            bitboard_single('h', 8).unwrap(),
        );

        // no white knight
        assert_eq!(
            0,
            resolve_knight_source(&board, &parse_move("Nf3").unwrap(), true)
        );
    }

    #[test]
    fn test_resolve_sliding_pieces() {
        let board = Board::from_fen("1k6/1b6/1b1RpR2/8/4Q2Q/8/B7/KB5Q");

        // non-ambiguous source
        assert_eq!(
            bitboard_single('b', 1).unwrap(),
            resolve_bishop_source(&board, &parse_move("Bc2").unwrap(), true)
        );

        assert_eq!(
            bitboard_single('b', 6).unwrap(),
            resolve_bishop_source(&board, &parse_move("Bf2").unwrap(), false)
        );

        assert_eq!(
            bitboard_single('d', 6).unwrap(),
            resolve_rook_source(&board, &parse_move("Rd5").unwrap(), true)
        );

        assert_eq!(
            bitboard_single('e', 4).unwrap(),
            resolve_queen_source(&board, &parse_move("Qe5").unwrap(), true)
        );

        // no piece found
        assert_eq!(
            0,
            resolve_bishop_source(&board, &parse_move("Bb2").unwrap(), true)
        );

        // ambiguous moves
        assert_eq!(
            PositionBuilder::new()
                .add_piece('d', 6)
                .add_piece('f', 6)
                .build(),
            resolve_rook_source(&board, &parse_move("Re6").unwrap(), true)
        );
        assert_eq!(
            PositionBuilder::new()
                .add_piece('d', 6)
                .add_piece('f', 6)
                .build(),
            resolve_rook_source(&board, &parse_move("Rxe6").unwrap(), true)
        );
        assert_eq!(
            PositionBuilder::new()
                .add_piece('e', 4)
                .add_piece('h', 4)
                .add_piece('h', 1)
                .build(),
            resolve_queen_source(&board, &parse_move("Qe1").unwrap(), true)
        );

        // ambiguous move with more details
        assert_eq!(
            bitboard_single('d', 6).unwrap(),
            resolve_rook_source(&board, &parse_move("Rde6").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('f', 6).unwrap(),
            resolve_rook_source(&board, &parse_move("Rfe6").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('d', 6).unwrap(),
            resolve_rook_source(&board, &parse_move("Rxde6").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('f', 6).unwrap(),
            resolve_rook_source(&board, &parse_move("Rfe6").unwrap(), true)
        );

        assert_eq!(
            bitboard_single('h', 4).unwrap(),
            resolve_queen_source(&board, &parse_move("Qh4e1").unwrap(), true)
        );

        // ambiguous move with more details but still ambiguous
        assert_eq!(
            PositionBuilder::new()
                .add_piece('h', 4)
                .add_piece('h', 1)
                .build(),
            resolve_queen_source(&board, &parse_move("Qhe1").unwrap(), true)
        );
        assert_eq!(
            PositionBuilder::new()
                .add_piece('e', 4)
                .add_piece('h', 4)
                .build(),
            resolve_queen_source(&board, &parse_move("Q4e1").unwrap(), true)
        );
    }

    #[test]
    fn test_resolve_sliding_pieces_path_blocked() {
        let board = Board::from_fen("4k3/8/1q1P2q1/7Q/5QQP/8/8/R3K2R");

        assert_eq!(
            bitboard_single('a', 1).unwrap(),
            resolve_rook_source(&board, &parse_move("Rd1").unwrap(), true)
        );

        // blocked queen by own piece, from resolver standpoint, we are only
        // interested in resolving potential source from the target box
        // regardless if target box is invalid or not, hence the two queens
        // got resolved here
        assert_eq!(
            PositionBuilder::new()
                .add_piece('h', 5)
                .add_piece('g', 4)
                .build(),
            resolve_queen_source(&board, &parse_move("Qh4").unwrap(), true)
        );

        assert_eq!(
            bitboard_single('f', 4).unwrap(),
            resolve_queen_source(&board, &parse_move("Qd4").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('g', 4).unwrap(),
            resolve_queen_source(&board, &parse_move("Qe2").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('b', 6).unwrap(),
            resolve_queen_source(&board, &parse_move("Qc6").unwrap(), false)
        );
    }

    #[test]
    fn test_resolve_king() {
        let board = Board::new(
            0,
            0,
            0,
            0,
            0,
            bitboard_single('d', 5).unwrap(),
            0,
            0,
            0,
            0,
            0,
            bitboard_single('a', 8).unwrap(),
        );

        assert_eq!(
            bitboard_single('d', 5).unwrap(),
            resolve_king_source(&board, &parse_move("Ke1").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('d', 5).unwrap(),
            resolve_king_source(&board, &parse_move("Kxe1").unwrap(), true)
        );
        assert_eq!(
            bitboard_single('a', 8).unwrap(),
            resolve_king_source(&board, &parse_move("Kxe1").unwrap(), false)
        );
    }

    #[test]
    fn test_find_blocker() {
        let board = Board::from_fen("4k3/8/1q1P2q1/7Q/5QQP/8/8/R3K2R");
        let tests = [
            (
                'h',
                1,
                UP,
                vec![('h', 4), ('h', 5), ('h', 6), ('h', 7), ('h', 8)],
            ),
            ('d', 2, UP_RIGHT, vec![('f', 4), ('g', 5), ('h', 6)]),
            ('a', 1, RIGHT, vec![('e', 1), ('f', 1), ('g', 1), ('h', 1)]),
            ('g', 2, DOWN_RIGHT, vec![('h', 1)]),
            (
                'g',
                7,
                DOWN,
                vec![('g', 6), ('g', 5), ('g', 4), ('g', 3), ('g', 2), ('g', 1)],
            ),
            ('c', 7, DOWN_LEFT, vec![('b', 6), ('a', 5)]),
            (
                'f',
                1,
                LEFT,
                vec![('e', 1), ('d', 1), ('c', 1), ('b', 1), ('a', 1)],
            ),
            ('f', 4, UP_LEFT, vec![('d', 6), ('c', 7), ('b', 8)]),
            ('c', 4, UP, vec![]),
            ('c', 4, UP_RIGHT, vec![]),
            ('c', 4, DOWN, vec![]),
            ('c', 4, DOWN_RIGHT, vec![]),
            ('c', 4, DOWN_LEFT, vec![]),
            ('c', 4, LEFT, vec![]),
            ('c', 4, UP_LEFT, vec![]),
            ('g', 2, RIGHT, vec![]),
            ('h', 1, RIGHT, vec![]),
        ];
        for (file, rank, direction, blocked_vec) in tests {
            let idx = bitboard_single(file, rank).unwrap().trailing_zeros() as usize;
            let ray = QUEEN_RAYS[idx][direction];
            let mut expected_mask_builder = PositionBuilder::new();
            for (file, rank) in blocked_vec {
                expected_mask_builder = expected_mask_builder.add_piece(file, rank)
            }
            let expected_mask = expected_mask_builder.build();
            assert_eq!(
                expected_mask,
                find_blocker_mask(ray, board.occupied, direction).1
            );
        }
    }
}
