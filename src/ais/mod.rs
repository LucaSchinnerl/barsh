use async_openai::types::{ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateAssistantRequestArgs, CreateChatCompletionRequestArgs};

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

    let request = CreateChatCompletionRequestArgs::default()
    .model("gpt-4o-mini")
    .messages([
        ChatCompletionRequestSystemMessageArgs::default()
            .content(prompt.system_message)
            .build()?
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(prompt.user_message)
            .build()?
            .into(),
    ])
    .build()?;
    //create the assistant

    let client = new_oa_client()?;
    let response = client.chat().create(request).await?;


    let msg = match response.choices.first() {
        Some(content) => {
            match content.message.content.as_deref() {
                Some(msg) => msg,
                None => return Err(anyhow!("Message content not found")),
            }
        },
        None => return Err(anyhow!("Message not found")),
    };

    let shell_command = ShellCommand::from_multiline(msg.into());

    Ok(shell_command)
}
