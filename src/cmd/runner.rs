use crate::error::Result;
use std::process::{Command, ExitStatus};

pub fn run_command(command: &str) -> Result<ExitStatus> {
    let status = if cfg!(target_os = "windows") {
        if which::which("sh").is_ok() {
            Command::new("sh").arg("-c").arg(command).status()
        } else {
            Command::new("cmd").arg("/C").arg(command).status()
        }
    } else {
        Command::new("sh").arg("-c").arg(command).status()
    }?;

    if status.success() {
        Ok(status)
    } else {
        Err(crate::error::Error::CommandFailed(
            status.code().unwrap_or(-1),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_success() {
        let result = run_command("true");
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn test_run_command_failure() {
        let result = run_command("false");
        assert!(result.is_err());
    }
}
