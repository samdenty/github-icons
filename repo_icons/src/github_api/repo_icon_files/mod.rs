mod repo_files;

use crate::RepoFile;
use fancy_regex::{escape, Regex};
use futures::future::join_all;
use itertools::Itertools;
use repo_files::{get_repo_files, File, FileType};
use std::convert::TryInto;
use std::error::Error;
use std::path::Path;
use vec1::Vec1;

const OWNER_PREFIXES: [&str; 1] = ["get"];
const OWNER_SUFFIXES: [&str; 7] = ["js", "rs", "io", "land", "pkg", "hq", "app"];

pub(crate) fn stripped_owner_lowercase(owner: &str) -> String {
  let mut owner = &owner.to_lowercase()[..];

  for prefix in OWNER_PREFIXES {
    owner = owner.strip_prefix(prefix).unwrap_or(owner);
  }

  // find the first non-empty segment
  for segment in owner.split('-') {
    if segment.len() > 0 {
      owner = segment;
      break;
    }
  }

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
  let owner = stripped_owner_lowercase(owner);
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

  let ignore_paths = regex!(
    "(e2e|fixtures|demo|deps|workspaces?|examples?|third[-_]party|manual|extensions|themes|tests?)/"
  );
  if !ignore_paths.is_match(&fullpath).unwrap() {
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

pub async fn get_repo_icon_files(
  owner: &str,
  repo: &str,
) -> Result<Option<(bool, Vec1<RepoFile>)>, Box<dyn Error>> {
  let (commit_sha, files) = get_repo_files(owner, repo).await?;

  let result = if let Some(result) = get_package_json_icon(owner, repo, &commit_sha, &files).await {
    Some((true, Vec1::new(result)))
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

    results.get(0).cloned().map(|(_, first_weight)| {
      let final_results = results
        .into_iter()
        .filter_map(|(file, other_weight)| (first_weight == other_weight).then_some(file))
        .unique_by(|file| file.sha.clone())
        .collect::<Vec<_>>();

      (false, final_results.try_into().unwrap())
    })
  };

  Ok(result.map(|(is_package_json, files)| {
    (
      is_package_json,
      files
        .into_iter()
        .map(|file| RepoFile {
          github: format!("{}/{}", owner, repo),
          commit_sha: commit_sha.clone(),

          sha: file.sha,
          path: file.path,
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap(),
    )
  }))
}

fn get_path_and_filename(fullpath: &str) -> (&str, &str) {
  fullpath.rsplit_once('/').unwrap_or(("", &fullpath))
}
