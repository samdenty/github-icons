use crate::serialize_json;
use repo_icons::RepoIcons;
use worker::*;

pub enum Transform {
  Serialize,
  Deserialize,
}

pub async fn transform_response(
  transform: Transform,
  token: Option<&String>,
  req: &Request,
  mut response: Response,
) -> Result<Response> {
  if response.status_code() == 301 {
    response = response.cloned()?;
    let headers = response.headers_mut();
    let mut url: Url = headers.get("Location")?.unwrap().parse()?;

    match transform {
      Transform::Serialize => {
        url.set_query(None);
      }
      Transform::Deserialize => {
        let req_url = req.url()?;
        url.set_query(req_url.query());
      }
    }

    headers.set("Location", &url.to_string())?;
  }

  if response
    .headers()
    .get("Content-Type")
    .unwrap()
    .is_some_and(|content_type| content_type == "application/json")
  {
    if let Some(token) = token {
      let bytes = response.bytes().await?;

      let bytes = match serde_json::from_slice::<RepoIcons>(&bytes) {
        Ok(mut repo_icons) => {
          for repo_icon in repo_icons.iter_mut() {
            if let Some(auth) = repo_icon.headers.get_mut("Authorization") {
              *auth = match transform {
                Transform::Deserialize => auth.replace("$GITHUB_TOKEN", token),
                Transform::Serialize => auth.replace(token, "$GITHUB_TOKEN"),
              }
            }
          }

          serialize_json(&repo_icons)?
        }
        Err(_) => bytes,
      };

      response = Response::from_bytes(bytes)?
        .with_status(response.status_code())
        .with_headers(response.headers().clone());
    }
  }

  Ok(response)
}
