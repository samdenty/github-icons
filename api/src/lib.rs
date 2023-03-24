#![feature(async_closure, let_chains)]
#[macro_use]
extern crate log;

mod npm;
mod transform_response;

use console_error_panic_hook::set_once;
use log::Level;
use repo_icons::{gh_api_get, Readme, RepoIcons};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use transform_response::*;
use worker::*;
use worker_sys::Response as EdgeResponse;

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

fn generate_token(token: &str) -> String {
  let mut hasher = Sha256::new();

  hasher.update("github-icons");
  hasher.update(token);

  let result = hasher.finalize();

  format!("ghi_{}", &format!("{:x}", result)[..36])
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

  console_log::init_with_level(Level::Info).unwrap();

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

  let mut token = url.query_pairs().find_map(|(key, token)| {
    if key == "token" {
      Some(token.to_string())
    } else {
      None
    }
  });

  match token {
    Some(icons_token) if icons_token.starts_with("ghi_") => {
      let tokens = env.kv("TOKENS")?;
      token = tokens.get(&icons_token).text().await?;
    }
    _ => {}
  }

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
  let cache_bucket = env.bucket("CACHE")?;

  url.set_query(None);
  let cache_key = url.path()[1..].to_string();
  let http_cache_key = url.to_string();

  if req.method() == Method::Get {
    if !refetch {
      if let Some(mut res) = cache.get(&http_cache_key, false).await? {
        info!("from HTTP cache");

        if let Some(token) = &token {
          res = transform_response(true, token, res).await?
        }

        return Ok(res);
      };

      if let Some(cache_res) = cache_bucket.get(&cache_key).execute().await? {
        info!("from R2 cache");

        let mut headers = Headers::new();
        let mut status_code = None;

        let metadata = cache_res.http_metadata();

        if let Some(content_type) = metadata.content_type {
          headers.set("Content-Type", &content_type)?;
        }

        if let Some(cache_control) = metadata.cache_control {
          headers.set("Cache-Control", &cache_control)?;
        }

        for (key, value) in &cache_res.custom_metadata()? {
          if key == "status-code" {
            status_code = Some(value.parse().unwrap());
          } else {
            headers.set(key, value)?;
          }
        }

        let mut res = match cache_res.body() {
          Some(body) => Response::from_bytes(body.bytes().await?)?,
          None => Response::from_body(ResponseBody::Empty)?,
        }
        .with_status(status_code.unwrap())
        .with_headers(headers);

        if let Some(token) = &token {
          res = transform_response(true, token, res).await?
        }

        return Ok(res);
      };

      info!("missed cache");
    } else {
      info!("force refetching");
    }
  }

  let router = Router::new();

  let npm_handler = async move |req: Request, ctx: RouteContext<()>| {
    if is_navigate(&req) {
      return redirect_to_www(&req, false);
    }

    let org_or_package = ctx.param("org_or_package").unwrap().clone();

    let package_name = if let Some(package) = ctx.param("package") {
      format!("{}/{}", org_or_package, package)
    } else {
      org_or_package
    };

    let url = req.url()?;
    let is_all = url.path().ends_with("/all") && !package_name.ends_with("/all");

    let url = match npm::get_redirect_url(url, &package_name, is_all).await {
      Ok(url) => url,
      Err(err) => return Response::error(err.to_string(), 404),
    };

    let mut response = modifiable_response(Response::redirect_with_status(url, 301)?)?;

    let headers = response.headers_mut();
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Cache-Control", "public, max-age=259200")?;

    Ok(response)
  };

  let mut response = router
    .get("/", move |req, _| redirect_to_www(&req, true))
    .get("/:owner", move |req, _| redirect_to_www(&req, true))
    .post_async("/token-exchange", async move |req, ctx| {
      let tokens = ctx.kv("TOKENS")?;
      let url = req.url()?;

      let github_token =
        match url
          .query_pairs()
          .find_map(|(name, token)| if name == "token" { Some(token) } else { None })
        {
          Some(token) => token,
          _ => return Response::error("missing token", 401),
        };

      if github_token.starts_with("ghi_")
        || gh_api_get!("user")
          .send()
          .await
          .and_then(|res| res.error_for_status())
          .is_err()
      {
        return Response::error("invalid token", 401);
      };

      let token = generate_token(&github_token);

      tokens.put(&token, &github_token)?.execute().await?;

      return Response::from_bytes(token.as_bytes().to_vec());
    })
    .get_async("/npm/:org_or_package/:package/all", npm_handler)
    .get_async("/npm/:org_or_package/all", npm_handler)
    .get_async("/npm/:org_or_package/:package", npm_handler)
    .get_async("/npm/:org_or_package", npm_handler)
    .get_async("/:owner/:repo", async move |req, ctx| {
      if is_navigate(&req) {
        return redirect_to_www(&req, false);
      }

      let mut write_to_cache = true;
      let owner = ctx.param("owner").unwrap().trim_start_matches("@");
      let repo = ctx.param("repo").unwrap().as_str();

      let result = RepoIcons::load(owner, repo, true).await;

      if let Some(errors) = &result.errors {
        for error in errors {
          error!("{}", error);
        }
        write_to_cache = false;
      }

      let repo_icon = match result.icons {
        Ok(icons) => icons.into_best_match(),
        Err(err) => return Response::error(err, 404),
      };

      let mut headers = Headers::new();
      headers.set("User-Agent", "github-icons")?;
      for (header_name, header_value) in &repo_icon.headers {
        headers.set(header_name, header_value)?;
      }

      let stream = match repo_icon.js_stream().await {
        Ok(stream) => stream,
        Err(err) => return Response::error(err.to_string(), 404),
      };

      let mut res: Response = EdgeResponse::new_with_opt_stream(Some(&stream))?.into();

      let headers = res.headers_mut();

      if write_to_cache {
        headers.set("Cache-Control", "public, max-age=259200")?;
      }

      headers.set("Kind", &repo_icon.kind.to_string())?;

      if let Some(size) = repo_icon.info.size() {
        headers.set("Size", &size.to_string())?;
      }

      headers.set("Content-Type", repo_icon.info.mime_type())?;

      Ok(res)
    })
    .get_async("/:owner/:repo/all", async move |_, ctx| {
      let owner = ctx.param("owner").unwrap().trim_start_matches("@");
      let repo = ctx.param("repo").unwrap().as_str();

      let result = RepoIcons::load(owner, repo, false).await;

      let mut response = from_json_pretty(&result)?;

      let headers = response.headers_mut();
      headers.set("Cache-Control", "public, max-age=259200")?;

      Ok(response)
    })
    .get_async("/:owner/:repo/images", async move |_, ctx| {
      let owner = ctx.param("owner").unwrap().trim_start_matches("@");
      let repo = ctx.param("repo").unwrap().as_str();

      let images = match Readme::load(owner, repo).await {
        Some(images) => images,
        None => return Response::error("repo not found", 404),
      };

      let mut response = from_json_pretty(&images)?;

      let headers = response.headers_mut();
      headers.set("Access-Control-Allow-Origin", "*")?;
      headers.set("Cache-Control", "public, max-age=259200")?;

      Ok(response)
    })
    .run(req.clone()?, env)
    .await?;

  if req.method() == Method::Get {
    let mut response = response.cloned()?;

    if let Some(token) = &token {
      response = transform_response(false, token, response).await?
    }

    ctx.wait_until(async move {
      if response.status_code() > 400 {
        let _ = cache.delete(&http_cache_key, false).await;
      } else if response.headers().has("Cache-Control").unwrap() {
        info!("caching as {}", cache_key);

        let mut metadata = response.headers().into_iter().collect::<HashMap<_, _>>();

        metadata.insert(
          "status-code".to_string(),
          response.status_code().to_string(),
        );

        let content_type = metadata.remove("content-type");
        let cache_control = metadata.remove("cache-control");

        let _ = cache_bucket
          .put(
            &cache_key,
            response.cloned().unwrap().bytes().await.unwrap(),
          )
          .http_metadata(HttpMetadata {
            content_type,
            cache_control,
            content_language: None,
            content_disposition: None,
            content_encoding: None,
            cache_expiry: None,
          })
          .custom_metdata(metadata)
          .execute()
          .await;

        let _ = cache.put(http_cache_key, response).await;
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

pub fn serialize_json<B: Serialize>(value: &B) -> Result<Vec<u8>> {
  match serde_json::to_string_pretty(value) {
    Ok(json) => Ok(json.into_bytes()),
    Err(error) => Err(Error::Json((
      format!("Failed to encode data to json: {:?}", error).into(),
      404,
    ))),
  }
}
