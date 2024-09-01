use anyhow::{Context, Result};
use std::env;
use sysinfo::System;

use crate::utils::{get_user_query, UserQuery};

const PROMPT: &str = "Act as a natural language to {shell} command translation engine on {os}.

You are an expert in {shell} on {os} and translate the question at the end to valid syntax.

A user will as a question and will at least 5, but at most 10 unique and different {shell} command options.

Always provide at least 5 different commands, but at most 10.

All answer must be valid {shell} commands.

Output structred data that can be parsed without adjustements with each command seperated by a linebreak as follows:
option_1_command\noption_2_command\noption_3_command etc.

Only return plain text and no additional information.";

/// Represents a prompt with system and user messages.
pub struct Prompt {
    /// The system message to be included in the prompt.
    pub system_message: String,
    /// The user message to be included in the prompt.
    pub user_message: UserQuery,
}

/// Retrieves the name of the shell being used.
///
/// This function attempts to identify the shell by examining the parent process
/// of the current process. It handles various edge cases such as stripping
/// suffixes for Windows executables and login shells.
///
/// # Returns
///
/// * `Result<String>` - On success, returns the name of the shell as a `String`.
///   On failure, returns an error indicating the reason for failure.
///
/// # Errors
///
/// This function will return an error if it fails to get the current process ID,
/// find the parent process, or identify the shell name.
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
