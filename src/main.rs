#![allow(unused)]

mod ui;
mod engine;

use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::ExecutableCommand;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::{DefaultTerminal, Frame};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Widget};
use crate::ui::app::{App, CurrentScreen};
use crate::ui::ui::ui;



fn main() -> Result<(), io::Error> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    run(&mut terminal, &mut app)?;
    ratatui::restore();
    Ok(())
}


fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<bool> {
    // while !self.exit {
    loop {
        terminal.show_cursor()?;
        terminal.draw(|frame| ui(frame, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
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

                    CurrentScreen::GameOver => {}
                    CurrentScreen::Exiting => {
                        match key.code {
                            KeyCode::Char('y') => {
                                return Ok(true)
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                app.current_screen = CurrentScreen::Main;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

    //     self.handle_events()?;
    }
}