mod repo_files;

use crate::RepoBlob;
use fancy_regex::{escape, Regex};
use futures::future::join_all;
use repo_files::{get_repo_files, File, FileType};
use std::error::Error;
use std::path::Path;

fn is_valid_blob(file: &File) -> bool {
  matches!(file.r#type, FileType::Blob)
    && (file.path.ends_with(".png")
      || file.path.ends_with(".jpg")
      || file.path.ends_with(".jpeg")
      || file.path.ends_with(".ico")
      || file.path.ends_with(".svg"))
}

fn get_weight(owner: &str, repo: &str, file: &File) -> u8 {
  let owner = owner.to_lowercase();
  let owner = owner.rsplit_once('-').unwrap_or((&owner, "")).0;
  let repo = repo.to_lowercase();

  let fullpath = file.path.to_lowercase();
  let (path, filename) = get_path_and_filename(&fullpath);

  let mut weight = 0;
  let mut matches_icon = false;

  if filename.contains(owner) {
    weight += 1;
    matches_icon = true;
  }

  if filename.contains(&repo) {
    weight += 2;
    matches_icon = true;
  }

  let exactly_repo_name =
    Regex::new(&format!("^{}(?:[-_]logo.*)?\\.[^.]+$", escape(&repo))).unwrap();
  if exactly_repo_name.is_match(&filename).unwrap() {
    weight += 1;
  }

  let fixtures = regex!("(e2e|fixtures|third[-_]party|test(s)?)/");
  if !fixtures.is_match(&fullpath).unwrap() {
    if filename.contains("favicon") {
      weight += 2;
      matches_icon = true;
    }

    let logo = regex!("logo(?!ut|n|s)");
    if logo.is_match(&fullpath).unwrap() {
      weight += 1;
      matches_icon = true;

      let exactly_logo = regex!("^logo\\.[^.]+$");
      if exactly_logo.is_match(&filename).unwrap() {
        weight += 2;
      }
    }

    if matches_icon {
      let public = regex!("(public|static|resources|assets|www)/");
      if public.is_match(&fullpath).unwrap() {
        weight += 1;
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

    results.get(0).cloned().map(|(file, weight)| {
      let final_results = results
        .into_iter()
        .filter(|(_, other_weight)| weight == *other_weight)
        .collect::<Vec<_>>();

      let (file, _) = final_results
        .iter()
        .cloned()
        .find(|(file, _)| file.path.ends_with(".svg"))
        .or_else(|| {
          final_results
            .into_iter()
            .find(|(file, _)| file.path.ends_with(".png"))
        })
        .unwrap_or((file, weight));

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
