use std::pin::Pin;
use std::marker::PhantomData;

use async_openai::types::{CreateChatCompletionStreamResponse, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateAssistantRequestArgs, CreateChatCompletionRequestArgs, CreateChatCompletionRequest};

use anyhow::{anyhow, Result};
use async_openai::config::OpenAIConfig;

pub mod client;
pub mod prompt;

use client::new_oa_client;
use futures::StreamExt;
use prompt::generate_prompt;
use tui::{backend::Backend, Terminal};

use crate::app::{ui::ui, App};

pub struct UserQuery(String);

pub struct ShellCommand {
    pub commands: Vec<String>,
}

impl ShellCommand {
    // Factory method to create a ShellCommand from a multiline string
    pub fn from_multiline(input: &str) -> Self {
        let commands = input // Split into lines
            .lines()
            .map(str::trim) // Trim each line
            .filter(|line| !line.is_empty()) // Remove empty lines
            .map(|line| line.to_string())
            .collect(); // Collect into a Vec<String>

        ShellCommand { commands }
    }

    pub fn new() -> Self {
        ShellCommand { commands: Vec::new() }
    }
}

pub fn create_request() -> Result<CreateChatCompletionRequest> {
    // The function returns a Result containing a ShellCommand object on success, or a serde_json::Error on failure.
    let prompt = generate_prompt()?;

    let request: async_openai::types::CreateChatCompletionRequest = CreateChatCompletionRequestArgs::default()
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

    Ok(request)
}

pub async fn process_stream<B: Backend>(stream_request: CreateChatCompletionRequest, terminal: &mut Terminal<B>) -> Result<ShellCommand> {
    let mut stream = new_oa_client()?.chat().create_stream(stream_request).await?;

    let mut accumulated_result = String::new();
    let mut app = App::new(ShellCommand::new());

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                response.choices.iter().for_each(|chat_choice| {
                    if let Some(ref content) = chat_choice.delta.content {
                        accumulated_result.push_str(content);
                        let shell_commands = ShellCommand::from_multiline(&accumulated_result);
                        app.items = shell_commands;
                        terminal.draw(|f| ui(f, &mut app)).unwrap();
                    }
                });
            }
            Err(err) => {
                anyhow::bail!("Error processing stream: {:?}", err);
            }
        }
    }
    Ok(ShellCommand::from_multiline(&accumulated_result))
}
