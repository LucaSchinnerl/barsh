use serde::{Deserialize, Serialize};
use serde_json;
use shlex::split;
use std::process::Command;

use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};

use std::env;
use std::fs;
use std::path::PathBuf;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState, Widget},
    Frame, Terminal,
};
struct App<'a> {
    state: TableState,
    items: Vec<&'a str>,
}
impl<'a> App<'a> {
    fn new(cmds: &'a Vec<&str>) -> App<'a> {
        App {
            state: TableState::default(),
            items: cmds.to_vec(),
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
    }

    pub fn execute(&mut self) {
        let argv =
            split(self.items[self.state.selected().unwrap()]).expect("Could not parse command");
        Command::new(&argv[0])
            .args(&argv[1..])
            .spawn()
            .expect("Command failed to start");
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(50)].as_ref())
        .margin(7)
        .split(f.size());
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let rows = app.items.chunks(1).map(|item| {
        let height = item
            .iter()
            .filter(|element| !element.is_empty())
            .map(|content| content.chars().filter(|c| *c == '#').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Commands"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.state);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Get OS and shell
    let os = env::consts::OS;
    let shell = "fish";

    // Parse input
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        panic!("Please input a command");
    }
    let command = args[1..].join(" ");

    // Define the promt
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("data/promt.txt");
    let mut prompt = fs::read_to_string(path)
        .expect("Could not find promt data")
        .replace("{os}", os)
        .replace("{shell}", shell);
    prompt.push_str(&command);

    // Define OPENAI request
    let client = Client::new(env::var("OPENAI_SK").expect("Could not find API key"));
    let req = ChatCompletionRequest {
        model: chat_completion::GPT3_5_TURBO.to_string(),
        messages: vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: prompt,
        }],
    };

    // Send out reqest and parse command
    let result = client.chat_completion(req).await?;

    let parent: ShellCommand = //Vec<String> = 
            serde_json::from_str::<ShellCommand>(
            &result.choices[0]
            .message
            .content
            ).expect("Could not parse Commands");

    let cmd: Vec<&str> = parent.commands.iter().map(AsRef::as_ref).collect();

    // Define the TUI
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title("Block").borders(Borders::ALL);
        f.render_widget(block, size);
    })?;
    // create app and run it
    let app = App::new(&cmd);
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct ShellCommand {
    commands: Vec<String>,
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Enter => {
                    app.execute();
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}
