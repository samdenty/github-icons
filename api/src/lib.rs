#![feature(async_closure)]
use console_error_panic_hook::set_once;
use repo_icons::{IconInfo, Readme, RepoIcons};
use serde::Serialize;
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, ctx: worker::Context) -> Result<Response> {
  set_once();

  let mut url = req.url()?;

  let token = url
    .query_pairs()
    .find_map(|(key, token)| if key == "token" { Some(token) } else { None });
  repo_icons::set_token(token);

  let cache = Cache::default();

  if !url.path().ends_with("/all") {
    url.set_query(None);
  }
  let url = url.to_string();

  if let Some(res) = cache.get(&url, false).await? {
    return Ok(res);
  };

  let router = Router::new();

  let mut response = router
    .get_async("/:owner/:repo", async move |_, ctx| {
      let owner = ctx.param("owner").ok_or("expected owner")?.as_str();
      let repo = ctx.param("repo").ok_or("expected repo")?.as_str();

      let repo_icons = match RepoIcons::load(owner, repo, true).await {
        Ok(repo_icons) => repo_icons,
        Err(err) => return Response::error(err.to_string(), 404),
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

      let mut res = match Fetch::Request(request).send().await {
        Ok(mut response) => response.cloned()?,
        Err(err) => return Response::error(err.to_string(), 404),
      };

      let headers = res.headers_mut();
      headers.set("Cache-Control", "public, max-age=259200")?;
      headers.set(
        "Content-Type",
        match repo_icon.info {
          IconInfo::PNG { .. } => "image/png",
          IconInfo::JPEG { .. } => "image/jpeg",
          IconInfo::ICO { .. } => "image/x-icon",
          IconInfo::SVG => "image/svg+xml",
        },
      )?;

      Ok(res)
    })
    .get_async("/:owner/:repo/all", async move |_, ctx| {
      let owner = ctx.param("owner").ok_or("expected owner")?.as_str();
      let repo = ctx.param("repo").ok_or("expected repo")?.as_str();

      let repo_icons = match RepoIcons::load(owner, repo, false).await {
        Ok(repo_icons) => repo_icons,
        Err(err) => return Response::error(err.to_string(), 404),
      };

      let mut response = from_json_pretty(&repo_icons)?;

      response
        .headers_mut()
        .set("Cache-Control", "public, max-age=259200")?;

      Ok(response)
    })
    .get_async("/:owner/:repo/images", async move |_, ctx| {
      let owner = ctx.param("owner").ok_or("expected owner")?.as_str();
      let repo = ctx.param("repo").ok_or("expected repo")?.as_str();

      let images = match Readme::load(owner, repo).await {
        Ok(readme) => match readme.load_body().await {
          Ok(images) => images,
          Err(err) => return Response::error(err.to_string(), 404),
        },
        Err(err) => return Response::error(err.to_string(), 404),
      };

      let mut response = from_json_pretty(&images)?;

      response
        .headers_mut()
        .set("Cache-Control", "public, max-age=259200")?;

      Ok(response)
    })
    .run(req.clone()?, env)
    .await?;

  {
    let response = response.cloned()?;
    ctx.wait_until(async move {
      if response.status_code() == 404 {
        let _ = cache.delete(&url, false).await;
      } else {
        let _ = cache.put(&url, response).await;
      }
    });
  }

  Ok(response)
}

fn from_json_pretty<B: Serialize>(value: &B) -> Result<Response> {
  if let Ok(data) = serde_json::to_string_pretty(value) {
    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json")?;

    Response::from_body(ResponseBody::Body(data.into_bytes())).map(|res| res.with_headers(headers))
  } else {
    Err(Error::Json(("Failed to encode data to json".into(), 404)))
  }
}
