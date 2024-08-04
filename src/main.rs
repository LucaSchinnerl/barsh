mod ais;
mod app;

use app::{run_app, App};

use ais::{create_request, process_stream, ShellCommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{backend::CrosstermBackend, Terminal};

use std::time::Duration;

use anyhow::{anyhow, Result};

const MAX_RETRIES: u32 = 5;
const RETRY_DELAY: u64 = 200;

#[tokio::main]
async fn main() -> Result<()> {

    let request = create_request()?;

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let shell_command = process_stream(request, &mut terminal).await?;

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
