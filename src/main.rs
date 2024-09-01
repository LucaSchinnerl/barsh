mod ais;
mod app;
mod utils;

use app::{run_app, App};

use ais::prompt::generate_prompt;
use ais::{create_request, process_stream};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Generate the prompt which includes system and user messages.
    let prompt = generate_prompt()?;

    let request = create_request(&prompt)?;

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let shell_command =
        process_stream(request, &mut terminal, &prompt.user_message.endpoint).await?;

    // create app and run it
    let app = App::new(shell_command);
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
