#![feature(async_closure)]
use console_error_panic_hook::set_once;
use repo_icons::{IconInfo, Readme, RepoIcons};
use serde::Serialize;
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
  set_once();

  let router = Router::new();

  for (key, token) in req.url()?.query_pairs() {
    if key == "token" {
      repo_icons::set_token(token)
    }
  }

  router
    .get_async("/:owner/:repo", async move |_, ctx| {
      let owner = ctx.param("owner").ok_or("expected owner")?.as_str();
      let repo = ctx.param("repo").ok_or("expected repo")?.as_str();

      let repo_icons = RepoIcons::load(owner, repo)
        .await
        .map_err(|error| Error::Json((format!("{}", error).into(), 500)))?;

      let repo_icon = repo_icons.closest_match();

      let mut headers = Headers::new();
      headers.set("User-Agent", "repo-icons-worker")?;
      for (header_name, header_value) in &repo_icon.headers {
        headers.set(header_name, header_value)?;
      }

      let request = Request::new_with_init(
        &repo_icon.url.to_string(),
        RequestInit::new().with_headers(headers),
      )?;

      let mut response = Fetch::Request(request).send().await?;
      response.headers_mut().set(
        "Content-Type",
        match repo_icon.info {
          IconInfo::PNG { .. } => "image/png",
          IconInfo::JPEG { .. } => "image/jpeg",
          IconInfo::ICO { .. } => "image/x-icon",
          IconInfo::SVG => "image/svg+xml",
        },
      )?;

      Ok(response)
    })
    .get_async("/:owner/:repo/icons", async move |_, ctx| {
      let owner = ctx.param("owner").ok_or("expected owner")?.as_str();
      let repo = ctx.param("repo").ok_or("expected repo")?.as_str();

      let repo_icons = RepoIcons::load(owner, repo)
        .await
        .map_err(|error| Error::Json((format!("{}", error).into(), 500)))?;

      from_json_pretty(&repo_icons)
    })
    .get_async("/:owner/:repo/images", async move |_, ctx| {
      let owner = ctx.param("owner").ok_or("expected owner")?.as_str();
      let repo = ctx.param("repo").ok_or("expected repo")?.as_str();

      let readme = Readme::load(owner, repo)
        .await
        .map_err(|error| Error::Json((format!("{}", error).into(), 500)))?;
      let images = readme.images().await;

      from_json_pretty(&images)
    })
    .run(req, env)
    .await
}

fn from_json_pretty<B: Serialize>(value: &B) -> Result<Response> {
  if let Ok(data) = serde_json::to_string_pretty(value) {
    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json")?;

    Response::from_body(ResponseBody::Body(data.into_bytes())).map(|res| res.with_headers(headers))
  } else {
    Err(Error::Json(("Failed to encode data to json".into(), 500)))
  }
}
