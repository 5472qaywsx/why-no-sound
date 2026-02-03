//! Command execution utilities for running system commands safely.

use std::process::Command;

/// Result of running a command.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// Run a command and capture its output.
/// Never panics - returns a failed CommandOutput if the command cannot be executed.
pub fn run_command(program: &str, args: &[&str]) -> CommandOutput {
    match Command::new(program).args(args).output() {
        Ok(output) => CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        },
        Err(e) => CommandOutput {
            stdout: String::new(),
            stderr: format!("Failed to execute command: {}", e),
            success: false,
        },
    }
}
