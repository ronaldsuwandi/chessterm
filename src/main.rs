#![allow(unused)]

mod ui;
mod engine;

use std::io;
use std::io::{stdout, Error, ErrorKind, Stdout};
use crossterm::event::{self, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::{execute, terminal, ExecutableCommand};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{DefaultTerminal, Frame, Terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Widget};
use crate::ui::app::{App, CurrentScreen};
use crate::ui::ui::render;

const MIN_WIDTH: u16 = 140;
const MIN_HEIGHT: u16 = 46;

fn check_size(terminal: &mut DefaultTerminal) -> bool {
    let size = terminal.size().unwrap();
    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
        terminal.clear();
        println!("TOO SMALL");
        false
        // return Err(Error::new(ErrorKind::Other, format!("Terminal must have at least {MIN_WIDTH} x {MIN_HEIGHT} dimension. Current size: {} x {}", size.width, size.height)));
    } else {
        true
    }
    // Ok(())
}

fn main() -> Result<(), io::Error> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    run(&mut terminal, &mut app)?;
    ratatui::restore();
    Ok(())
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<bool> {
    loop {
        if !check_size(terminal) {
            continue;
        }
        terminal.hide_cursor()?;
        terminal.draw(|frame| render(frame, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Char('.') {
                    app.flipped = !app.flipped;
                    continue;
                }

                match app.current_screen {
                    CurrentScreen::Main => {
                        match key.code {
                            KeyCode::Esc => app.current_screen = CurrentScreen::Exiting,
                            KeyCode::Enter => app.process_cmd(),
                            KeyCode::Char(to_insert) => app.add_char(to_insert),
                            KeyCode::Backspace => app.delete_char(),
                            _ => {}
                        }
                    }

                    CurrentScreen::GameOver => {
                        match key.code {
                            KeyCode::Char('y') => {
                                app.current_screen = CurrentScreen::Main;
                                app.new_game();
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                return Ok(true)
                            }
                            _ => {}
                        }
                    }
                    CurrentScreen::Exiting => {
                        match key.code {
                            KeyCode::Char('y') => {
                                return Ok(true)
                            }
                            KeyCode::Char('n') => {
                                app.current_screen = CurrentScreen::Main;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}