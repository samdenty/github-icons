use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use repo_icons::set_token;
use std::error::Error;

#[derive(Parser)]
struct Opts {
  #[clap(global = true, long)]
  /// Print out errors that occurred for skipped items
  debug: bool,
  #[clap(global = true, long)]
  /// Use a github token to get icons for private repos
  token: Option<String>,

  #[clap(subcommand)]
  action: Action,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
  /// Sync icons for all repos or a singular repo
  Sync {
    repo: Option<String>,
    #[clap(long)]
    unlimited: bool,
  },

  /// Set the repo icon to the given path
  Set { repo: String, icon_path: String },
  /// Set the repo icon back to the default
  SetDefault { repo: String },

  /// List all repo directories
  ListRepos {
    #[clap(long)]
    json: bool,
  },
  /// List all icons for a repo
  ListIcons {
    #[clap(long)]
    json: bool,
    repo: String,
  },

  /// Clear the cache
  ClearCache,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let opts: Opts = Opts::parse();

  if opts.debug {
    let mut builder = Builder::new();
    builder.filter_level(LevelFilter::Info);
    builder.init();
  }

  if let Some(token) = &opts.token {
    set_token(token);
  }

  match opts.action {
    Action::ClearCache => {
      git_icons::clear_cache().await?;
    }
    Action::ListIcons { repo, json } => {
      let icons = git_icons::list_icons(&repo)
        .await
        .map_err(|err| err.to_string())?;

      if json {
        println!("{}", serde_json::to_string_pretty(&icons)?)
      } else {
        for icon in icons {
          println!("{icon}");
        }
      }
    }
    Action::ListRepos { json } => {
      let repos = git_icons::list_repos().await?;

      if json {
        println!("{}", serde_json::to_string_pretty(&repos)?)
      } else {
        for repo in repos {
          println!("{}/{} {}", repo.owner, repo.repo, repo.path);
        }
      }
    }
    Action::Sync { repo, unlimited } => match repo {
      Some(repo) => git_icons::sync(&repo)
        .await
        .map_err(|err| err.to_string())?,
      None => git_icons::sync_all(opts.token.as_deref(), opts.debug, !unlimited).await?,
    },
    Action::Set { repo, icon_path } => {
      git_icons::set(&repo, &icon_path, true)
        .await
        .map_err(|err| err.to_string())?;
    }
    Action::SetDefault { repo } => {
      git_icons::set_default(&repo)
        .await
        .map_err(|err| err.to_string())?;
    }
  }

  // let icons = RepoIcons::load(user, repo).await?;

  Ok(())
}
