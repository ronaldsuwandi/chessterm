use std::collections::HashMap;
use std::io;
use std::path::Path;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use image::ImageReader;
use ratatui::{DefaultTerminal, Frame};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::StatefulImage;
use crate::ui::ui;

pub struct App {
    pub current_screen: CurrentScreen,
    pub chess_pieces: HashMap<char, StatefulProtocol>,

    pub input: String,
    pub character_index: usize,
}


pub enum CurrentScreen {
    Main,
    GameOver,
    Exiting,
}

pub enum CurrentlyEditing {
    Key,
    Value,
}
//
// pub struct Assets {
//
// }

const MAX_MOVE_LENGTH: usize = 6;

impl App {
    pub fn new() -> Self {
        let mut picker = Picker::from_fontsize((8, 12)); // Adjust font size to match terminal
        let mut pieces = HashMap::new();

        let fen_pieces = ['p', 'r', 'b', 'n', 'q', 'k', 'P', 'R', 'B', 'N', 'Q', 'K'];

        for &piece in &fen_pieces {
            let filename = match piece {
                'p' => "pawn_black",
                'P' => "pawn_white",
                'r' => "rook_black",
                'R' => "rook_white",
                'b' => "bishop_black",
                'B' => "bishop_white",
                'n' => "knight_black",
                'N' => "knight_white",
                'q' => "queen_black",
                'Q' => "queen_white",
                'k' => "king_black",
                'K' => "king_white",
                _ => panic!("Unknown piece: {}", piece),
            };
            let path = format!("./assets/sprite/{}.png", filename);
            if let Ok(dyn_img) = ImageReader::open(Path::new(&path)).unwrap().decode() {
                let protocol = picker.new_resize_protocol(dyn_img);
                pieces.insert(piece, protocol);
            }
        }

        App {
            current_screen: CurrentScreen::Main,
            chess_pieces: pieces,
            input: String::new(),
            character_index: 0,
        }
    }

    pub fn process_cmd(&mut self) {

    }
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn add_char(&mut self, ch: char) {
        if self.input.chars().count() < MAX_MOVE_LENGTH {
            self.input.push(ch);
            self.move_cursor_right();
        }
    }
    pub fn delete_char(&mut self) {
        self.input.pop();
        self.move_cursor_left();
    }
}