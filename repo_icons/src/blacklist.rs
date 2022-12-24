use fancy_regex::Regex;
use once_cell::sync::Lazy;
use url::Url;

// Domains which serve badges without 'badge' in the URL
static BADGE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
  regexes![
    r"badge",
    r"widget",
    r"license",
    r"embed",
    r"button",
    r"build",
    r"status",
    r"coverage",
    r"ukraine",
    r"shields.io",
    r"travis-ci",
    "nodei.co",
    r"herokucdn.com",
    r"appveyor.com",
    r"david-dm.org",
    r"codeclimate.com",
    r"codecov.io",
    r"buymeacoffee.com",
    r"snyk.io",
    r"deploy.workers.cloudflare.com",
    r"circleci.com",
    r"dev.azure.com",
    r"deps.rs",
    r"docs.rs",
    // with paths
    r"^github.com/[^/]+/[^/]+/workflows",
    r"liberapay.com/assets/widgets",
    r"herokucdn.com",
    r"codesandbox.io/static/img",
    r"developer.stackblitz.com/img",
    r"asciinema.org"
  ]
  .to_vec()
});

static BLACKLISTED_HOMEPAGES: Lazy<Vec<Regex>> = Lazy::new(|| {
  regexes![
    r"^stackblitz.com/edit",
    r"^pr.new",
    r"^codesandbox.io/s/",
    r"^[^/]*npm[^/*]*",
    r"^hex.pm",
    r"^crates.io",
    r"^docs.rs",
    r"git.io",
    r"^github.com",
    r"^chrome.google.com/webstore",
    r"^marketplace.visualstudio.com"
  ]
  .to_vec()
});

pub fn is_badge_url(url: &Url) -> bool {
  let domain = if let Some(domain) = url.domain() {
    domain
  } else {
    return false;
  };
  let url = format!("{}{}", domain, url.path());

  is_badge_text(&url)
}

pub fn is_badge_text(string: &str) -> bool {
  let string = string.to_lowercase();

  BADGE_PATTERNS
    .iter()
    .any(|url_regex| url_regex.is_match(&string).unwrap())
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
    .any(|url_regex| url_regex.is_match(&url).unwrap())
}
