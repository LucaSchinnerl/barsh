use crossterm::event::{self, Event, KeyCode};
use std::io;
use std::process::Command;
use tui::{backend::Backend, Terminal};

pub mod ui;

use ui::ui;

use tui::widgets::TableState;

use crate::ais::ShellCommand;

/// App holds the state of the application
pub struct App {
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub items: ShellCommand,
    /// Current input position
    pub position: usize,
    /// Table state
    pub state: TableState,
}

impl App {
    pub fn new(input: ShellCommand) -> App {
        App {
            input_mode: InputMode::Normal,
            items: input,
            position: 0,
            state: TableState::default(),
        }
    }

    pub fn next(&mut self) {
        // Select next item in the table
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.commands.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.position = self.items.commands[self.state.selected().unwrap()].len();
    }

    pub fn previous(&mut self) {
        // Select previous item in table
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.commands.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.position = self.items.commands[self.state.selected().unwrap()].len();
    }

    pub fn execute(&mut self) {
        // Exectue the current selected command
        let command_str = &self.items.commands[self.state.selected().unwrap()];
        Command::new("sh")
            .arg("-c")
            .arg(command_str)
            .spawn()
            .expect("Command failed to start");
    }
}

pub enum InputMode {
    Normal,  // Normal mode
    Editing, // Editing mode
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Handle input
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
                        app.items.commands[app.state.selected().unwrap()].insert(app.position, c);
                        app.position += 1;
                    }
                    KeyCode::Backspace => {
                        if app.position == app.items.commands[app.state.selected().unwrap()].len()
                            && app.position != 0
                        {
                            app.items.commands[app.state.selected().unwrap()].pop();
                            app.position -= 1;
                        } else if app.position > 0 {
                            app.items.commands[app.state.selected().unwrap()]
                                .remove(app.position - 1);
                            app.position -= 1;
                        }
                    }
                    KeyCode::Left => {
                        if 0 != app.position {
                            app.position -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if app.items.commands[app.state.selected().unwrap()].len() != app.position {
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
