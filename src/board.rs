use crate::moves::compute_pawns_moves;

#[derive(Debug)]
pub struct Board {
    pub white_pawns: u64,
    // pub white_knights: u64,
    // pub white_rooks: u64,
    // pub white_bishops: u64,
    // pub white_queens: u64,
    // pub white_king: u64,
    pub black_pawns: u64,
    // pub black_knights: u64,
    // pub black_rooks: u64,
    // pub black_bishops: u64,
    // pub black_queens: u64,
    // pub black_king: u64,

    pub white_pieces: u64,
    pub black_pieces: u64,
    pub occupied: u64,
    pub free: u64,
}

impl Board {
    pub fn new(white_pawns: u64, black_pawns: u64) -> Board {
        let white_pieces = white_pawns;
        let black_pieces = black_pawns;

        let occupied = white_pieces | black_pieces;
        let free = !occupied;

        Board {
            white_pawns,
            black_pawns,

            white_pieces,
            black_pieces,
            occupied,
            free,
        }
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
        self.white_pieces = self.white_pawns;
        self.black_pieces = self.black_pawns;
        self.occupied = self.white_pieces | self.black_pieces;
        self.free = !self.occupied;
    }

    pub fn get_piece_at(&mut self, position: u64, is_white: bool) -> Option<&mut u64> {
        if is_white {
            if self.white_pawns & position != 0 {
                Some(&mut self.white_pawns)
            } else {
                // TODO check for queen/bishops/rooks/knights/king
                None
            }
        } else {
            if self.black_pawns & position != 0 {
                Some(&mut self.black_pawns)
            } else {
                // TODO check for queen/bishops/rooks/knights/king
                None
            }
        }
    }

    /// moves the given piece without any checking
    pub fn move_piece(&mut self, from: u64, to: u64, is_white: bool) {
        if let Some(piece) = self.get_piece_at(from, is_white) {
           *piece = (*piece ^ from) | to;
           self.update_pieces();;
        }
    }

    /// removes piece from the board
    pub fn remove_piece(&mut self, position: u64, is_white: bool) {
        if let Some(piece) = self.get_piece_at(position, is_white) {
            *piece = *piece ^ position;
            self.update_pieces();
        }
    }

    pub fn render(&self) {
        let mut board_representation = [[' '; 8]; 8];
        // Combine all pieces into a single representation
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square_index = rank * 8 + file;

                let piece = if (self.white_pawns >> square_index) & 1 != 0 {
                    'P'
                } else if (self.black_pawns >> square_index) & 1 != 0 {
                    'p'
                // } else if (white_rooks >> square_index) & 1 != 0 {
                //     'R'
                // } else if (black_rooks >> square_index) & 1 != 0 {
                //     'r'
                // } else if (white_knights >> square_index) & 1 != 0 {
                //     'N'
                // } else if (black_knights >> square_index) & 1 != 0 {
                //     'n'
                // } else if (white_bishops >> square_index) & 1 != 0 {
                //     'B'
                // } else if (black_bishops >> square_index) & 1 != 0 {
                //     'b'
                // } else if (white_queens >> square_index) & 1 != 0 {
                //     'Q'
                // } else if (black_queens >> square_index) & 1 != 0 {
                //     'q'
                // } else if (white_kings >> square_index) & 1 != 0 {
                //     'K'
                // } else if (black_kings >> square_index) & 1 != 0 {
                //     'k'
                } else {
                    '.' // Empty square
                };

                board_representation[rank as usize][file as usize] = piece;
            }
        }

        // Render the board
        println!("  +------------------------+");
        for (rank, row) in board_representation.iter().enumerate().rev() {
            print!("{} |", rank + 1); // Print rank number
            for piece in row {
                print!(" {} ", piece);
            }
            println!("|");
        }
        println!("  +------------------------+");
        println!("    a  b  c  d  e  f  g  h");
    }
}

impl Default for Board {
    fn default() -> Board {
        let white_pawns = 0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000;
        let black_pawns = 0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000;

        Self::new(white_pawns, black_pawns)
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
        assert_eq!(true, is_rank(bitboard_single('a', 2).unwrap(), 2));
        assert_eq!(true, is_rank(bitboard_single('b', 2).unwrap(), 2));
        assert_eq!(true, is_rank(bitboard_single('c', 2).unwrap(), 2));
        assert_eq!(true, is_rank(bitboard_single('d', 2).unwrap(), 2));
        assert_eq!(true, is_rank(bitboard_single('e', 2).unwrap(), 2));
        assert_eq!(true, is_rank(bitboard_single('f', 2).unwrap(), 2));
        assert_eq!(true, is_rank(bitboard_single('g', 2).unwrap(), 2));
        assert_eq!(true, is_rank(bitboard_single('h', 2).unwrap(), 2));
        assert_eq!(true, is_rank(bitboard_single('b', 1).unwrap(), 1));
        assert_eq!(false, is_rank(bitboard_single('a', 5).unwrap(), 2));
        assert_eq!(false, is_rank(bitboard_single('b', 1).unwrap(), 2));
    }

    #[test]
    fn test_is_file() {
        assert_eq!(true, is_file(bitboard_single('b', 1).unwrap(), 'b'));
        assert_eq!(true, is_file(bitboard_single('b', 2).unwrap(), 'b'));
        assert_eq!(true, is_file(bitboard_single('b', 3).unwrap(), 'b'));
        assert_eq!(true, is_file(bitboard_single('b', 4).unwrap(), 'b'));
        assert_eq!(true, is_file(bitboard_single('b', 5).unwrap(), 'b'));
        assert_eq!(true, is_file(bitboard_single('b', 6).unwrap(), 'b'));
        assert_eq!(true, is_file(bitboard_single('b', 7).unwrap(), 'b'));
        assert_eq!(true, is_file(bitboard_single('b', 8).unwrap(), 'b'));
        assert_eq!(true, is_file(bitboard_single('a', 1).unwrap(), 'a'));
        assert_eq!(false, is_file(bitboard_single('a', 5).unwrap(), 'b'));
        assert_eq!(false, is_file(bitboard_single('g', 1).unwrap(), 'f'));
    }

    #[test]
    fn test_is_capture() {
        let white_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 2)
            .add_piece('e', 3)
            .add_piece('f', 2)
            .add_piece('g', 6)
            .build();

        let black_pawns: u64 = PositionBuilder::new()
            .add_piece('a', 3)
            .add_piece('b', 3)
            .add_piece('g', 3)
            .add_piece('h', 7)
            .build();

        let board = Board::new(white_pawns, black_pawns);

        assert!(board.is_capture(bitboard_single('b', 3).unwrap(), true));
        assert!(board.is_capture(bitboard_single('g', 3).unwrap(), true));

        assert!(board.is_capture(bitboard_single('a', 2).unwrap(), false));
        assert!(board.is_capture(bitboard_single('g', 6).unwrap(), false));

        assert!(!board.is_capture(bitboard_single('a', 2).unwrap(), true));
        assert!(!board.is_capture(bitboard_single('b', 5).unwrap(), true));
        assert!(!board.is_capture(bitboard_single('h', 6).unwrap(), false));
    }
}
