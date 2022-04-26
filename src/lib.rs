#![feature(async_closure)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod commands;
pub mod database;
mod models;
mod modify_gitignore;

use crate::models::{Icon, Repo};
use database::db;
use diesel::prelude::*;

use image::{imageops::FilterType, io::Reader as ImageReader, ImageBuffer, ImageFormat};
use once_cell::sync::Lazy;
use std::{
  env,
  error::Error,
  fs::create_dir,
  io::{Cursor, ErrorKind},
  path::{Path, PathBuf},
  process::{Command, Stdio},
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

pub struct GitIcons {}

impl GitIcons {
  pub async fn clear_cache() -> Result<(), Box<dyn Error>> {
    if CACHE_DIR.exists() {
      fs::remove_dir_all(&*CACHE_DIR).await?;
    }

    Ok(())
  }

  pub async fn sync_all() -> Result<(), Box<dyn Error>> {
    commands::sync_all().await
  }

  pub async fn sync(slug_or_path: &str) -> Result<(), Box<dyn Error>> {
    commands::sync(slug_or_path).await
  }

  pub async fn set(
    slug_or_path: &str,
    icon_path: &str,
    overwrite: bool,
  ) -> Result<(), Box<dyn Error>> {
    commands::set(slug_or_path, icon_path, overwrite).await
  }

  pub async fn set_default(slug_or_path: &str) -> Result<(), Box<dyn Error>> {
    commands::set_default(slug_or_path).await
  }

  /// Write the icon for a repo to the filesystem
  pub async fn write(slug_or_path: &str) -> Result<(), Box<dyn Error>> {
    let (user, repo_name, _) = get_slug(slug_or_path)?;

    let repo_results = {
      use database::schema::repos::dsl::*;
      repos
        .filter(owner.eq(user).and(repo.eq(&repo_name)))
        .load::<Repo>(db())?
    };

    for repo in repo_results {
      if let Some(icon_path) = repo.icon_path {
        let icon_path = CACHE_DIR.join(&icon_path);

        if !icon_path.exists() {
          continue;
        }

        let icon_rsrc = format!("{}.rsrc", icon_path.to_string_lossy());
        let icon_rsrc = Path::new(&icon_rsrc);

        let current_dir = env::current_dir()?;
        env::set_current_dir(&repo.path)?;

        if !icon_rsrc.exists() {
          let mut tmp_icon = format!(
            "/tmp/{}.png",
            Path::new(&icon_path).file_stem().unwrap().to_string_lossy()
          );

          let img = if icon_path.extension().unwrap() == "svg" {
            let svg_data = fs::read(&icon_path).await?;
            let options = usvg::Options::default();
            let rtree = usvg::Tree::from_data(&svg_data, &options.to_ref())?;

            let pixmap_size = rtree.svg_node().size.to_screen_size();
            let (width, height) = resize_box(1024, 1024, pixmap_size.width(), pixmap_size.height());

            let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();
            resvg::render(
              &rtree,
              usvg::FitTo::Size(1024, 1024),
              tiny_skia::Transform::default(),
              pixmap.as_mut(),
            );

            let mut reader = ImageReader::new(Cursor::new(pixmap.encode_png()?));
            reader.set_format(ImageFormat::Png);
            reader.decode()?
          } else {
            ImageReader::open(&icon_path)?.decode()?
          };

          let (width, height) = resize_box(1024, 1024, img.width(), img.height());

          let thumbnail = img.resize_exact(width, height, FilterType::Nearest);
          let mut img = ImageBuffer::from_fn(1024, 1024, |_x, _y| image::Rgba([255, 255, 255, 1]));
          image::imageops::overlay(
            &mut img,
            &thumbnail,
            (1024 - i64::from(width)) / 2,
            (1024 - i64::from(height)) / 2,
          );

          img.save(&tmp_icon)?;

          // if it's a .ico then it'll be outputted to -0 -1 -2 -3 etc.
          if !Path::new(&tmp_icon).exists() {
            tmp_icon = format!(
              "/tmp/{}-0.png",
              Path::new(&icon_path).file_stem().unwrap().to_string_lossy()
            );
          };

          Command::new("SetFile").args(["-a", "C", "."]).spawn()?;

          Command::new("sips")
            .args(["-i", &tmp_icon])
            .stderr(Stdio::inherit())
            .stdout(Stdio::null())
            .spawn()?
            .wait()?;

          let icon_rsrc_file = std::fs::File::create(&icon_rsrc)?;
          Command::new("DeRez")
            .args(["-only", "icns", &tmp_icon])
            .stdout(icon_rsrc_file)
            .spawn()?
            .wait()?;
        }

        Command::new("touch").args(["Icon\r"]).spawn()?.wait()?;

        Command::new("Rez")
          .args(["-append", &icon_rsrc.to_string_lossy(), "-o", "Icon\r"])
          .stderr(Stdio::inherit())
          .stdout(Stdio::null())
          .spawn()?
          .wait()?;

        Command::new("SetFile")
          .args(["-a", "V", "Icon\r"])
          .stderr(Stdio::inherit())
          .stdout(Stdio::null())
          .spawn()?
          .wait()?;

        env::set_current_dir(&current_dir)?;
      }
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

fn resize_box(box_width: u32, box_height: u32, mut width: u32, mut height: u32) -> (u32, u32) {
  let aspect_ratio = (width as f32) / (height as f32);

  if width > box_width || height < box_height {
    width = box_width;
    height = ((width as f32) / aspect_ratio) as u32;
  }

  if height > box_height || width < box_width {
    height = box_height;
    width = ((height as f32) * aspect_ratio) as u32;
  }

  (width, height)
}
