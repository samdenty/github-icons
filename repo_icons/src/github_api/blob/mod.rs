mod repo_files;

use crate::RepoBlob;
use fancy_regex::{escape, Regex};
use futures::future::join_all;
use repo_files::{get_repo_files, File, FileType};
use std::error::Error;
use std::path::Path;

const OWNER_SUFFIXES: [&str; 5] = ["js", "rs", "io", "land", "pkg"];

pub(crate) fn owner_name_lowercase(owner: &str) -> String {
  let owner = owner.to_lowercase();
  let mut owner = owner.rsplit_once('-').unwrap_or((&owner, "")).0;

  for suffix in OWNER_SUFFIXES {
    owner = owner.strip_suffix(suffix).unwrap_or(owner);
  }

  owner.to_string()
}

fn is_valid_blob(file: &File) -> bool {
  matches!(file.r#type, FileType::Blob)
    && (file.path.ends_with(".png") || file.path.ends_with(".ico") || file.path.ends_with(".svg"))
}

fn get_weight(owner: &str, repo: &str, file: &File) -> u8 {
  let owner = owner_name_lowercase(owner);
  let repo = repo.to_lowercase();

  let fullpath = file.path.to_lowercase();
  let (path, filename) = get_path_and_filename(&fullpath);

  let mut weight = 0;
  let mut matches_icon = false;

  if filename.contains("issue") || path.contains("setup") {
    return 0;
  }

  if filename.contains(&repo) {
    matches_icon = true;
    weight += 2;
  }

  let exactly_repo_name = Regex::new(&format!(
    "^{}([-_](logo|icon).*)?([-_]?(\\d+x\\d+|\\d+))?\\.[^.]+$",
    escape(&repo)
  ))
  .unwrap();
  if exactly_repo_name.is_match(&filename).unwrap() {
    weight += 2;
  }

  let fixtures =
    regex!("(e2e|fixtures|(examples?)|third[-_]party|manual|extensions|themes|tests?)/");
  if !fixtures.is_match(&fullpath).unwrap() {
    if filename.contains(&owner) {
      matches_icon = true;
      weight += 1;
    }

    let is_favicon = filename.contains("favicon");
    if is_favicon {
      matches_icon = true;
      weight += 2;
    }

    let app_icon = regex!("(app.*icon)|(icon.*app)");
    if app_icon.is_match(filename).unwrap() {
      matches_icon = true;
      weight += 2;
    }

    let logo = regex!("logo(?!ut|n|s)");
    if logo.is_match(&fullpath).unwrap() {
      matches_icon = true;
      weight += 1;

      let exactly_logo = regex!("^(logo|icon)\\.[^.]+$");
      if exactly_logo.is_match(&filename).unwrap() {
        weight += 2;
      }
    }

    if matches_icon {
      let public = regex!("(public|static|resources|assets|media|www|xcassets|appiconset)/");
      if public.is_match(&fullpath).unwrap() {
        weight += 1;
      }

      let directly_in_images = regex!("(images|img|public|static|resources|assets|media|www)$");
      if directly_in_images.is_match(&path).unwrap() {
        weight += 2;
        if is_favicon {
          weight += 1;
        }
      }

      if path.contains("server") || fullpath.contains("website") {
        weight += 1;
      }
    }
  }

  weight
}

async fn get_package_json_icon(
  owner: &str,
  repo: &str,
  commit_sha: &str,
  files: &Vec<File>,
) -> Option<File> {
  let package_json_icons: Vec<(bool, File)> = join_all(
    files
      .iter()
      .filter(|file| {
        let (path, filename) = get_path_and_filename(&file.path);

        matches!(file.r#type, FileType::Blob)
          && path == ""
          && filename == "package.json"
          && files.iter().any(|file| {
            (path == "" || file.path.starts_with(&format!("{}/", path))) && is_valid_blob(file)
          })
      })
      .map(async move |file| {
        #[derive(Deserialize)]
        struct PackageJSON {
          icon: String,
        }

        let package_json = gh_get!(
          "https://raw.githubusercontent.com/{}/{}/{}/{}",
          owner,
          repo,
          commit_sha,
          file.path
        )
        .send()
        .await
        .ok()?
        .json::<PackageJSON>()
        .await
        .ok()?;

        let (path, _) = get_path_and_filename(&file.path);

        let icon_file = files.iter().cloned().find(|file| {
          file.path
            == Path::new(path)
              .join(package_json.icon.clone())
              .into_os_string()
              .into_string()
              .unwrap()
        })?;

        Some((path == "", icon_file))
      }),
  )
  .await
  .into_iter()
  .filter_map(|icon_file| icon_file)
  .collect();

  if package_json_icons.len() > 0 {
    let first = package_json_icons[0].1.clone();

    Some(
      package_json_icons
        .into_iter()
        .find_map(|(is_root, file)| if is_root { Some(file) } else { None })
        .unwrap_or(first),
    )
  } else {
    None
  }
}

pub async fn get_blob(owner: &str, repo: &str) -> Result<Option<(bool, RepoBlob)>, Box<dyn Error>> {
  let (commit_sha, files) = get_repo_files(owner, repo).await?;

  let result = if let Some(result) = get_package_json_icon(owner, repo, &commit_sha, &files).await {
    Some((true, result))
  } else {
    let mut results = files
      .into_iter()
      .filter(|file| is_valid_blob(file))
      .map(|file| {
        let weight = get_weight(owner, repo, &file);
        (file, weight)
      })
      .filter(|(_, weight)| *weight > 0)
      .collect::<Vec<_>>();

    results.sort_by(|(_, a_weight), (_, b_weight)| b_weight.cmp(&a_weight));

    results.get(0).cloned().map(|(first_file, first_weight)| {
      let final_results = results
        .into_iter()
        .filter_map(|(file, other_weight)| (first_weight == other_weight).then_some(file))
        .collect::<Vec<_>>();

      let file = final_results
        .iter()
        .cloned()
        .find(|file| file.path.ends_with(".svg"))
        .or_else(|| {
          let mut sizes = final_results
            .iter()
            .filter_map(|file| get_filename_size(file).map(|size| (size, file)))
            .collect::<Vec<_>>();

          sizes.sort_by(|(a_size, _), (b_size, _)| b_size.cmp(a_size));

          sizes.get(0).map(|(_, file)| *file).cloned()
        })
        .or_else(|| {
          final_results
            .into_iter()
            .find(|file| file.path.ends_with(".png"))
        })
        .unwrap_or(first_file);

      (false, file)
    })
  };

  Ok(result.map(|(is_package_json, file)| {
    (
      is_package_json,
      RepoBlob {
        owner: owner.to_string(),
        repo: repo.to_string(),
        commit_sha,

        sha: file.sha,
        path: file.path,
      },
    )
  }))
}

fn get_path_and_filename(fullpath: &str) -> (&str, &str) {
  fullpath.rsplit_once('/').unwrap_or(("", &fullpath))
}

fn get_filename_size(file: &File) -> Option<u64> {
  regex!(".*([\\d.]+)x([\\d.]+)(?:@([\\d.]+)x)")
    .captures(&file.path.to_lowercase())
    .unwrap()
    .map(|result| {
      let width = (result[2].parse::<f64>().unwrap()) as u64;
      let height = (result[3].parse::<f64>().unwrap()) as u64;
      let scale = result
        .get(4)
        .map(|times| times.as_str().parse::<f64>().unwrap() as u64)
        .unwrap_or(1);

      width * height * scale
    })
}
