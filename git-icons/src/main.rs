use clap::{Parser, Subcommand};
use env_logger::Builder;
use git_icons::GitIcons;
use log::LevelFilter;
use repo_icons::{set_token, RepoIcons};
use std::error::Error;
use std::path::Path;

#[derive(Parser)]
struct Opts {
  #[clap(long)]
  /// Print out errors that occurred for skipped items
  debug: bool,
  #[clap(long)]
  /// Use a github token to get icons for private repos
  token: Option<String>,

  #[clap(subcommand)]
  action: Action,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
  /// Sync icons for all repos or a singular repo
  Sync { repo: Option<String> },
  /// Set the repo icon to the given path
  Set { repo: String, icon_path: String },
  /// Set the repo icon back to the default
  SetDefault { repo: String },
}

macro_rules! regex {
  ($re:literal $(,)?) => {{
    static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| regex::Regex::new($re).unwrap())
  }};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let opts: Opts = Opts::parse();

  print!("{:?}", opts.action);
  if opts.debug {
    let mut builder = Builder::new();
    builder.filter_level(LevelFilter::Info);
    builder.init();
  }

  if let Some(token) = opts.token {
    set_token(token);
  }

  match opts.action {
    Action::Sync { repo } => {
      GitIcons::sync(repo.as_deref()).await?;
    }
    Action::Set { repo, icon_path } => {
      GitIcons::set(&repo, &icon_path).await?;
    }
    Action::SetDefault { repo } => {
      GitIcons::set_default(&repo).await?;
    }
  }

  // let icons = RepoIcons::load(user, repo).await?;

  Ok(())
}
