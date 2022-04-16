#![feature(async_closure)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod database;
mod models;
mod modify_gitignore;

use crate::models::{Icon, Repo};
use database::db;
use diesel::prelude::*;
use futures::future;
use image::{imageops::FilterType, io::Reader as ImageReader, ImageBuffer, ImageFormat};
use once_cell::sync::Lazy;
use rand::Rng;
use repo_icons::RepoIcons;
use site_icons::IconInfo;
use std::{
  collections::hash_map::DefaultHasher,
  env,
  error::Error,
  fs::create_dir,
  hash::{Hash, Hasher},
  io::{BufRead, BufReader},
  io::{Cursor, ErrorKind},
  path::{Path, PathBuf},
  process::{Command, Stdio},
};
use tokio::{
  fs::{self, File},
  io::copy,
  task::JoinHandle,
};
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
    let home_dir = home::home_dir().unwrap();
    let home_dir = home_dir.to_str().unwrap();

    let mut cmd = Command::new("find")
      .args([
        home_dir,
        "-path",
        &format!("{}/.Trash", home_dir),
        "-prune",
        "-o",
        "-path",
        &format!("{}/Library", home_dir),
        "-prune",
        "-o",
        "-type",
        "d",
        "-name",
        ".git",
        "-exec",
        "echo",
        "{}",
        ";",
      ])
      .stderr(Stdio::inherit())
      .stdout(Stdio::piped())
      .spawn()?;

    let stdout = cmd.stdout.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    for repo_path in stdout_lines {
      let repo_path = repo_path?;
      let repo_path = match repo_path.strip_suffix("/.git") {
        Some(repo_path) => repo_path,
        None => &repo_path,
      };

      // ignore paths contained within hidden folders
      if repo_path
        .split("/")
        .find(|path| path.starts_with("."))
        .is_some()
      {
        continue;
      }

      println!("{}", &repo_path);

      match GitIcons::sync(&repo_path).await {
        Err(error) => eprintln!("{}", error),
        _ => {}
      };
    }
    // let tasks: Vec<_> = stdout_lines
    //   .map(|repo_path| {
    //     tokio::spawn(async move {
    //       GitIcons::sync(&repo_path.unwrap()).await.unwrap();
    //     })
    //   })
    //   .collect();

    // future::join_all(tasks).await;

    Ok(())
  }

  pub async fn sync(slug_or_path: &str) -> Result<(), Box<dyn Error>> {
    modify_gitignore::modify()?;

    let (user, repo_name, repo_path) = get_slug(slug_or_path)?;
    let icons = RepoIcons::load(&user, &repo_name).await;

    if let Ok(icons) = icons {
      let mut tasks: Vec<_> = icons
        .into_iter()
        .enumerate()
        .map(|(i, icon)| -> JoinHandle<Option<()>> {
          let slug_or_path = slug_or_path.to_string();
          let (user, repo_name) = (user.clone(), repo_name.clone());

          tokio::spawn(async move {
            let cache_name = format!("{}{}", icon.url.host_str().unwrap_or(""), icon.url.path())
              .replace("/", "-")
              .replace(":", "-");

            let mut hasher = DefaultHasher::new();
            cache_name.hash(&mut hasher);

            let icon_name = format!(
              "{}.{}",
              hasher.finish().to_string(),
              match icon.info {
                IconInfo::PNG { .. } => "png",
                IconInfo::JPEG { .. } => "jpg",
                IconInfo::ICO { .. } => "ico",
                IconInfo::SVG => "svg",
              }
            );
            let icon_path = CACHE_DIR.join(icon_name.clone());

            if !icon_path.exists() {
              let mut icon_file = File::create(&icon_path).await.ok()?;

              match icon.url.scheme() {
                "data" => {
                  let data_uri_path = icon.url.path();
                  let data_index = data_uri_path.find(",").unwrap_or(0);
                  let type_index = data_uri_path[..data_index].find(";");

                  let data = data_uri_path[(data_index + 1)..].to_string();
                  let mut written = false;

                  if let Some(type_index) = type_index {
                    let data_type = data_uri_path[(type_index + 1)..data_index].to_string();
                    if data_type == "base64" {
                      let mut content = Cursor::new(base64::decode(&data).unwrap_or(Vec::new()));
                      copy(&mut content, &mut icon_file).await.ok()?;
                      written = true;
                    }
                  }

                  if !written {
                    let mut content = Cursor::new(urlencoding::decode_binary(&data.as_bytes()));
                    copy(&mut content, &mut icon_file).await.ok()?;
                  }
                }
                _ => {
                  let response = reqwest::get(icon.url).await.ok()?;
                  let mut content = Cursor::new(response.bytes().await.ok()?);

                  copy(&mut content, &mut icon_file).await.ok()?;
                }
              }
            }

            // If it's the first icon, then write it as the default to
            if i == 0 {
              GitIcons::set(&slug_or_path, &icon_name, false).await.ok()?;
            }

            let icon = Icon {
              owner: user,
              repo: repo_name,
              path: icon_name,
            };

            {
              use database::schema::icons::dsl::*;
              diesel::insert_or_ignore_into(icons)
                .values(&icon)
                .execute(db())
                .ok()?;
            }

            Some(())
          })
        })
        .collect();

      while !tasks.is_empty() {
        match future::select_all(tasks).await {
          (Ok(_), _index, remaining) => {
            tasks = remaining;
          }
          (Err(error), _index, remaining) => {
            eprintln!("{:?}", error);
            tasks = remaining;
          }
        }
      }
    } else {
      eprintln!("{:?}", icons);

      // add the repo with an empty icon
      if let Some(repo_path) = repo_path {
        let new_repo = Repo {
          owner: user.clone(),
          repo: repo_name.clone(),
          path: repo_path.clone(),
          icon_path: None,
        };

        {
          use database::schema::repos::dsl::*;

          diesel::insert_or_ignore_into(repos)
            .values(&new_repo)
            .execute(db())?;
        }
      }
    }

    GitIcons::write(&slug_or_path).await?;

    Ok(())
  }

  async fn set_with_repo_path(
    repo_path: &str,
    icon_path: &str,
    overwrite: bool,
  ) -> Result<(), Box<dyn Error>> {
    let (user, repo_name, repo_path) = get_slug(repo_path)?;
    let repo_path = repo_path.unwrap();
    let icon_path = CACHE_DIR.join(icon_path);

    let icon_name = if icon_path.exists() {
      if icon_path.starts_with(&*CACHE_DIR) {
        icon_path
          .file_name()
          .unwrap()
          .to_string_lossy()
          .into_owned()
      } else {
        let icon_name = format!(
          "{}.{}",
          rand::thread_rng().gen_range(100000000..999999999),
          icon_path.extension().unwrap().to_string_lossy()
        );

        fs::copy(icon_path, CACHE_DIR.join(&icon_name)).await?;

        icon_name
      }
    } else {
      if !icon_path.exists() {
        return Err(Box::new(std::io::Error::new(
          std::io::ErrorKind::NotFound,
          "Icon not found",
        )));
      };

      icon_path.to_string_lossy().into_owned()
    };

    let icon = Icon {
      owner: user.clone(),
      repo: repo_name.clone(),
      path: icon_name.clone(),
    };

    {
      use database::schema::icons::dsl::*;
      diesel::insert_or_ignore_into(icons)
        .values(&icon)
        .execute(db())?;
    }

    let new_repo = Repo {
      owner: user.clone(),
      repo: repo_name.clone(),
      path: repo_path.clone(),
      icon_path: Some(icon_name.clone()),
    };

    {
      use database::schema::repos::dsl::*;

      diesel::insert_or_ignore_into(repos)
        .values(&new_repo)
        .execute(db())?;

      if overwrite
        || repos
          .filter(owner.eq(&user).and(repo.eq(&repo_name)))
          .first::<Repo>(db())?
          .icon_path
          .is_none()
      {
        diesel::update(
          repos.filter(
            owner
              .eq(&user)
              .and(repo.eq(&repo_name))
              .and(path.eq(&repo_path)),
          ),
        )
        .set(icon_path.eq(&icon_name))
        .execute(db())?;
      }
    }

    GitIcons::write(&repo_path).await?;

    Ok(())
  }

  pub async fn set(
    slug_or_path: &str,
    icon_path: &str,
    overwrite: bool,
  ) -> Result<(), Box<dyn Error>> {
    let (user, repo_name, repo_path) = get_slug(slug_or_path)?;

    if repo_path.is_some() {
      GitIcons::set_with_repo_path(slug_or_path, icon_path, overwrite).await?;
    } else {
      let repos = {
        use database::schema::repos::dsl::*;

        repos
          .filter(owner.eq(&user).and(repo.eq(&repo_name)))
          .load::<Repo>(db())?
      };

      for repo in repos {
        GitIcons::set_with_repo_path(&repo.path, icon_path, overwrite).await?;
      }
    };

    Ok(())
  }

  pub async fn set_default(slug_or_path: &str) -> Result<(), Box<dyn Error>> {
    let (user, repo_name, repo_path) = get_slug(slug_or_path)?;

    {
      use database::schema::repos::dsl::*;
      if let Some(repo_path) = repo_path {
        diesel::delete(
          repos.filter(
            owner
              .eq(&user)
              .and(repo.eq(&repo_name))
              .and(path.eq(&repo_path)),
          ),
        )
        .execute(db())?;
      } else {
        diesel::delete(repos.filter(owner.eq(&user).and(repo.eq(&repo_name)))).execute(db())?;
      };
    }

    GitIcons::sync(slug_or_path).await?;

    Ok(())
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
