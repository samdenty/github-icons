use crate::{
  database::{self, db},
  get_slug,
  models::Repo,
  CACHE_DIR,
};
use diesel::prelude::*;
use image::{imageops::FilterType, io::Reader as ImageReader, ImageBuffer, ImageFormat};
use std::{
  env,
  error::Error,
  io::Cursor,
  path::Path,
  process::{Command, Stdio},
};
use tokio::fs;

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
