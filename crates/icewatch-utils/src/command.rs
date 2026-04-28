use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
}

impl Command {
    pub fn new(program: impl Into<String>) -> Self {
        Self { program: program.into(), args: vec![], cwd: None }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn run(&self) -> Result<()> {
        let mut cmd = std::process::Command::new(&self.program);
        let sanitized_args =
            self.args.iter().fold(Vec::with_capacity(self.args.len()), |mut acc, arg| {
                if arg.contains(" ") {
                    acc.push(format!("\"{}\"", arg));
                } else {
                    acc.push(arg.clone());
                }
                acc
            });
        cmd.args(&sanitized_args);
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }
        let output = cmd.output().context("Failed to execute command")?;
        if !output.status.success() {
            return Err(anyhow::anyhow!("Command failed: {}", self));
        }
        Ok(())
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.program)?;
        if !self.args.is_empty() {
            write!(f, " {}", self.args.join(" "))?;
        }
        if let Some(cwd) = &self.cwd {
            write!(f, " (cwd: {})", cwd.display())?;
        }
        Ok(())
    }
}
