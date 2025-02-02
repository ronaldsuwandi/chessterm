use crate::engine::moves::{compute_bishops_moves, compute_king_moves, compute_knights_moves, compute_pawns_moves, compute_queens_moves, compute_rooks_moves, WHITE_PAWN_MOVES};
use crate::engine::parser::Piece;

#[derive(Debug, Clone, Copy)]
pub struct Board {
    pub white_pawns: u64,
    pub white_knights: u64,
    pub white_rooks: u64,
    pub white_bishops: u64,
    pub white_queens: u64,
    pub white_king: u64,

    pub black_pawns: u64,
    pub black_knights: u64,
    pub black_rooks: u64,
    pub black_bishops: u64,
    pub black_queens: u64,
    pub black_king: u64,
    pub white_pieces: u64,
    pub black_pieces: u64,
    
    pub occupied: u64,
    pub free: u64,

    pub white_pawns_pseudolegal_moves: u64,
    pub white_knights_pseudolegal_moves: u64,
    pub white_rooks_pseudolegal_moves: u64,
    pub white_bishops_pseudolegal_moves: u64,
    pub white_queens_pseudolegal_moves: u64,
    pub white_king_pseudolegal_moves: u64,

    pub black_pawns_pseudolegal_moves: u64,
    pub black_knights_pseudolegal_moves: u64,
    pub black_rooks_pseudolegal_moves: u64,
    pub black_bishops_pseudolegal_moves: u64,
    pub black_queens_pseudolegal_moves: u64,
    pub black_king_pseudolegal_moves: u64,

    pub white_pawns_attack_moves: u64, // pawn can only attack diagonally
    pub black_pawns_attack_moves: u64, // pawn can only attack diagonally
    pub white_attack_moves: u64,
    pub black_attack_moves: u64,
}

impl Board {
    pub fn from_fen(fen: &str) -> Board {
        let mut white_pawns_builder = PositionBuilder::new();
        let mut white_knights_builder = PositionBuilder::new();
        let mut white_rooks_builder = PositionBuilder::new();
        let mut white_bishops_builder = PositionBuilder::new();
        let mut white_queens_builder = PositionBuilder::new();
        let mut white_king_builder = PositionBuilder::new();
        let mut black_pawns_builder = PositionBuilder::new();
        let mut black_knights_builder = PositionBuilder::new();
        let mut black_rooks_builder = PositionBuilder::new();
        let mut black_bishops_builder = PositionBuilder::new();
        let mut black_queens_builder = PositionBuilder::new();
        let mut black_king_builder = PositionBuilder::new();

        let mut rank = 8;
        let mut file = 'a';

        for c in fen.chars() {
            match c {
                'P' => {
                    white_pawns_builder = white_pawns_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'p' => {
                    black_pawns_builder = black_pawns_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'R' => {
                    white_rooks_builder = white_rooks_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'r' => {
                    black_rooks_builder = black_rooks_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'N' => {
                    white_knights_builder = white_knights_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'n' => {
                    black_knights_builder = black_knights_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'B' => {
                    white_bishops_builder = white_bishops_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'b' => {
                    black_bishops_builder = black_bishops_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'Q' => {
                    white_queens_builder = white_queens_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'q' => {
                    black_queens_builder = black_queens_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'K' => {
                    white_king_builder = white_king_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                'k' => {
                    black_king_builder = black_king_builder.add_piece(file, rank);
                    file = ((file as u8) + 1) as char;
                }
                '/' => {
                    rank -= 1;
                    file = 'a';
                }
                '1'..='8' => {
                    file = ((file as u8) + (c as u8 - '0' as u8)) as char;
                }
                _ => panic!("Invalid FEN character: {}", c),

            }
        }

        Self::new(
            white_pawns_builder.build(),
            white_knights_builder.build(),
            white_rooks_builder.build(),
            white_bishops_builder.build(),
            white_queens_builder.build(),
            white_king_builder.build(),
            black_pawns_builder.build(),
            black_knights_builder.build(),
            black_rooks_builder.build(),
            black_bishops_builder.build(),
            black_queens_builder.build(),
            black_king_builder.build(),
        )
    }

    pub fn new(
        white_pawns: u64,
        white_knights: u64,
        white_rooks: u64,
        white_bishops: u64,
        white_queens: u64,
        white_king: u64,
        black_pawns: u64,
        black_knights: u64,
        black_rooks: u64,
        black_bishops: u64,
        black_queens: u64,
        black_king: u64,
    ) -> Board {
        let white_pieces =
            white_pawns | white_knights | white_rooks | white_bishops | white_queens | white_king;
        let black_pieces =
            black_pawns | black_knights | black_rooks | black_bishops | black_queens | black_king;

        let occupied = white_pieces | black_pieces;
        let free = !occupied;

        let mut board = Board {
            white_pawns,
            white_knights,
            white_rooks,
            white_bishops,
            white_queens,
            white_king,

            black_pawns,
            black_knights,
            black_rooks,
            black_bishops,
            black_queens,
            black_king,

            white_pieces,
            black_pieces,
            occupied,
            free,

            white_pawns_pseudolegal_moves: 0,
            white_knights_pseudolegal_moves: 0,
            white_rooks_pseudolegal_moves: 0,
            white_bishops_pseudolegal_moves: 0,
            white_queens_pseudolegal_moves: 0,
            white_king_pseudolegal_moves: 0,
            black_pawns_pseudolegal_moves: 0,
            black_knights_pseudolegal_moves: 0,
            black_rooks_pseudolegal_moves: 0,
            black_bishops_pseudolegal_moves: 0,
            black_queens_pseudolegal_moves: 0,
            black_king_pseudolegal_moves: 0,

            white_pawns_attack_moves: 0,
            black_pawns_attack_moves: 0,
            white_attack_moves: 0,
            black_attack_moves: 0,
        };

        board.update_compute_moves();
        board
    }

    pub fn update_compute_moves(&mut self) {
        (self.white_pawns_pseudolegal_moves, self.white_pawns_attack_moves) = compute_pawns_moves(&self, true);
        self.white_knights_pseudolegal_moves = compute_knights_moves(&self, true);
        self.white_rooks_pseudolegal_moves = compute_rooks_moves(&self, true);
        self.white_bishops_pseudolegal_moves = compute_bishops_moves(&self, true);
        self.white_queens_pseudolegal_moves = compute_queens_moves(&self, true);
        self.white_king_pseudolegal_moves = compute_king_moves(&self, true);

        (self.black_pawns_pseudolegal_moves, self.black_pawns_attack_moves) = compute_pawns_moves(&self, false);
        self.black_knights_pseudolegal_moves = compute_knights_moves(&self, false);
        self.black_rooks_pseudolegal_moves = compute_rooks_moves(&self, false);
        self.black_bishops_pseudolegal_moves = compute_bishops_moves(&self, false);
        self.black_queens_pseudolegal_moves = compute_queens_moves(&self, false);
        self.black_king_pseudolegal_moves = compute_king_moves(&self, false);

        self.update_attack_moves();
    }

    pub fn update_attack_moves(&mut self) {
        // for attack moves, we do not use pawns pseudolegal moves
        self.white_attack_moves = self.white_pawns_attack_moves
            | self.white_knights_pseudolegal_moves
            | self.white_rooks_pseudolegal_moves
            | self.white_bishops_pseudolegal_moves
            | self.white_queens_pseudolegal_moves
            | self.white_king_pseudolegal_moves;
        self.black_attack_moves = self.black_pawns_attack_moves
            | self.black_knights_pseudolegal_moves
            | self.black_rooks_pseudolegal_moves
            | self.black_bishops_pseudolegal_moves
            | self.black_queens_pseudolegal_moves
            | self.black_king_pseudolegal_moves;
    }

    /// check if the target position on the board is a capture move or not
    pub fn is_capture(&self, target: u64, is_white: bool) -> bool {
        if is_white {
            target & self.black_pieces != 0
        } else {
            target & self.white_pieces != 0
        }
    }

    pub fn update_pieces(&mut self) {
        self.white_pieces = self.white_pawns
            | self.white_knights
            | self.white_rooks
            | self.white_bishops
            | self.white_queens
            | self.white_king;
        self.black_pieces = self.black_pawns
            | self.black_knights
            | self.black_rooks
            | self.black_bishops
            | self.black_queens
            | self.black_king;
        self.occupied = self.white_pieces | self.black_pieces;
        self.free = !self.occupied;
    }

    pub fn get_piece_at(&mut self, position: u64, is_white: bool) -> Option<&mut u64> {
        let pieces: [&mut u64; 6] = if is_white {
            [
                &mut self.white_pawns,
                &mut self.white_knights,
                &mut self.white_rooks,
                &mut self.white_bishops,
                &mut self.white_queens,
                &mut self.white_king,
            ]
        } else {
            [
                &mut self.black_pawns,
                &mut self.black_knights,
                &mut self.black_rooks,
                &mut self.black_bishops,
                &mut self.black_queens,
                &mut self.black_king,
            ]
        };

        for piece in pieces {
            if (*piece & position) != 0 {
                return Some(piece);
            }
        }
        None
    }

    /// moves the given piece without any checking
    pub fn move_piece(&mut self, from: u64, to: u64, is_white: bool) {
        if let Some(piece) = self.get_piece_at(from, is_white) {
            *piece = (*piece ^ from) | to;
            self.update_pieces();
        }
    }

    /// removes piece from the board
    pub fn remove_piece(&mut self, position: u64, is_white: bool) {
        if let Some(piece) = self.get_piece_at(position, is_white) {
            *piece = *piece ^ position;
            self.update_pieces();
        }
    }

    /// used for promotion. only perform promotion if pawn exists at the position
    pub fn replace_pawn(&mut self, position: u64, is_white: bool, new_piece: Piece) {
        let pawns = if is_white {
            self.white_pawns
        } else {
            self.black_pawns
        };

        if pawns & position != 0 {
            match new_piece {
                Piece::Knight => {
                    if is_white {
                        self.white_knights |= position;
                    } else {
                        self.black_knights |= position;
                    }
                }
                Piece::Rook => {
                    if is_white {
                        self.white_rooks |= position;
                    } else {
                        self.black_rooks |= position;
                    }
                }
                Piece::Bishop => {
                    if is_white {
                        self.white_bishops |= position;
                    } else {
                        self.black_bishops |= position;
                    }
                }
                Piece::Queen => {
                    if is_white {
                        self.white_queens |= position;
                    } else {
                        self.black_queens |= position;
                    }
                }
                _ => {
                    return;
                }
            }
            if is_white {
                self.white_pawns ^= position;
            } else {
                self.black_pawns ^= position;
            }
        }
    }

    pub fn pieces_array(&self, unicode: bool) -> [[char; 8]; 8] {
        let mut board_representation = [[' '; 8]; 8];
        // Combine all pieces into a single representation
        for rank in 0..8 {
            for file in 0..8 {
                let square_index = rank * 8 + file;

                let piece = if (self.white_pawns >> square_index) & 1 != 0 {
                    if unicode { '♟' } else { 'P' }
                } else if (self.black_pawns >> square_index) & 1 != 0 {
                    if unicode { '♙' } else { 'p' }
                } else if (self.white_rooks >> square_index) & 1 != 0 {
                    if unicode { '♜' } else { 'R' }
                } else if (self.black_rooks >> square_index) & 1 != 0 {
                    if unicode { '♖' } else { 'r' }
                } else if (self.white_knights >> square_index) & 1 != 0 {
                    if unicode { '♞' } else { 'N' }
                } else if (self.black_knights >> square_index) & 1 != 0 {
                    if unicode { '♘' } else { 'n' }
                } else if (self.white_bishops >> square_index) & 1 != 0 {
                    if unicode { '♝' } else { 'B' }
                } else if (self.black_bishops >> square_index) & 1 != 0 {
                    if unicode { '♗' } else { 'b' }
                } else if (self.white_queens >> square_index) & 1 != 0 {
                    if unicode { '♛' } else { 'Q' }
                } else if (self.black_queens >> square_index) & 1 != 0 {
                    if unicode { '♕' } else { 'q' }
                } else if (self.white_king >> square_index) & 1 != 0 {
                    if unicode { '♚' } else { 'K' }
                } else if (self.black_king >> square_index) & 1 != 0 {
                    if unicode { '♔' } else { 'k' }
                } else {
                    '.' // Empty square
                };

                board_representation[rank as usize][file as usize] = piece;
            }
        }
        board_representation
    }

    // Temporary helper function to render the chess board in terminal
    pub fn render(&self) {


        // Render the board
        println!("  +------------------------+");
        for (rank, row) in self.pieces_array((true)).iter().enumerate().rev() {
            print!("{} |", rank + 1); // Print rank number
            for (file,piece) in row.iter().enumerate() {
                print!(" r={rank},f={file},{} ", piece);
            }
            println!("|");
        }
        println!("  +------------------------+");
        println!("    a  b  c  d  e  f  g  h");
    }
    
    /// Helper function to return the piece type based on position
    /// returns optional piece type and boolean flag to indicate if it's white or black
    pub fn get_piece_type_at(&self, position: u64) -> Option<(Piece, bool)> {
        if self.white_pieces & position != 0 {
            if self.white_pawns & position != 0 {
                Some((Piece::Pawn, true))
            } else if self.white_bishops & position != 0 {
                Some((Piece::Bishop, true))
            } else if self.white_rooks & position != 0 {
                Some((Piece::Rook, true))
            } else if self.white_knights & position != 0 {
                Some((Piece::Knight, true))
            } else if self.white_queens & position != 0 {
                Some((Piece::Queen, true))
            } else if self.white_king & position != 0 {
                Some((Piece::King, true))
            } else {
                None
            }
        } else if self.black_pieces & position != 0 {
            if self.black_pawns & position != 0 {
                Some((Piece::Pawn, false))
            } else if self.black_bishops & position != 0 {
                Some((Piece::Bishop, false))
            } else if self.black_rooks & position != 0 {
                Some((Piece::Rook, false))
            } else if self.black_knights & position != 0 {
                Some((Piece::Knight, false))
            } else if self.black_queens & position != 0 {
                Some((Piece::Queen, false))
            } else if self.black_king & position != 0 {
                Some((Piece::King, false))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for Board {
    fn default() -> Board {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
    }
}

/// Helper function to render single bitboard for debugging
pub fn render_bitboard(bitboard: &u64, render: char) {
    println!("  +------------------------+");
    for rank in (0..8).rev() {
        print!("{} |", rank + 1); // Print rank number
        for file in 0..8 {
            let square_index = rank * 8 + file;
            let bit = (bitboard >> square_index) & 1u64;
            if bit == 1u64 {
                print!(" {} ", render);
            } else {
                print!(" . ");
            }
        }
        println!("|");
    }
    println!("  +------------------------+");
    println!("    a  b  c  d  e  f  g  h");
}

/// Helper function to return the bit index for a given file/rank in a bitboard
pub fn bit_pos(file: char, rank: u64) -> Option<u64> {
    if file < 'a' || file > 'h' || rank < 1 || rank > 8 {
        return None;
    }
    let file_idx = file as u8 - 'a' as u8;
    Some((rank - 1) * 8 + file_idx as u64)
}

/// Helper to create single bit in a bitboard for a given file/rank
pub fn bitboard_single(file: char, rank: u64) -> Option<u64> {
    if let Some(bit_index) = bit_pos(file, rank) {
        Some(1 << bit_index)
    } else {
        None
    }
}

/// Checjk
pub fn is_rank(bitboard: u64, rank: u64) -> bool {
    let mask = match rank {
        1 => MASK_RANK_1,
        2 => MASK_RANK_2,
        3 => MASK_RANK_3,
        4 => MASK_RANK_4,
        5 => MASK_RANK_5,
        6 => MASK_RANK_6,
        7 => MASK_RANK_7,
        8 => MASK_RANK_8,
        _ => 0,
    };

    if mask == 0 {
        false
    } else {
        (bitboard & mask) != 0
    }
}

pub fn is_file(bitboard: u64, file: char) -> bool {
    let mask = match file {
        'a' => MASK_FILE_A,
        'b' => MASK_FILE_B,
        'c' => MASK_FILE_C,
        'd' => MASK_FILE_D,
        'e' => MASK_FILE_E,
        'f' => MASK_FILE_F,
        'g' => MASK_FILE_G,
        'h' => MASK_FILE_H,
        _ => 0,
    };

    if mask == 0 {
        false
    } else {
        (bitboard & mask) != 0
    }
}

/// Helper struct to help putting pieces into bitboard
pub struct PositionBuilder {
    bitboard: u64,
}

impl PositionBuilder {
    pub fn new() -> Self {
        PositionBuilder { bitboard: 0 }
    }

    pub fn add_piece(mut self, file: char, rank: u64) -> Self {
        if let Some(pos) = bit_pos(file, rank) {
            self.bitboard |= 1 << pos;
        }
        self
    }

    pub fn build(self) -> u64 {
        self.bitboard
    }
}

pub const MASK_RANK_1: u64 =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111;
pub const MASK_RANK_2: u64 =
    0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000;
pub const MASK_RANK_3: u64 =
    0b00000000_00000000_00000000_00000000_00000000_11111111_00000000_00000000;
pub const MASK_RANK_4: u64 =
    0b00000000_00000000_00000000_00000000_11111111_00000000_00000000_00000000;
pub const MASK_RANK_5: u64 =
    0b00000000_00000000_00000000_11111111_00000000_00000000_00000000_00000000;
pub const MASK_RANK_6: u64 =
    0b00000000_00000000_11111111_00000000_00000000_00000000_00000000_00000000;
pub const MASK_RANK_7: u64 =
    0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000;
pub const MASK_RANK_8: u64 =
    0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
pub const MASK_FILE_A: u64 =
    0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001;
pub const MASK_FILE_B: u64 =
    0b00000010_00000010_00000010_00000010_00000010_00000010_00000010_00000010;
pub const MASK_FILE_C: u64 =
    0b00000100_00000100_00000100_00000100_00000100_00000100_00000100_00000100;
pub const MASK_FILE_D: u64 =
    0b00001000_00001000_00001000_00001000_00001000_00001000_00001000_00001000;
pub const MASK_FILE_E: u64 =
    0b00010000_00010000_00010000_00010000_00010000_00010000_00010000_00010000;
pub const MASK_FILE_F: u64 =
    0b00100000_00100000_00100000_00100000_00100000_00100000_00100000_00100000;
pub const MASK_FILE_G: u64 =
    0b01000000_01000000_01000000_01000000_01000000_01000000_01000000_01000000;
pub const MASK_FILE_H: u64 =
    0b10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000;

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn test_bit_pos() {
        assert_eq!(bit_pos('a', 1), Some(0));
        assert_eq!(bit_pos('h', 8), Some(63));
        assert_eq!(bit_pos('e', 4), Some(28));
        assert_eq!(bit_pos('b', 2), Some(9));
        assert_eq!(bit_pos('f', 7), Some(53));
        assert_eq!(bit_pos('h', 9), None);
        assert_eq!(bit_pos('h', 0), None);
        assert_eq!(bit_pos('z', 1), None);
    }

    #[test]
    fn test_bitboard_single() {
        assert_eq!(bitboard_single('a', 1), Some(1 << bit_pos('a', 1).unwrap()));
        assert_eq!(bitboard_single('h', 8), Some(1 << bit_pos('h', 8).unwrap()));
        assert_eq!(bitboard_single('e', 4), Some(1 << bit_pos('e', 4).unwrap()));
        assert_eq!(bitboard_single('b', 2), Some(1 << bit_pos('b', 2).unwrap()));
        assert_eq!(bitboard_single('f', 7), Some(1 << bit_pos('f', 7).unwrap()));
        assert_eq!(bitboard_single('h', 9), None);
        assert_eq!(bitboard_single('h', 0), None);
        assert_eq!(bitboard_single('z', 1), None);
    }

    #[test]
    fn test_position_builder() {
        let builder = PositionBuilder::new();
        let actual = builder
            .add_piece('h', 8)
            .add_piece('e', 4)
            .add_piece('a', 1)
            .add_piece('a', 1000) // invalid - ignore
            .build();

        let mut expected: u64 = 0;
        expected = expected | bitboard_single('h', 8).unwrap();
        expected = expected | bitboard_single('e', 4).unwrap();
        expected = expected | bitboard_single('a', 1).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_is_rank() {
        assert!(is_rank(bitboard_single('a', 2).unwrap(), 2));
        assert!(is_rank(bitboard_single('b', 2).unwrap(), 2));
        assert!(is_rank(bitboard_single('c', 2).unwrap(), 2));
        assert!(is_rank(bitboard_single('d', 2).unwrap(), 2));
        assert!(is_rank(bitboard_single('e', 2).unwrap(), 2));
        assert!(is_rank(bitboard_single('f', 2).unwrap(), 2));
        assert!(is_rank(bitboard_single('g', 2).unwrap(), 2));
        assert!(is_rank(bitboard_single('h', 2).unwrap(), 2));
        assert!(is_rank(bitboard_single('b', 1).unwrap(), 1));
        assert!(!is_rank(bitboard_single('a', 5).unwrap(), 2));
        assert!(!is_rank(bitboard_single('b', 1).unwrap(), 2));
    }

    #[test]
    fn test_is_file() {
        assert!(is_file(bitboard_single('b', 1).unwrap(), 'b'));
        assert!(is_file(bitboard_single('b', 2).unwrap(), 'b'));
        assert!(is_file(bitboard_single('b', 3).unwrap(), 'b'));
        assert!(is_file(bitboard_single('b', 4).unwrap(), 'b'));
        assert!(is_file(bitboard_single('b', 5).unwrap(), 'b'));
        assert!(is_file(bitboard_single('b', 6).unwrap(), 'b'));
        assert!(is_file(bitboard_single('b', 7).unwrap(), 'b'));
        assert!(is_file(bitboard_single('b', 8).unwrap(), 'b'));
        assert!(is_file(bitboard_single('a', 1).unwrap(), 'a'));
        assert!(!is_file(bitboard_single('a', 5).unwrap(), 'b'));
        assert!(!is_file(bitboard_single('g', 1).unwrap(), 'f'));
    }

    #[test]
    fn test_from_fen() {
        let board = Board::from_fen("1k5q/p5Pr/pq6/8/8/5NB1/1P6/4K3");
        let pieces = [
            (board.black_king, 'b', 8),
            (board.black_queens, 'h', 8),
            (board.black_queens, 'b', 6),
            (board.black_rooks, 'h', 7),
            (board.black_pawns, 'a', 7),
            (board.black_pawns, 'a', 6),
            (board.white_king, 'e', 1),
            (board.white_knights, 'f', 3),
            (board.white_bishops, 'g', 3),
            (board.white_pawns, 'b', 2),
            (board.white_pawns, 'g', 7),
        ];

        for (piece, file, rank) in pieces {
            assert_ne!(0, piece & bitboard_single(file, rank).unwrap())
        }

        assert_eq!(0, board.black_knights);
        assert_eq!(0, board.black_bishops);
        assert_eq!(0, board.white_rooks);
        assert_eq!(0, board.white_queens);

        // confirm c4 is empty
        assert_eq!(0, board.white_pieces & bitboard_single('c', 4).unwrap());
        assert_eq!(0, board.black_pieces & bitboard_single('c', 4).unwrap());
    }

    #[test]
    fn test_is_capture() {
        let board = Board::from_fen("4k3/7p/6P1/8/8/pp2P1p1/P4P2/4K3");

        assert!(board.is_capture(bitboard_single('b', 3).unwrap(), true));
        assert!(board.is_capture(bitboard_single('g', 3).unwrap(), true));

        assert!(board.is_capture(bitboard_single('a', 2).unwrap(), false));
        assert!(board.is_capture(bitboard_single('g', 6).unwrap(), false));

        assert!(!board.is_capture(bitboard_single('a', 2).unwrap(), true));
        assert!(!board.is_capture(bitboard_single('b', 5).unwrap(), true));
        assert!(!board.is_capture(bitboard_single('h', 6).unwrap(), false));
    }

    #[test]
    fn test_move_piece() {
        let white_pawns = PositionBuilder::new()
            .add_piece('d', 2)
            .add_piece('e', 2)
            .build();
        let black_pawns = PositionBuilder::new().add_piece('f', 7).build();

        let mut board = Board::new(white_pawns, 0, 0, 0, 0, bitboard_single('e', 1).unwrap(), black_pawns, 0, 0, 0, 0, bitboard_single('e', 8).unwrap());
        board.move_piece(
            bitboard_single('e', 2).unwrap(),
            bitboard_single('e', 4).unwrap(),
            true,
        );

        assert_eq!(
            PositionBuilder::new()
                .add_piece('d', 2)
                .add_piece('e', 4)
                .build(),
            board.white_pawns
        );
        assert_eq!(black_pawns, board.black_pawns);

        // no validation for valid/invalid move
        board.move_piece(
            bitboard_single('f', 7).unwrap(),
            bitboard_single('g', 1).unwrap(),
            false,
        );
        assert_eq!(
            PositionBuilder::new().add_piece('g', 1).build(),
            board.black_pawns
        );
        assert_eq!(
            PositionBuilder::new()
                .add_piece('d', 2)
                .add_piece('e', 4)
                .build(),
            board.white_pawns
        );
    }

    #[test]
    fn test_remove_piece() {
        let white_pawns = PositionBuilder::new()
            .add_piece('d', 2)
            .add_piece('e', 2)
            .build();
        let black_pawns = PositionBuilder::new().add_piece('f', 7).build();

        let mut board = Board::new(white_pawns, 0, 0, 0, 0, bitboard_single('e', 1).unwrap(), black_pawns, 0, 0, 0, 0, bitboard_single('e', 8).unwrap());

        assert_eq!(white_pawns, board.white_pawns);
        assert_eq!(black_pawns, board.black_pawns);

        board.remove_piece(bitboard_single('e', 2).unwrap(), true);
        board.remove_piece(bitboard_single('d', 2).unwrap(), true);
        assert_eq!(0, board.white_pawns);
        assert_eq!(black_pawns, board.black_pawns);

        // remove non-existent piece
        board.remove_piece(bitboard_single('e', 2).unwrap(), false);
        assert_eq!(0, board.white_pawns);
        assert_eq!(black_pawns, board.black_pawns);

        board.remove_piece(bitboard_single('f', 7).unwrap(), true);
        assert_eq!(0, board.white_pawns);
        assert_eq!(black_pawns, board.black_pawns);

        // remove correct black piece
        board.remove_piece(bitboard_single('f', 7).unwrap(), false);
        assert_eq!(0, board.white_pawns);
        assert_eq!(0, board.black_pawns);
    }

    #[test]
    fn test_get_piece_at() {
        let white_pawns = PositionBuilder::new()
            .add_piece('d', 2)
            .add_piece('e', 2)
            .build();
        let black_pawns = PositionBuilder::new()
            .add_piece('f', 7)
            .add_piece('g', 7)
            .build();

        let mut board = Board::new(white_pawns, 0, 0, 0, 0, bitboard_single('e', 1).unwrap(),
                                   black_pawns, 0, 0, 0, 0, bitboard_single('e', 8).unwrap());

        let actual_white_piece = *board
            .get_piece_at(bitboard_single('e', 2).unwrap(), true)
            .unwrap();
        let actual_black_piece = *board
            .get_piece_at(bitboard_single('f', 7).unwrap(), false)
            .unwrap();
        assert_eq!(board.white_pawns, actual_white_piece);
        assert_eq!(board.black_pawns, actual_black_piece);

        assert_eq!(
            None,
            board.get_piece_at(bitboard_single('e', 2).unwrap(), false)
        );
        assert_eq!(
            None,
            board.get_piece_at(bitboard_single('f', 7).unwrap(), true)
        );
        assert_eq!(
            None,
            board.get_piece_at(bitboard_single('a', 5).unwrap(), true)
        );
    }

    #[test]
    fn test_replace_piece() {
        let white_pawns = PositionBuilder::new()
            .add_piece('d', 8)
            .add_piece('e', 8)
            .build();
        let white_queen = PositionBuilder::new().add_piece('a', 1).build();

        let mut board = Board::new(white_pawns, 0, 0, 0, white_queen, bitboard_single('e', 1).unwrap(), 0, 0, 0, 0, 0, bitboard_single('e', 8).unwrap());
        assert_eq!(
            PositionBuilder::new().add_piece('a', 1).build(),
            board.white_queens
        );
        assert_eq!(
            PositionBuilder::new()
                .add_piece('d', 8)
                .add_piece('e', 8)
                .build(),
            board.white_pawns
        );
        assert_eq!(0, board.white_knights);

        board.replace_pawn(bitboard_single('e', 8).unwrap(), true, Piece::Queen);
        assert_eq!(
            PositionBuilder::new()
                .add_piece('a', 1)
                .add_piece('e', 8)
                .build(),
            board.white_queens
        );
        assert_eq!(
            PositionBuilder::new().add_piece('d', 8).build(),
            board.white_pawns
        );
        assert_eq!(0, board.white_knights);

        board.replace_pawn(bitboard_single('d', 8).unwrap(), true, Piece::Knight);
        assert_eq!(
            PositionBuilder::new()
                .add_piece('a', 1)
                .add_piece('e', 8)
                .build(),
            board.white_queens
        );
        assert_eq!(
            0, board.white_pawns
        );
        assert_eq!(
            PositionBuilder::new().add_piece('d', 8).build(),
            board.white_knights
        );
    }

    #[test]
    fn test_get_piece_type_at() {
        let board = Board::from_fen("4k1bn/6nn/R6K/2q5/3P2N1/6Q1/3P4/7B");
        let expected = [
            ('a', 6, Some((Piece::Rook, true))),
            ('a', 1, None),
            ('c', 5, Some((Piece::Queen, false))),
            ('d', 4, Some((Piece::Pawn, true))),
            ('d', 2, Some((Piece::Pawn, true))),
            ('e', 8, Some((Piece::King, false))),
            ('e', 7, None),
            ('f', 5, None),
            ('g', 8, Some((Piece::Bishop, false))),
            ('g', 7, Some((Piece::Knight, false))),
            ('g', 4, Some((Piece::Knight, true))),
            ('g', 3, Some((Piece::Queen, true))),
            ('g', 1, None),
            ('h', 8, Some((Piece::Knight, false))),
            ('h', 7, Some((Piece::Knight, false))),
            ('h', 6, Some((Piece::King, true))),
            ('h', 1, Some((Piece::Bishop, true))),
        ];
        for (file, rank, piece) in expected {
            let pos = bitboard_single(file, rank).unwrap();
            assert_eq!(piece, board.get_piece_type_at(pos));
        }
    }

    #[test]
    fn test() {
        let board = Board::default();
        board.render();
    }
}
