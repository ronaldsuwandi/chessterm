use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use image::{DynamicImage, ImageReader};
use ratatui::{DefaultTerminal, Frame};
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Image, Resize, StatefulImage};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use rodio::buffer::SamplesBuffer;
use crate::engine::game::{Game, MoveError, Status};
use crate::ui::ui;

pub struct App {
    pub current_screen: CurrentScreen,
    pub chess_pieces: HashMap<char, RefCell<StatefulProtocol>>,

    pub input: String,
    pub character_index: usize,

    pub game: Game,
    pub error: Option<MoveError>,
    pub moves: Vec<String>,
    pub flipped: bool,

    pub picker: Picker,

    _audio_stream: OutputStream,
    audio_stream_handle: OutputStreamHandle,

    audio_buffers: HashMap<Audio, SamplesBuffer<f32>>,

}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
enum Audio {
    Move,
    Notify,
    Error
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

const MAX_MOVE_LENGTH: usize = 6;

impl App {
    pub fn new() -> Self {
        let mut pieces = HashMap::new();
        let fen_pieces = ['p', 'r', 'b', 'n', 'q', 'k', 'P', 'R', 'B', 'N', 'Q', 'K'];
        let mut picker = Picker::from_query_stdio().unwrap();

        for &piece in &fen_pieces {
            if piece == '.' {
                continue;
            }
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
                pieces.insert(piece, RefCell::new(protocol));
            }
        }

        let (_audio_stream, audio_stream_handle) = OutputStream::try_default().unwrap();
        let mut audio_buffers = HashMap::new();

        for audio_type in [Audio::Move, Audio::Error, Audio::Notify] {
            let filename = match audio_type {
                Audio::Move => "move",
                Audio::Notify => "notify",
                Audio::Error => "error",
            };

            let file = BufReader::new(File::open(format!("./assets/audio/{}.mp3", filename)).unwrap());
            // Decode that sound file into a source
            let source = Decoder::new(file).unwrap();

            // Convert into a buffered format
            let sample_rate = source.sample_rate();
            let channels = source.channels();
            let samples: Vec<f32> = source.convert_samples().collect();

            let buffer = SamplesBuffer::new(channels, sample_rate, samples);
            audio_buffers.insert(audio_type, buffer);
        }

        App {
            current_screen: CurrentScreen::Main,
            chess_pieces: pieces,
            input: String::new(),
            character_index: 0,
            game: Game::default(),
            error: None,
            moves: Vec::new(),
            flipped: false,
            picker,

            _audio_stream,
            audio_stream_handle,
            audio_buffers: audio_buffers,
        }
    }

    pub fn process_cmd(&mut self) {
        // do nothing
        if self.input.trim().is_empty() {
            return;
        }

        match self.game.process_move(self.input.as_str()) {
            Ok(_) => {
                self.error = None;
                self.moves.push(self.input.clone());
                self.input.clear();

                if self.game.status != Status::Ongoing {
                    self.current_screen = CurrentScreen::GameOver;

                    self.play_audio(Audio::Notify);
                } else {
                    self.play_audio(Audio::Move);
                }
            }
            Err(err) => {
                self.error = Some(err);
                self.play_audio(Audio::Error);
            }

        }
    }

    fn play_audio(&self, audio_type: Audio) {
        if let Some(buffer) = self.audio_buffers.get(&audio_type) {
            let sink = Sink::try_new(&self.audio_stream_handle).unwrap();
            sink.append(buffer.clone());
            sink.detach();
        }
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
            self.error = None;
        }
    }
    pub fn delete_char(&mut self) {
        self.input.pop();
        self.error = None;
        self.move_cursor_left();
    }

    pub fn new_game(&mut self) {
        self.game = Game::default();
        self.input.clear();
        self.moves.clear();
        self.error = None;
    }
}