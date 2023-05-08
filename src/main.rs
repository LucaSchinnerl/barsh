use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use proceed::proceed;
use shlex::split;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get OS and shell
    let os = env::consts::OS;
    let shell = "bash";

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
    let cmd = &result.choices[0].message.content;

    println!("{}", cmd);
    println!("Enter y to run or n to cancel");

    if proceed() {
        let argv = split(cmd).expect("Could not parse command");
        Command::new(&argv[0])
            .args(&argv[1..])
            .spawn()
            .expect("Command failed to start");
    } else {
        println!("Canceled");
    }

    Ok(())
}
