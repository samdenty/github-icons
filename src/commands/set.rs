use crate::{
  database::{self, db},
  get_slug,
  models::{Icon, Repo},
  sync, write, CACHE_DIR,
};
use diesel::prelude::*;
use rand::Rng;
use std::error::Error;
use tokio::fs;

async fn set_with_repo_path(
  repo_path: &str,
  icon_path: &str,
  overwrite: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
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
        .filter(owner.like(&user).and(repo.like(&repo_name)))
        .first::<Repo>(db())?
        .icon_path
        .is_none()
    {
      diesel::update(
        repos.filter(
          owner
            .like(&user)
            .and(repo.like(&repo_name))
            .and(path.eq(&repo_path)),
        ),
      )
      .set(icon_path.eq(&icon_name))
      .execute(db())?;
    }
  }

  write(&repo_path).await?;

  Ok(())
}

pub async fn set(
  slug_or_path: &str,
  icon_path: &str,
  overwrite: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
  let (user, repo_name, repo_path) = get_slug(slug_or_path)?;

  if repo_path.is_some() {
    set_with_repo_path(slug_or_path, icon_path, overwrite).await?;
  } else {
    let repos = {
      use database::schema::repos::dsl::*;

      repos
        .filter(owner.like(&user).and(repo.like(&repo_name)))
        .load::<Repo>(db())?
    };

    for repo in repos {
      set_with_repo_path(&repo.path, icon_path, overwrite).await?;
    }
  };

  Ok(())
}

pub async fn set_default(slug_or_path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
  let (user, repo_name, repo_path) = get_slug(slug_or_path)?;

  {
    use database::schema::repos::dsl::*;
    if let Some(repo_path) = repo_path {
      diesel::delete(
        repos.filter(
          owner
            .like(&user)
            .and(repo.like(&repo_name))
            .and(path.eq(&repo_path)),
        ),
      )
      .execute(db())?;
    } else {
      diesel::delete(repos.filter(owner.like(&user).and(repo.like(&repo_name)))).execute(db())?;
    };
  }

  sync(slug_or_path).await?;

  Ok(())
}
