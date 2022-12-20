#![feature(async_closure, let_chains)]
mod serialized_response;

use console_error_panic_hook::set_once;
use futures::{future::select_all, FutureExt};
use repo_icons::{IconInfo, Readme, RepoIcon, RepoIconKind, RepoIcons};
use serde::Serialize;
use serialized_response::SerializedResponse;
use worker::*;

fn is_navigate(req: &Request) -> bool {
  // if user navigates to URL directly, then redirect them to www.
  if let Some(fetch_mode) = req.headers().get("Sec-Fetch-Mode").unwrap() {
    if fetch_mode == "navigate" {
      return true;
    }
  }

  false
}

fn redirect_to_www(req: &Request, permanent: bool) -> Result<Response> {
  let mut url = req.url()?;

  url.set_host(
    url
      .host_str()
      .map(|host| format!("www.{}", host))
      .as_deref(),
  )?;

  Response::redirect_with_status(url, if permanent { 301 } else { 302 })
}

fn modifiable_response(response: Response) -> Result<Response> {
  Ok(
    Response::from_body(response.body().clone())?
      .with_status(response.status_code())
      .with_headers(response.headers().clone()),
  )
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, ctx: worker::Context) -> Result<Response> {
  let mut response = modifiable_response(request(req, env, ctx).await?)?;

  let headers = response.headers_mut();
  headers.set("Access-Control-Allow-Origin", "*")?;

  Ok(response)
}

async fn request(req: Request, env: Env, ctx: worker::Context) -> Result<Response> {
  set_once();

  let mut url = req.url()?;

  let lowercase_path = url.path().to_lowercase();
  if lowercase_path != url.path() {
    url.set_path(&lowercase_path);
    return Response::redirect_with_status(url, 301);
  }

  let json_req = lowercase_path.ends_with("/all") || lowercase_path.ends_with("/images");

  let refetch = url
    .query_pairs()
    .any(|(key, _)| key == "refetch" || key == "force" || key == "refresh")
    || (!json_req && is_navigate(&req));

  let token = url
    .query_pairs()
    .find_map(|(key, token)| if key == "token" { Some(token) } else { None });
  if json_req || token.is_some() {
    repo_icons::set_token(token);
  }

  // default to the default token
  if repo_icons::get_token().is_none() {
    repo_icons::set_token(env.secret("GITHUB_TOKEN").ok());
  }

  let cache = Cache::default();
  let cache_kv = env.kv("CACHE")?;

  // clear the token from the URL
  if !lowercase_path.ends_with("/all") {
    url.set_query(None);
  }
  let cache_key = url.to_string();

  if !refetch {
    if let Some(res) = cache.get(&cache_key, false).await? {
      return Ok(res);
    };

    if let Some(res) = cache_kv.get(&cache_key).text().await? {
      return Ok(SerializedResponse::deserialize(res)?);
    };
  }

  let router = Router::new();

  let mut response = router
    .get("/", move |req, _| redirect_to_www(&req, true))
    .get_async("/:owner/:repo", async move |req, ctx| {
      if is_navigate(&req) {
        return redirect_to_www(&req, false);
      }

      let mut write_to_cache = true;
      let owner = ctx.param("owner").ok_or("expected owner")?.as_str();
      let repo = ctx.param("repo").ok_or("expected repo")?.as_str();

      let user_avatar = RepoIcon::load_avatar(owner).shared();

      let mut futures = vec![
        async {
          RepoIcons::load(owner, repo, true)
            .await
            .map(|icons| icons.into_best_match())
            .ok()
        }
        .boxed_local(),
        user_avatar.clone().boxed_local(),
      ];

      let mut repo_icon = None;

      while !futures.is_empty() {
        let (icon, index, _) = select_all(&mut futures).await;
        futures.remove(index);

        if let Some(icon) = icon && !matches!(icon.kind, RepoIconKind::UserAvatar) {
          repo_icon = Some(icon);
        }
      }

      if repo_icon.is_none() {
        write_to_cache = false;
        repo_icon = user_avatar.await;
      }

      if repo_icon.is_none() {
        return Response::error("repo not found", 404);
      }

      let repo_icon = repo_icon.unwrap();

      let mut headers = Headers::new();
      headers.set("User-Agent", "github-icons-worker")?;
      for (header_name, header_value) in &repo_icon.headers {
        headers.set(header_name, header_value)?;
      }

      let request = Request::new_with_init(
        &repo_icon.url.to_string(),
        RequestInit::new().with_headers(headers),
      )?;

      let mut res = match Fetch::Request(request).send().await {
        Ok(response) => Response::from_body(response.body().clone())?,
        Err(err) => return Response::error(err.to_string(), 404),
      };

      let headers = res.headers_mut();
      if write_to_cache {
        headers.set("Cache-Control", "public, max-age=259200")?;
      }
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

      let headers = response.headers_mut();
      headers.set("Cache-Control", "public, max-age=259200")?;

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

      let headers = response.headers_mut();
      headers.set("Access-Control-Allow-Origin", "*")?;
      headers.set("Cache-Control", "public, max-age=259200")?;

      Ok(response)
    })
    .run(req.clone()?, env)
    .await?;

  {
    let mut response = response.cloned()?;
    ctx.wait_until(async move {
      if response.status_code() == 404 {
        let _ = cache.delete(&cache_key, false).await;
      } else if response.headers().has("Cache-Control").unwrap() {
        let serialized_response = SerializedResponse::from(response.cloned().unwrap())
          .await
          .ok();

        if let Some(put) = serialized_response
          .and_then(|serialized_response| cache_kv.put(&cache_key, serialized_response).ok())
        {
          let _ = put.execute().await;
        }

        let _ = cache.put(cache_key, response).await;
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
