use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

// Domains which serve badges without 'badge' in the URL
static BADGE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
  regexes![
    r"badge",
    r"status",
    r"coverage",
    r"^img.shields.io",
    r"^travis-ci.org",
    r"^api.travis-ci.com",
    r"^ci.appveyor.com",
    r"^david-dm.org",
    r"^api.codeclimate.com",
    r"^codecov.io",
    r"^snyk.io",
    r"^deploy.workers.cloudflare.com",
    r"^circleci.com",
    r"^dev.azure.com",
    r"^deps.rs",
    r"^docs.rs",
    // with paths
    r"^github.com/[^/]+/[^/]+/workflows",
    r"^liberapay.com/assets/widgets",
    r"^www.herokucdn.com/deploy/button.png",
    r"^vercel.com/button",
    r"^codesandbox.io/static/img/play-codesandbox.svg",
  ]
  .to_vec()
});

static BLACKLISTED_HOMEPAGES: Lazy<Vec<Regex>> = Lazy::new(|| {
  regexes![
    r"^codesandbox.io/s/",
    r"^[^/]*npm[^/*]*",
    r"^crates.io",
    r"^docs.rs",
    r"^chrome.google.com/webstore"
  ]
  .to_vec()
});

pub fn is_badge(url: &Url) -> bool {
  let domain = if let Some(domain) = url.domain() {
    domain
  } else {
    return false;
  };
  let url = format!("{}{}", domain, url.path());

  BADGE_PATTERNS
    .iter()
    .any(|url_regex| url_regex.is_match(&url))
}

pub fn is_blacklisted_homepage(url: &Url) -> bool {
  let domain = if let Some(domain) = url.domain() {
    domain
  } else {
    return false;
  };
  let url = format!("{}{}", domain, url.path());

  BLACKLISTED_HOMEPAGES
    .iter()
    .any(|url_regex| url_regex.is_match(&url))
}
