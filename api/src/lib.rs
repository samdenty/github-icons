use log::Level;
use repo_icons::Readme;
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() {
  console_log::init_with_level(Level::Info).unwrap();
  console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn set_token(token: String) {
  repo_icons::set_token(&token);
}

#[wasm_bindgen]
pub async fn get_repo_images(owner: String, repo: String) -> String {
  let readme = Readme::load(&owner, &repo).await.unwrap();
  let images = readme.images().await;

  serde_json::to_string_pretty(&images).unwrap()
}

#[wasm_bindgen]
pub async fn get_repo_icons(owner: String, repo: String) -> String {
  let images = repo_icons::get_repo_icons(&owner, &repo).await.unwrap();

  serde_json::to_string_pretty(&images).unwrap()
}

#[wasm_bindgen]
pub async fn get_repo_icon_url(owner: String, repo: String) -> Option<String> {
  let images = repo_icons::get_repo_icons(&owner, &repo).await.unwrap();

  images.first().map(|icon| icon.url.to_string())
}
