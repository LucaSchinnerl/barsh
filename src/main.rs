use std::fs;
use std::env;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use std::process::Command;
use shlex::split;
use proceed::proceed;


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
    let mut prompt = fs::read_to_string("./data/promt.txt")
        .expect("Could not find promt data")
        .replace("{os}", os)
        .replace("{shell}", shell);
    prompt.push_str(&command);

    // Define OPENAI request
    let client = Client::new(env::var("OPENAI_SK").expect("Could not find API key"));
    let req = ChatCompletionRequest {
        model: chat_completion::GPT4.to_string(),
        messages: vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: prompt, 
        }],
    };

    // Send out reqest and parse command
    let result = client.chat_completion(req).await?;
    let cmd = &result.choices[0].message.content;

    println!("{}", cmd);
    println!("Enter y to expect or n to cancel");

    if proceed() {
        let argv = split(cmd).expect("Could not parse command");
        Command::new(&argv[0]).args(&argv[1..]).spawn().expect("Command failed to start");
    } else {
        println!("Canceled");
    }

    Ok(())
}
