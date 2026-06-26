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

// run_command 是对 std::process::Command 的薄封装，其行为由标准库保证，无需单元测试
