<h1 align="center">
  <img src="./cli/logo.png" width="150">
</h1>

[![Open in Codeflow](https://developer.stackblitz.com/img/open_in_codeflow.svg)](https://pr.new/samdenty/github-icons)

[![Website demo](assets/api-demo.gif)](https://github-icons.com)

## API Usage

To use the API, generate a token by signing into the website first (click the search bar). A token is required we use the GitHub API, and that is rate limited.

Consider [sponsoring the project](https://github.com/sponsors/samdenty) as it costs to run the API.

```bash
# GitHub Repo icon API:
GET https://github-icons.com/[user]/[repo]?token=[token]
# NPM Package icon API:
GET https://github-icons.com/npm/[package]?token=[token]

# List all icons for a repo:
GET https://github-icons.com/[user]/[repo]/all?token=[token]
# List all icons for a package:
GET https://github-icons.com/npm/[package]/all?token=[token]
```

## Mac APP

### [Download the app](https://github.com/samdenty/github-icons/releases/latest)

Automatically adds project logos to your locally cloned GitHub repos. [Youtube Video](https://www.youtube.com/watch?v=jrO3qSEpAFU)

This repository contains the source code for the github-icons CLI. You can also [sponsor this project](https://github.com/sponsors/samdenty)

## Repo structure

| Folder                                  | Description                                          |
| --------------------------------------- | ---------------------------------------------------- |
| [`api`](/api)                           | Cloudflare worker for fetching repo icons            |
| [`cli`](/cli)                           | CLI for adding repo icons to .git folders            |
| [`chrome-extension`](/chrome-extension) | Chrome extension for adding repo icons to github.com |
| [`vscode-extension`](/vscode-extension) | VSCode extension that adds NPM / GitHub icons        |
| [`repo_icons`](/repo_icons)             | Rust crate for scraping repo icons                   |
| [`website`](/website)                   | The www.github-icons.com website                     |

## Running the CLI

You can run the CLI using the below commands.

<!-- brew install mysql-client
cargo install diesel_cli --no-default-features --features mysql -->

```bash
# To install rust
curl https://sh.rustup.rs -sSf | sh

# Clone the repository
git clone https://github.com/samdenty/github-icons
cd github-icons

cargo run -- sync
# or with github token (for private repos)
cargo run -- sync --token INSERT_TOKEN
```

[![Banner](./banner.gif)](https://samddenty.gumroad.com/l/git-icons)
