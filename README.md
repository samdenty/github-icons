# repo_icons

[![Crates.io](https://img.shields.io/crates/v/repo_icons.svg)](https://crates.io/crates/repo_icons)
[![Documentation](https://docs.rs/repo_icons/badge.svg)](https://docs.rs/repo_icons/)
![GitHub Sponsors](https://img.shields.io/github/sponsors/samdenty?style=social)

An API / rust library to get icons for any GitHub repo.

Demo

## Features

- Super fast!
- Partially downloads images to find the sizes
- Extracts images from the repo's homepage using [site_icons](https://github.com/samdenty/site_icons)
- Supports WASM (and cloudflare workers)

### Command line usage

```bash
cargo install repo_icons

repo-icons facebook/react
# https://github.githubassets.com/favicons/favicon.svg site_favicon svg
# https://github.githubassets.com/app-icon-512.png app_icon png 512x512
# https://github.githubassets.com/app-icon-192.png app_icon png 192x192
# https://github.githubassets.com/apple-touch-icon-180x180.png app_icon png 180x180
```

### Rust usage

```rust
use repo_icons::Icons;

let icons = Icons::new();
// scrape the icons from a url
icons.load_website("https://github.com").await?;

// fetch all icons, ensuring they exist & determining size
let entries = icons.entries().await;

// entries are sorted from highest to lowest resolution
for icon in entries {
  println("{:?}", icon)
}
```

## Running locally

Install [cargo make](https://github.com/sagiegurari/cargo-make) and then:

```bash
cargo make run facebook/react
```
