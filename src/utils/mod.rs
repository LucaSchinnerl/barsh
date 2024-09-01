use anyhow::{anyhow, Result};
use std::env;

use crate::ais::client::ApiEndpoint;

const DEFAULT_ENDPOINT: ApiEndpoint = ApiEndpoint::Groq;
#[derive(Debug)]
pub struct UserQuery {
    pub endpoint: ApiEndpoint,
    pub command: String,
}
impl UserQuery {
    pub fn new() -> UserQuery {
        UserQuery {
            endpoint: DEFAULT_ENDPOINT,
            command: "".into(),
        }
    }
}

fn get_endpoint(arg: &str) -> ApiEndpoint {
    match arg {
        "g" | "groq" => ApiEndpoint::Groq,
        "a" | "anthropic" => ApiEndpoint::Anthropic,
        "o" | "openai" => ApiEndpoint::OpenAI,
        "l" | "local" => ApiEndpoint::Local,

        _ => {
            println!("No matching endpoint found, please sellect one of the following: anthropic (a), groq (g), openai (o) or local (l)\n defaulting to groq");
            DEFAULT_ENDPOINT
        }
    }
}
pub fn get_user_query() -> Result<UserQuery> {
    // Collect all userinput to form the Shell question
    let args: Vec<String> = env::args().collect();

    // Panic if user did not input a question
    if args.len() <= 3 {
        return Err(anyhow!(
            "Please add an input, e.g. `barsh -l local what's the time` or `barsh list all files`"
        ));
    }

    let mut user_query = UserQuery::new();

    if args[1] == "-l" {
        user_query.endpoint = get_endpoint(&args[2]);
        user_query.command = args[3..].join(" ");
        if user_query.command == "" {
            return Err(anyhow!("Please add an input, e.g. `barsh -l local what's the time` or `barsh list all files`"));
        }
    } else {
        user_query.command = args[1..].join(" ")
    }

    Ok(user_query)
}
