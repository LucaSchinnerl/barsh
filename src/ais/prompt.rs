use anyhow::{Context, Result};
use std::env;
use std::fs;
use sysinfo::{Pid, System};

const PROMPT: &str = "Act as a natural language to {shell} command translation engine on {os}.

You are an expert in {shell} on {os} and translate the question at the end to valid syntax.

A user will as a question and will at least 5, but at most 10 unique and different {shell} command options.

All answer must be valid {shell} commands.

Output structred data that can be parsed without adjustements with each command seperated by a linebreak as follows:
command1\ncommand2\ncommand3

Only return plain text";

pub struct Prompt {
    pub system_message: String,
    pub user_message: String,
}

fn get_shell_name() -> Result<String> {
    // Initialize system information
    let sys = System::new_all();

    // Get the current process ID
    let pid = match sysinfo::get_current_pid() {
        Ok(pid) => pid,
        Err(_) => return Err(anyhow::anyhow!("Failed to get current PID")),
    };

    // Try to find the process corresponding to the current PID
    if let Some(process) = sys.process(pid) {
        // Get the parent process
        let parent = sys
            .process(process.parent().context("Parent process not found")?)
            .context("Failed to get parent process")?;

        // Get the parent process name
        let shell = parent.name().to_string_lossy().into_owned();

        // Strip the ".exe" suffix for Windows executables
        let shell = shell.strip_suffix(".exe").unwrap_or(&shell);

        // Strip the leading "-" for login shells
        let shell = shell.strip_prefix("-").unwrap_or(shell);

        // Return the resulting shell name
        return Ok(shell.to_string());
    }

    // Return an error if no process was found
    Err(anyhow::anyhow!("Shell could not be identified"))
}

pub fn get_user_query() -> Result<String> {
    // Collect all userinput to form the Shell question
    let args: Vec<String> = env::args().collect();
    // Panic if user did not input a question
    if args.len() <= 1 {
        panic!("Please input a command");
    }
    let command = args[1..].join(" ");

    Ok(command)
}

pub fn generate_prompt() -> Result<Prompt> {
    // Get the OS
    let os = env::consts::OS;

    let shell = match get_shell_name() {
        Ok(name) => name,
        Err(_) => "Bash".to_string(),
    };

    let system_message = PROMPT.replace("{os}", os).replace("{shell}", &shell);
    let user_message = get_user_query()?;

    Ok(Prompt {
        system_message,
        user_message,
    })
}
