#![allow(unused)]

mod engine;
mod ui;

use crate::ui::app::{App, CurrentScreen};
use crate::ui::ui::{render, render_size_error};
use crossterm::event::{self, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, terminal, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Widget};
use ratatui::{DefaultTerminal, Frame, Terminal};
use std::io::{stdout, Error, ErrorKind, Stdout};
use std::{env, io, process};

pub const MIN_WIDTH: u16 = 132;
pub const MIN_HEIGHT: u16 = 46;

fn check_size(terminal: &mut DefaultTerminal) -> Result<(), io::Error> {
    let size = terminal.size()?;
    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
        terminal.clear();
        terminal.draw(|frame| render_size_error(frame, MIN_WIDTH, MIN_HEIGHT, size))?;

        loop {
            match event::read()? {
                Event::Resize(new_width, new_height) => {
                    if new_width >= MIN_WIDTH && new_height >= MIN_HEIGHT {
                        return Ok(());
                    }
                }
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press
                        && key.code == KeyCode::Char('c')
                        && key.modifiers.contains(event::KeyModifiers::CONTROL)
                        || key.code == KeyCode::Esc
                    {
                        ratatui::restore();
                        process::exit(0);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let use_halfblocks = args.contains(&"--halfblocks".to_string());
    let mut terminal = ratatui::init();
    let mut app = App::new(use_halfblocks);
    run(&mut terminal, &mut app)?;
    ratatui::restore();
    Ok(())
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<bool> {
    loop {
        check_size(terminal)?;
        terminal.hide_cursor()?;
        terminal.draw(|frame| render(frame, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('.') => {
                        app.flipped = !app.flipped;
                        continue;
                    }
                    KeyCode::Up => {
                        if app.show_scrollbar {
                            app.scroll_up(1);
                        }
                        continue;
                    }
                    KeyCode::Down => {
                        if app.show_scrollbar {
                            app.scroll_down(1);
                        }
                        continue;
                    }
                    _ => {}
                }

                match app.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Esc => app.current_screen = CurrentScreen::Exiting,
                        KeyCode::Enter => app.process_cmd(),
                        KeyCode::Char(to_insert) => app.add_char(to_insert),
                        KeyCode::Backspace => app.delete_char(),
                        _ => {}
                    },

                    CurrentScreen::GameOver => match key.code {
                        KeyCode::Char('y') => {
                            app.current_screen = CurrentScreen::Main;
                            app.new_game();
                        }
                        KeyCode::Char('n') | KeyCode::Esc => return Ok(true),
                        _ => {}
                    },
                    CurrentScreen::Exiting => match key.code {
                        KeyCode::Char('y') => return Ok(true),
                        KeyCode::Char('n') => {
                            app.current_screen = CurrentScreen::Main;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}
