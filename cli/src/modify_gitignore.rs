use std::{
  fs::OpenOptions,
  io::{BufRead, BufReader, Write},
  error::Error,
  process::Command,
};

pub fn modify() -> Result<(), Box<dyn Error + Send + Sync>> {
  let output = Command::new("git")
    .args(["config", "--global", "core.excludesFile"])
    .output()?;
  let mut gitignore = shellexpand::full(&String::from_utf8(output.stdout)?)?.to_string();

  if gitignore == "" {
    Command::new("git")
      .args(["config", "--global", "core.excludesFile", "~/.gitignore"])
      .output()?;
    gitignore = shellexpand::full("~/.gitignore")?.to_string();
  }

  gitignore = gitignore
    .strip_suffix("\r\n")
    .or(gitignore.strip_suffix("\n"))
    .unwrap_or(&gitignore)
    .to_string();

  let mut gitignore_file = OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)
    .create(true)
    .open(&gitignore)?;

  let reader = BufReader::new(&gitignore_file);

  for line in reader.lines() {
    if line? == "Icon?" {
      return Ok(());
    }
  }

  writeln!(gitignore_file, "Icon?")?;
  writeln!(gitignore_file, "![iI]con[_a-zA-Z0-9]")?;

  Ok(())
}
