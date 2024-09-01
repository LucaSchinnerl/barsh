use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode};
use shlex::split;
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

/// Represents the application state and behavior.
impl App {
    /// Creates a new `App` instance with the given `ShellCommand`.
    ///
    /// # Arguments
    ///
    /// * `input` - A `ShellCommand` containing the initial commands.
    ///
    /// # Returns
    ///
    /// * `App` - A new instance of `App`.
    pub fn new(input: ShellCommand) -> App {
        App {
            input_mode: InputMode::Normal,
            items: input,
            position: 0,
            state: TableState::default(),
        }
    }

    /// Selects the next item in the table.
    /// This method updates the selected item to the next one in the list.
    /// If the current item is the last one, it wraps around to the first item.
    pub fn next(&mut self) {
        // Determine the next index to select.
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.commands.len() - 1 {
                    0 // Wrap around to the first item.
                } else {
                    i + 1 // Select the next item.
                }
            }
            None => 0, // If no item is selected, select the first item.
        };
        self.state.select(Some(i));
        self.position = self.items.commands[self.state.selected().unwrap()].len();
    }

    /// Selects the previous item in the table.
    ///
    /// This method updates the selected item to the previous one in the list.
    /// If the current item is the first one, it wraps around to the last item.
    pub fn previous(&mut self) {
        // Determine the previous index to select.
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.commands.len() - 1 // Wrap around to the last item.
                } else {
                    i - 1 // Select the previous item.
                }
            }
            None => 0, // If no item is selected, select the first item.
        };
        self.state.select(Some(i));
        self.position = self.items.commands[self.state.selected().unwrap()].len();
    }

    /// Executes the currently selected command.
    ///
    /// This method splits the selected command into arguments and spawns a new process
    /// to execute the command.
    ///
    /// # Errors
    ///
    /// This method will return an error if the command parsing or execution fails.
    pub fn execute(&mut self) -> Result<()> {
        // Get the selected command and split it into arguments.
        let selected = self.state.selected().context("No item selected")?;
        let argv = split(&self.items.commands[selected]).context("Could not parse command")?;

        // Spawn a new process to execute the command.
        Command::new(&argv[0])
            .args(&argv[1..])
            .spawn()
            .context("Command failed to start")?;

        Ok(())
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
                        if let Err(e) = app.execute() {
                            eprintln!("Error executing command: {}", e);
                        }
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        if let Err(e) = app.execute() {
                            eprintln!("Error executing command: {}", e);
                        }
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
