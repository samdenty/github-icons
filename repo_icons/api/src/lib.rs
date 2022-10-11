#![feature(async_closure)]
use console_error_panic_hook::set_once;
use repo_icons::{IconInfo, Readme, RepoIcons};
use serde::Serialize;
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
  set_once();

  for (key, token) in req.url()?.query_pairs() {
    if key == "token" {
      repo_icons::set_token(token)
    }
  }

  let router = Router::new();

  router
    .get_async("/:owner/:repo", async move |_, ctx| {
      let owner = ctx.param("owner").ok_or("expected owner")?.as_str();
      let repo = ctx.param("repo").ok_or("expected repo")?.as_str();

      let repo_icons = match RepoIcons::load(owner, repo).await {
        Ok(repo_icons) => repo_icons,
        Err(err) => return Response::error(err.to_string(), 500),
      };

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

      let mut response = match Fetch::Request(request).send().await {
        Ok(mut response) => response.cloned()?,
        Err(err) => return Response::error(err.to_string(), 404),
      };

      let headers = response.headers_mut();
      headers.set("Cache-Control", "public, max-age=43200, immutable")?;
      headers.set(
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
    .get_async("/:owner/:repo/all", async move |_, ctx| {
      let owner = ctx.param("owner").ok_or("expected owner")?.as_str();
      let repo = ctx.param("repo").ok_or("expected repo")?.as_str();

      let repo_icons = match RepoIcons::load(owner, repo).await {
        Ok(repo_icons) => repo_icons,
        Err(err) => return Response::error(err.to_string(), 404),
      };

      let mut response = from_json_pretty(&repo_icons)?;

      response
        .headers_mut()
        .set("Cache-Control", "public, max-age=43200, immutable")?;

      Ok(response)
    })
    .get_async("/:owner/:repo/images", async move |_, ctx| {
      let owner = ctx.param("owner").ok_or("expected owner")?.as_str();
      let repo = ctx.param("repo").ok_or("expected repo")?.as_str();

      let images = match Readme::load(owner, repo).await {
        Ok(readme) => readme.images().await,
        Err(err) => return Response::error(err.to_string(), 404),
      };

      let mut response = from_json_pretty(&images)?;

      response
        .headers_mut()
        .set("Cache-Control", "public, max-age=43200, immutable")?;

      Ok(response)
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
