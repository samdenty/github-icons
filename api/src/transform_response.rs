use crate::serialized_response::serialize_json;
use repo_icons::RepoIcons;
use worker::*;

pub async fn transform_response(
  include_token: bool,
  token: &str,
  mut response: Response,
) -> Result<Response> {
  if response.headers().get("Content-Type").unwrap() != Some("application/json".to_string()) {
    return Ok(response);
  }

  let bytes = response.bytes().await?;

  let bytes = match serde_json::from_slice::<RepoIcons>(&bytes) {
    Ok(mut repo_icons) => {
      for repo_icon in repo_icons.iter_mut() {
        if let Some(auth) = repo_icon.headers.get_mut("Authorization") {
          *auth = if include_token {
            auth.replace("$GITHUB_TOKEN", token)
          } else {
            auth.replace(token, "$GITHUB_TOKEN")
          }
        }
      }

      serialize_json(&repo_icons)?
    }
    Err(_) => bytes,
  };

  Ok(
    Response::from_bytes(bytes)?
      .with_status(response.status_code())
      .with_headers(response.headers().clone()),
  )
}
