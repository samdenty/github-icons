use glob::{MatchOptions, Pattern};
use once_cell::sync::Lazy;
use url::Url;

// Domains which serve badges without 'badge' in the URL
static BADGE_PATTERNS: Lazy<Vec<Pattern>> = Lazy::new(|| {
  patterns![
    "img.shields.io/**",
    "travis-ci.org/**",
    "api.travis-ci.com/**",
    "ci.appveyor.com/**",
    "david-dm.org/**",
    "api.codeclimate.com/**",
    "codecov.io/**",
    "snyk.io/**",
    "deploy.workers.cloudflare.com/**",
    "circleci.com/**",
    "dev.azure.com/**",
    "deps.rs/**",
    "docs.rs/**",
    // with paths
    "github.com/*/*/workflows/**",
    "liberapay.com/assets/widgets/**",
    "www.herokucdn.com/deploy/button.png",
    "vercel.com/button",
    "codesandbox.io/static/img/play-codesandbox.svg",
  ]
  .to_vec()
});

static BLACKLISTED_HOMEPAGES: Lazy<Vec<Pattern>> = Lazy::new(|| {
  patterns![
    "codesandbox.io/s/**",
    "npmjs.com/**",
    "chrome.google.com/webstore/**"
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

  if url.contains("badge") || url.contains("status") {
    return true;
  }

  BADGE_PATTERNS.iter().any(|url_pattern| {
    url_pattern.matches_with(
      &url,
      MatchOptions {
        case_sensitive: false,
        require_literal_separator: true,
        require_literal_leading_dot: false,
      },
    )
  })
}

pub fn is_blacklisted_homepage(url: &Url) -> bool {
  let domain = if let Some(domain) = url.domain() {
    domain
  } else {
    return false;
  };
  let url = format!("{}{}", domain, url.path());

  BLACKLISTED_HOMEPAGES.iter().any(|url_pattern| {
    url_pattern.matches_with(
      &url,
      MatchOptions {
        case_sensitive: false,
        require_literal_separator: true,
        require_literal_leading_dot: false,
      },
    )
  })
}
