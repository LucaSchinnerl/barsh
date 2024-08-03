use async_openai::types::CreateCompletionRequestArgs;

use anyhow::{anyhow, Result};
use async_openai::config::OpenAIConfig;

pub mod client;
pub mod prompt;

use client::new_oa_client;
use prompt::generate_prompt;

pub struct UserQuery(String);

pub struct ShellCommand {
    pub commands: Vec<String>,
}

impl ShellCommand {
    // Factory method to create a ShellCommand from a multiline string
    fn from_multiline(input: String) -> Self {
        let commands = input // Split into lines
            .lines()
            .map(str::trim) // Trim each line
            .filter(|line| !line.is_empty()) // Remove empty lines
            .map(|line| line.to_string())
            .collect(); // Collect into a Vec<String>

        ShellCommand { commands }
    }
}

pub async fn get_commands() -> Result<(ShellCommand)> {
    // The function returns a Result containing a ShellCommand object on success, or a serde_json::Error on failure.

    let prompt = generate_prompt()?;

    // Send out request and parse command
    let request = CreateCompletionRequestArgs::default()
        .model("gpt-3.5-turbo-instruct")
        .prompt(prompt)
        .build()?;

    let result = new_oa_client()?
        .completions()
        .create(request)
        .await
        .unwrap();

    let msg = match result.choices.first() {
        Some(content) => content.text.clone(),
        None => return Err(anyhow!("Message not found")),
    };

    let shell_command = ShellCommand::from_multiline(msg);

    Ok(shell_command)
}
