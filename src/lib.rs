#![feature(async_closure)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod commands;
pub mod database;
mod models;
mod modify_gitignore;

pub use commands::*;

use crate::models::{Icon, Repo};
use database::db;
use diesel::prelude::*;

use once_cell::sync::Lazy;
use std::{
  error::Error,
  fs::create_dir,
  io::ErrorKind,
  path::{Path, PathBuf},
  process::Command,
};
use tokio::fs;
use url::Url;

static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
  let path = Path::new(&home::home_dir().unwrap()).join("Library/Caches/com.samdenty.git-icons");

  if !path.exists() {
    create_dir(&path).unwrap()
  }

  path
});

pub async fn clear_cache() -> Result<(), Box<dyn Error>> {
  if CACHE_DIR.exists() {
    fs::remove_dir_all(&*CACHE_DIR).await?;
  }

  Ok(())
}

pub async fn list_icons(slug_or_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
  let (user, repo_name, _) = get_slug(slug_or_path)?;

  let icons = {
    use database::schema::icons::dsl::*;

    icons
      .filter(owner.eq(&user).and(repo.eq(&repo_name)))
      .load::<Icon>(db())?
  };

  Ok(
    icons
      .into_iter()
      .map(|icon| CACHE_DIR.join(icon.path).to_string_lossy().to_string())
      .collect(),
  )
}

pub async fn list_repos() -> Result<Vec<Repo>, Box<dyn Error>> {
  let mut repo_results = {
    use database::schema::repos::dsl::*;
    repos.load::<Repo>(db())?
  };

  for repo_result in &mut repo_results {
    repo_result.icon_path = match &repo_result.icon_path {
      Some(icon_path) => Some(CACHE_DIR.join(icon_path).to_string_lossy().to_string()),
      None => None,
    }
  }

  Ok(repo_results)
}

fn get_slug(repo: &str) -> Result<(String, String, Option<String>), Box<dyn Error>> {
  if repo.split("/").count() == 2 && !Path::new(&repo).exists() {
    let mut slug = repo.split("/");

    let user = slug.next().unwrap().to_string();
    let repo = slug.next().unwrap().to_string();

    Ok((user, repo, None))
  } else {
    let output = Command::new("git")
      .args(["config", "--get", "remote.origin.url"])
      .current_dir(Path::new(repo))
      .output()?;

    let url = Url::parse(&String::from_utf8(output.stdout)?)
      .map_err(|_| std::io::Error::new(ErrorKind::Other, "No repository found for folder"))?;
    let slug = url.path().strip_suffix(".git").unwrap_or(url.path());
    let mut slug = slug.split("/");

    slug.next();

    let user = slug
      .next()
      .ok_or(std::io::Error::new(
        ErrorKind::Other,
        "No user in repo url found",
      ))?
      .to_string();

    let repo_name = slug
      .next()
      .ok_or(std::io::Error::new(
        ErrorKind::Other,
        "No repo in repo url found",
      ))?
      .to_string();

    Ok((user, repo_name, Some(repo.to_string())))
  }
}
