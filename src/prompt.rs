use std::env;
use std::fs;
use std::path::PathBuf;

pub fn generate_prompt(shell: &str) -> String {
    // Get OS and shell
    // Parse input
    let os = env::consts::OS;
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
    prompt
}
