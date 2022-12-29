use futures::{future::select_all, Future, FutureExt};
use regex::Regex;
use serde_json::Value;
use std::{error::Error, pin::Pin};
use worker::{Fetch, Url};

const NODEJS_BUILTINS: [&str; 41] = [
  "assert",
  "async_hooks",
  "buffer",
  "child_process",
  "cluster",
  "console",
  "constants",
  "crypto",
  "dgram",
  "diagnostics_channel",
  "dns",
  "domain",
  "events",
  "fs",
  "http",
  "http2",
  "https",
  "inspector",
  "module",
  "net",
  "os",
  "path",
  "perf_hooks",
  "process",
  "punycode",
  "querystring",
  "readline",
  "repl",
  "stream",
  "string_decoder",
  "timers",
  "tls",
  "trace_events",
  "tty",
  "url",
  "util",
  "v8",
  "vm",
  "wasi",
  "worker_threads",
  "zlib",
];

pub async fn get_redirect_url(mut url: Url, package_name: &str) -> Result<Url, Box<dyn Error>> {
  if package_name.starts_with("node:") || NODEJS_BUILTINS.contains(&package_name) {
    return Ok(
      "https://nodejs.org/static/images/logos/js-green.svg"
        .parse()
        .unwrap(),
    );
  }

  let mut slug = get_slug(&package_name).await?;

  // if it starts with a reserved name,
  // then prefix it with @
  if slug.starts_with("npm/") {
    slug = format!("@{}", slug);
  }

  url.set_path(&format!("/{}", slug));

  Ok(url)
}

pub async fn get_slug(package_name: &str) -> Result<String, Box<dyn Error>> {
  let mut futures: Vec<Pin<Box<dyn Future<Output = Result<String, Box<dyn Error>>>>>> = vec![
    npm_resolver(package_name).boxed_local(),
    npms_resolver(package_name).boxed_local(),
  ];

  loop {
    let (repository_url, index, _) = select_all(&mut futures).await;
    futures.remove(index);

    if let Ok(repository_url) = repository_url {
      let re = Regex::new(r"github(\.com)?[/:]([^/]+/[^/]+)").unwrap();
      let captures = re
        .captures(&repository_url)
        .ok_or("not a github repository")?;

      let slug = captures[2].trim_end_matches(".git").to_lowercase();
      return Ok(slug);
    }

    if futures.is_empty() {
      return Err(repository_url.unwrap_err());
    }
  }
}

async fn npms_resolver(package_name: &str) -> Result<String, Box<dyn Error>> {
  let resp: Value = Fetch::Url(
    format!("https://api.npms.io/v2/package/{}", package_name)
      .parse()
      .unwrap(),
  )
  .send()
  .await?
  .json()
  .await?;

  if let Value::String(message) = &resp["message"] {
    return Err(message.clone().into());
  }

  resp["collected"]["metadata"]["links"]["repository"]
    .as_str()
    .map(|repo| repo.to_string())
    .ok_or("no repository field specified".into())
}

async fn npm_resolver(package_name: &str) -> Result<String, Box<dyn Error>> {
  let resp: Value = Fetch::Url(
    format!("https://registry.npmjs.org/{}/latest", package_name)
      .parse()
      .unwrap(),
  )
  .send()
  .await?
  .json()
  .await?;

  let repository = &resp["repository"];

  match repository {
    Value::String(s) => Ok(s.to_string()),
    Value::Object(o) => Ok(o["url"].as_str().unwrap().to_string()),
    Value::Null => Err(format!("no repository field specified").into()),
    value => Err(format!("unexpected npm response {:?}", value).into()),
  }
}
