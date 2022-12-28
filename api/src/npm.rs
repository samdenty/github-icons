use regex::Regex;
use serde_json::Value;
use std::error::Error;
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
      "https://nodejs.org/static/images/logos/nodejs-dark.eps"
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

  let repository_url = match repository {
    Value::String(s) => &s,
    Value::Object(o) => o["url"].as_str().unwrap(),
    Value::Null => return Err(format!("no repository field specified").into()),
    value => return Err(format!("unexpected npm response {:?}", value).into()),
  };

  let re = Regex::new(r"github(\.com)?[/:]([^/]+/[^/]+)").unwrap();
  let captures = re
    .captures(repository_url)
    .ok_or("not a github repository")?;

  let slug = captures[2].trim_end_matches(".git").to_lowercase();
  Ok(slug)
}
