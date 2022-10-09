use serde::Serialize;

use super::database::schema::{icons, repos};

#[derive(Queryable, Insertable, Debug, Serialize)]
pub struct Repo {
  pub owner: String,
  pub repo: String,
  pub path: String,
  pub icon_path: Option<String>,
}

#[derive(Queryable, Insertable, Debug)]
pub struct Icon {
  pub owner: String,
  pub repo: String,
  pub path: String,
}
