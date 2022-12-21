use regex::Regex;
use serde_json::Value;
use std::error::Error;
use worker::Fetch;

pub async fn get_slug(package_name: &str) -> Result<String, Box<dyn Error>> {
  let resp: Value = Fetch::Url(
    format!("https://registry.npmjs.org/{}", package_name)
      .parse()
      .unwrap(),
  )
  .send()
  .await?
  .json()
  .await?;

  let repository = &resp["repository"];

  let repository_url = match repository {
    Value::String(s) => &s,
    Value::Object(o) => o["url"].as_str().unwrap(),
    value => return Err(format!("unexpected npm response {:?}", value).into()),
  };

  let re = Regex::new(r"github(\.com)?[/:]([^/]+/[^/]+)").unwrap();
  let captures = re
    .captures(repository_url)
    .ok_or("not a github repository")?;

  let slug = captures[2].trim_end_matches(".git").to_lowercase();
  Ok(slug)
}
