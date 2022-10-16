use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use repo_icons::{set_token, RepoIcons};
use std::error::Error;

#[derive(Parser)]
struct Opts {
  slug: String,
  #[clap(long)]
  json: bool,
  #[clap(long)]
  /// Print out errors that occurred for skipped items
  debug: bool,
  #[clap(long)]
  /// Use a github token to get icons for private repos
  token: Option<String>,
}

macro_rules! regex {
  ($re:literal $(,)?) => {{
    static RE: once_cell::sync::OnceCell<fancy_regex::Regex> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| fancy_regex::Regex::new($re).unwrap())
  }};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let opts: Opts = Opts::parse();

  if opts.debug {
    let mut builder = Builder::new();
    builder.filter_level(LevelFilter::Info);
    builder.init();
  }

  set_token(opts.token);

  let slug = regex!("([^/]+)/(.+)")
    .captures(&opts.slug)
    .unwrap()
    .unwrap();
  let user = &slug[1];
  let repo = &slug[2];

  let icons = RepoIcons::load(user, repo, true).await?;
  let icon = icons.best_match();

  if opts.json {
    println!("{}", serde_json::to_string_pretty(icon)?)
  } else {
    println!("{} {} {}", icon.url, icon.kind, icon.info);
  }

  Ok(())
}
