#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod database;

use database::db;
use repo_icons::RepoIcons;
use std::{
  error::Error,
  io::{Cursor, ErrorKind},
  path::Path,
  process::Command,
};
use tokio::{fs::File, io::copy};
use url::Url;

pub struct GitIcons {}

impl GitIcons {
  pub async fn sync(repo: Option<&str>) -> Result<(), Box<dyn Error>> {
    if let Some(repo) = repo {
      let (user, repo) = get_slug(repo)?;
      let icons = RepoIcons::load(&user, &repo).await?;

      for icon in icons {
        let cache_name = format!("{}{}", icon.url.host_str().unwrap(), icon.url.path())
          .replace("/", "-")
          .replace(":", "-");

        let cache_path = Path::new(".cache").join(cache_name);

        if !cache_path.exists() {
          let mut cache_file = File::create(&cache_path).await?;

          let response = reqwest::get(icon.url).await?;
          let mut content = Cursor::new(response.bytes().await?);

          copy(&mut content, &mut cache_file).await?;
        }
      }
      // println!("{:?}", icons);
    } else {
    }
    Ok(())
  }

  pub async fn set(repo: &str, icon_path: &str) -> Result<(), Box<dyn Error>> {
    let db = db();

    let (user, repo) = get_slug(repo)?;
    println!("{user}/{repo}");

    Ok(())
  }

  pub async fn set_default(repo: &str) -> Result<(), Box<dyn Error>> {
    let db = db();

    let (user, repo) = get_slug(repo)?;
    Ok(())
  }
}

fn get_slug(repo: &str) -> Result<(String, String), Box<dyn Error>> {
  if repo.split("/").count() == 2 && !Path::new(&repo).exists() {
    let mut slug = repo.split("/");

    let user = slug.next().unwrap().to_string();
    let repo = slug.next().unwrap().to_string();

    Ok((user, repo))
  } else {
    let output = Command::new("git")
      .args(["config", "--get", "remote.origin.url"])
      .current_dir(Path::new(repo))
      .output()?;

    let url = Url::parse(&String::from_utf8(output.stdout)?)?;

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

    let repo = slug
      .next()
      .ok_or(std::io::Error::new(
        ErrorKind::Other,
        "No repo in repo url found",
      ))?
      .to_string();

    Ok((user, repo))
  }
}
