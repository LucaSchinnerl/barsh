use crossterm::event::{self, Event, KeyCode};
use shlex::split;
use std::io;
use std::process::Command;
use tui::{backend::Backend, Terminal};

use crate::ui;

use tui::widgets::TableState;

/// App holds the state of the application
pub struct App {
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub items: Vec<String>,
    /// Current input position
    pub position: usize,
    pub state: TableState,
}

impl App {
    pub fn new(input: Vec<String>) -> App {
        App {
            input_mode: InputMode::Normal,
            items: input,
            position: 0,
            state: TableState::default(),
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.position = self.items[self.state.selected().unwrap()].len();
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.position = self.items[self.state.selected().unwrap()].len();
    }

    pub fn execute(&mut self) {
        let argv =
            split(&self.items[self.state.selected().unwrap()]).expect("Could not parse command");
        Command::new(&argv[0])
            .args(&argv[1..])
            .spawn()
            .expect("Command failed to start");
    }
}

pub enum InputMode {
    Normal,
    Editing,
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Enter => {
                        app.execute();
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        app.execute();
                        return Ok(());
                    }
                    KeyCode::Char(c) => {
                        app.items[app.state.selected().unwrap()].insert(app.position, c);
                        app.position += 1;
                    }
                    KeyCode::Backspace => {
                        if app.position == app.items[app.state.selected().unwrap()].len()
                            && app.position != 0
                        {
                            app.items[app.state.selected().unwrap()].pop();
                            app.position -= 1;
                        } else if app.position > 0 {
                            app.items[app.state.selected().unwrap()].remove(app.position);
                            app.position -= 1;
                        }
                    }
                    KeyCode::Left => {
                        if 0 != app.position {
                            app.position -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if app.items[app.state.selected().unwrap()].len() != app.position {
                            app.position += 1;
                        }
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}
