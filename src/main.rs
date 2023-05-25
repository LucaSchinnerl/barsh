use serde::{Deserialize, Serialize};

use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};

use std::env;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod prompt;
mod ui;

use app::{run_app, App};
use prompt::generate_prompt;
use ui::ui;

#[derive(Debug, Deserialize, Serialize)]
struct ShellCommand {
    commands: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Open AI stuff
    // Get OS and shell
    let shell = "fish";

    let prompt = generate_prompt(shell);

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
    let msg = &result.choices[0].message.content;
    let parsed_command = &msg
        [msg.find("{").expect("Response not JSON")..msg.rfind("}").expect("Response not JSON") + 1];

    let parent: ShellCommand = //Vec<String> = 
            serde_json::from_str::<ShellCommand>(
               parsed_command
            ).expect("Could not parse Commands");

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(parent.commands);
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
