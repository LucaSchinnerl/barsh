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
use std::time::Duration;
use ui::ui;

const MAX_RETRIES: u32 = 5;
const RETRY_DELAY: u64 = 200;

#[derive(Debug, Deserialize, Serialize)]
struct ShellCommand {
    commands: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Open AI stuff
    // Get OS and shell
    let shell = "fish";

    // Define OPENAI request
    let client = Client::new(env::var("OPENAI_SK").expect("Could not find API key"));

    let mut retries = 0;
    let parent: Option<ShellCommand>;
    loop {
        let prompt = generate_prompt(shell);
        let req = ChatCompletionRequest {
            model: chat_completion::GPT3_5_TURBO.to_string(),
            messages: vec![chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: prompt,
            }],
        };
        match call_api(&client, req).await {
            Ok(result) => {
                parent = Some(result);
                break;
            }
            Err(err) => {
                retries += 1;

                if retries >= MAX_RETRIES {
                    panic!("Could not parse response. Maximum retries reached. Exiting...");
                }
                println!("{}", err);
                println!("Retrying in {} milliseconds...", RETRY_DELAY);
                std::thread::sleep(Duration::from_millis(RETRY_DELAY));
            }
        }
    }

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(parent.unwrap().commands);
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

async fn call_api(
    client: &Client,
    req: ChatCompletionRequest,
) -> Result<ShellCommand, serde_json::Error> {
    // Send out reqest and parse command
    let result = client
        .chat_completion(req)
        .await
        .expect("Could not reach API");
    let msg = &result.choices[0].message.content;
    
    let start = msg.find("{");
    let end = msg.rfind("}"); 
    if start.is_none() || end.is_none() {
        let parsed_command = msg;
        return serde_json::from_str::<ShellCommand>(parsed_command)
    } 
    let parsed_command = &msg[start.unwrap()..end.unwrap() + 1];
    serde_json::from_str::<ShellCommand>(parsed_command)
}
