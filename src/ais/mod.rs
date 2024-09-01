use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
};

use anyhow::Result;

pub mod client;
pub mod prompt;

use prompt::Prompt;
use tui::{backend::Backend, Terminal};

use crate::{
    ais::client::{ApiEndpoint, ClientFactory},
    app::{ui::ui, App},
};
use futures::StreamExt;

/// Represents a shell command consisting of multiple commands.
pub struct ShellCommand {
    /// A vector of command strings.
    pub commands: Vec<String>,
}

impl ShellCommand {
    /// Factory method to create a `ShellCommand` from a multiline string.
    ///
    /// This method takes a multiline string, splits it into individual lines,
    /// trims each line, filters out empty lines, and collects the remaining
    /// lines into a vector of strings.
    ///
    /// # Arguments
    ///
    /// * `input` - A multiline string containing shell commands, with the format `first shell
    /// command\nsecond shell command\n...`
    ///
    /// # Returns
    ///
    /// * `ShellCommand` - An instance of `ShellCommand` with the parsed commands.
    pub fn from_multiline(input: &str) -> Self {
        // Split the input string into lines, trim each line, filter out empty lines,
        // convert each line to a String, and collect them into a Vec<String>.
        let commands = input
            .lines() // Split into lines
            .map(str::trim) // Trim each line
            .filter(|line| !line.is_empty()) // Remove empty lines
            .map(|line| line.to_string()) // Convert each line to a String
            .collect(); // Collect into a Vec<String>

        // Return a ShellCommand instance with the collected commands.
        ShellCommand { commands }
    }

    /// Creates a new, empty `ShellCommand`.
    ///
    /// # Returns
    ///
    /// * `ShellCommand` - An instance of `ShellCommand` with an empty command list.
    pub fn new() -> Self {
        ShellCommand {
            commands: Vec::new(), // Initialize with an empty vector of commands.
        }
    }
}

/// Creates a chat completion request for the OpenAI like API.
///
/// This function generates a prompt and constructs a `CreateChatCompletionRequest`
/// using the `async_openai` crate. The request is configured with a specific model
/// and messages.
///
/// # Returns
///
/// * `Result<CreateChatCompletionRequest, serde_json::Error>` - On success, returns
///   a `CreateChatCompletionRequest` object. On failure, returns a `serde_json::Error`.
///
/// # Errors
///
/// This function will return an error if generating the prompt or building the request
/// fails.
pub fn create_request(prompt: &Prompt) -> Result<CreateChatCompletionRequest> {
    // Construct the chat completion request using the generated prompt.
    let request: CreateChatCompletionRequest = CreateChatCompletionRequestArgs::default()
        .model("llama-3.1-70b-versatile") // Specify the model to use.
        .messages([
            // Add the system message to the request.
            ChatCompletionRequestSystemMessageArgs::default()
                .content(prompt.system_message.clone())
                .build()? // Build the system message and handle potential errors.
                .into(),
            // Add the user message to the request.
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt.user_message.command.clone())
                .build()? // Build the user message and handle potential errors.
                .into(),
        ])
        .build()?; // Build the entire request and handle potential errors.

    // Return the constructed request.
    Ok(request)
}

/// Processes a stream of chat completions and updates the terminal UI with the results.
///
/// This function takes a `CreateChatCompletionRequest` and a mutable reference to a `Terminal`,
/// and processes the stream of chat completions asynchronously. It accumulates the results,
/// converts them into shell commands, and updates the terminal UI with these commands rendering
/// the streaming effect.
///
/// # Type Parameters
/// - `B`: A type that implements the `Backend` trait, used for terminal rendering.
///
/// # Arguments
/// - `stream_request`: The request to create a stream of chat completions.
/// - `terminal`: A mutable reference to the terminal that will be updated with the results.
///
/// # Returns
/// - `Result<ShellCommand>`: The accumulated shell commands from the stream.
///
/// # Errors
/// - Returns an error if there is an issue creating the stream or processing the stream results.
///
/// # Example
/// ```
/// let stream_request = CreateChatCompletionRequest::new();
/// let mut terminal = Terminal::new();
/// let result = process_stream(stream_request, &mut terminal).await;
/// match result {
///     Ok(shell_command) => println!("Shell command: {:?}", shell_command),
///     Err(e) => eprintln!("Error: {:?}", e),
/// }
/// ```
pub async fn process_stream<B: Backend>(
    stream_request: CreateChatCompletionRequest,
    terminal: &mut Terminal<B>,
    endpoint: &ApiEndpoint,
) -> Result<ShellCommand> {
    // Create a new OpenAI client and initiate a chat completion stream
    let mut stream = ClientFactory::create_client(endpoint)?
        .chat()
        .create_stream(stream_request)
        .await?;

    // Initialize an empty string to accumulate the results
    let mut accumulated_result = String::new();
    // Initialize the application state with an empty ShellCommand
    let mut app = App::new(ShellCommand::new());

    // Process each item in the stream
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                // Iterate over each choice in the response
                response.choices.iter().for_each(|chat_choice| {
                    if let Some(ref content) = chat_choice.delta.content {
                        // Append the content to the accumulated result
                        accumulated_result.push_str(content);
                        // Convert the accumulated result into shell commands
                        let shell_commands = ShellCommand::from_multiline(&accumulated_result);
                        // Update the application state with the new shell commands
                        app.items = shell_commands;
                        // Redraw the terminal UI with the updated application state
                        terminal.draw(|f| ui(f, &mut app)).unwrap();
                    }
                });
            }
            Err(err) => {
                // Handle any errors that occur while processing the stream
                anyhow::bail!("Error processing stream: {:?}", err);
            }
        }
    }
    // Return the accumulated shell commands
    Ok(ShellCommand::from_multiline(&accumulated_result))
}
