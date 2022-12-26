#![feature(async_closure, let_chains)]
mod npm_github;
mod serialized_response;
mod transform_response;

use console_error_panic_hook::set_once;
use repo_icons::{IconInfo, Readme, RepoIconKind, RepoIcons};
use serde::Serialize;
use serialized_response::{serialize_json, SerializedResponse};
use transform_response::*;
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
pub async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
  let mut response = modifiable_response(request(req, env, ctx).await?)?;

  let headers = response.headers_mut();
  headers.set("Access-Control-Allow-Origin", "*")?;

  Ok(response)
}

async fn request(req: Request, env: Env, ctx: Context) -> Result<Response> {
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

  let token = url.query_pairs().find_map(|(key, token)| {
    if key == "token" {
      Some(token.to_string())
    } else {
      None
    }
  });

  repo_icons::set_token(
    token.as_ref().or(
      env
        .secret("GITHUB_TOKEN")
        .ok()
        .map(|token| token.to_string())
        .as_ref(),
    ),
  );

  let cache = Cache::default();
  let cache_kv = env.kv("CACHE")?;

  url.set_query(None);
  let cache_key = url.to_string();

  if !refetch {
    if let Some(mut res) = cache.get(&cache_key, false).await? {
      console_log!("from HTTP cache");

      if let Some(token) = &token {
        res = transform_response(true, token, res).await?
      }

      return Ok(res);
    };

    if let Some(res) = cache_kv.get(&cache_key).text().await? {
      console_log!("from KV cache");

      let mut res = SerializedResponse::deserialize(res)?;

      if let Some(token) = &token {
        res = transform_response(true, token, res).await?
      }

      return Ok(res);
    };

    console_log!("missed cache");
  } else {
    console_log!("force refetching");
  }

  let router = Router::new();

  let npm_handler = async move |req: Request, ctx: RouteContext<()>| {
    let org_or_package = ctx.param("org_or_package").unwrap().clone();

    let package_name = if let Some(package) = ctx.param("package") {
      format!("{}/{}", org_or_package, package)
    } else {
      org_or_package
    };

    let mut slug = match npm_github::get_slug(&package_name).await {
      Ok(slug) => slug,
      Err(err) => return Response::error(err.to_string(), 404),
    };

    // if it starts with a reserved name,
    // then prefix it with @
    if slug.starts_with("npm/") {
      slug = format!("@{}", slug);
    }

    let mut url = req.url()?;
    url.set_path(&format!("/{}", slug));

    let mut response = modifiable_response(Response::redirect_with_status(url, 301)?)?;

    let headers = response.headers_mut();
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Cache-Control", "public, max-age=259200")?;

    Ok(response)
  };

  let mut response = router
    .get("/", move |req, _| redirect_to_www(&req, true))
    .get_async("/npm/:org_or_package/:package", npm_handler)
    .get_async("/npm/:org_or_package", npm_handler)
    .get_async("/:owner/:repo", async move |req, ctx| {
      if is_navigate(&req) {
        return redirect_to_www(&req, false);
      }

      let mut write_to_cache = true;
      let owner = ctx.param("owner").unwrap().trim_start_matches("@");
      let repo = ctx.param("repo").unwrap().as_str();

      let repo_icon = match RepoIcons::load(owner, repo, true).await {
        Err(err) => {
          console_error!("{}", err);
          return Response::error(format!("repo not found: {:?}", err), 404);
        }
        Ok(icons) => icons.into_best_match(),
      };

      if matches!(
        repo_icon.kind,
        RepoIconKind::OrgAvatar | RepoIconKind::UserAvatarFallback
      ) {
        write_to_cache = false;
      }

      let mut headers = Headers::new();
      headers.set("User-Agent", "github-icons-worker")?;
      for (header_name, header_value) in &repo_icon.headers {
        headers.set(header_name, header_value)?;
      }

      let bytes = match repo_icon.data().await {
        Ok(bytes) => bytes,
        Err(err) => return Response::error(err.to_string(), 404),
      };

      let mut res = Response::from_bytes(bytes)?;

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
          IconInfo::GIF { .. } => "image/gif",
          IconInfo::SVG => "image/svg+xml",
        },
      )?;

      Ok(res)
    })
    .get_async("/:owner/:repo/all", async move |_, ctx| {
      let owner = ctx.param("owner").unwrap().trim_start_matches("@");
      let repo = ctx.param("repo").unwrap().as_str();

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
      let owner = ctx.param("owner").unwrap().trim_start_matches("@");
      let repo = ctx.param("repo").unwrap().as_str();

      let images = match Readme::load(owner, repo).await {
        Ok(readme) => match readme.load_body().await {
          Some(images) => images,
          None => return Response::error("repo not found", 404),
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

    if let Some(token) = &token {
      response = transform_response(false, token, response).await?
    }

    ctx.wait_until(async move {
      if response.status_code() > 400 {
        let _ = cache.delete(&cache_key, false).await;
      } else if response.headers().has("Cache-Control").unwrap() {
        console_log!("caching as {}", cache_key);
        let serialized_response = SerializedResponse::serialize(response.cloned().unwrap())
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
  let bytes = serialize_json(value)?;

  let mut headers = Headers::new();
  headers.set("Content-Type", "application/json")?;

  Response::from_body(ResponseBody::Body(bytes)).map(|res| res.with_headers(headers))
}
