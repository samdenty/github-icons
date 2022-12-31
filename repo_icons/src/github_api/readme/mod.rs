mod primary_heading;
pub mod readme_image;

pub use readme_image::*;

use instant::Instant;
use primary_heading::PrimaryHeading;
use scraper::Html;
use std::{
  convert::TryInto,
  ops::{Deref, DerefMut},
};
use vec1::Vec1;

#[derive(Serialize, Deserialize, Debug)]
pub struct Readme(Vec1<ReadmeImage>);

impl Readme {
  pub async fn load(owner: &str, repo: &str) -> Option<Readme> {
    let url = format!("repos/{}/{}/readme", owner, repo);
    let start = Instant::now();
    let body = async {
      gh_api_get!("{}", url)
        .header("Accept", "application/vnd.github.html")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await
    }
    .await;

    info!("{}: {:?}", url, start.elapsed());

    let document = Html::parse_document(&body.ok()?);

    let primary_heading = &mut PrimaryHeading::new(&document);

    let mut images = Vec::new();
    for element_ref in document.select(selector!("img[src]")) {
      if let Some(image) = ReadmeImage::get(owner, repo, &element_ref, primary_heading).await {
        images.push(image);
      }
    }

    let mut iter = images.iter_mut().enumerate().peekable();
    while let Some((idx, image)) = iter.next() {
      if image.in_primary_heading
        && (idx == 0
          || iter
            .peek()
            .map(|(_, image)| !image.in_primary_heading)
            .unwrap_or(true))
      {
        image.edge_of_primary_heading = true;
      };
    }

    images.sort();

    images.try_into().ok().map(Readme)
  }
}

impl IntoIterator for Readme {
  type Item = ReadmeImage;
  type IntoIter = std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl Deref for Readme {
  type Target = Vec1<ReadmeImage>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Readme {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
