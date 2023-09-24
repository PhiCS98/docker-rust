use std::io::Write;
use anyhow::{Context, Result};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let output = std::process::Command::new(command)
        .args(command_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    std::io::stderr().write_all(&output.stderr).unwrap();
    if output.status.success() {
        std::io::stdout().write_all(&output.stdout)?;
    }
    std::process::exit(output.status.code().unwrap_or(1))
}
