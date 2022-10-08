use bytes::Bytes;
use data_url::DataUrl;
use gh_api::get_token;
#[cfg(feature = "image")]
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use maplit::hashmap;
use site_icons::{IconInfo, IconKind};
use std::{
  collections::HashMap,
  convert::TryInto,
  error::Error,
  fmt::{self, Display},
  str::FromStr,
};
use url::Url;

#[derive(Debug, PartialOrd, Ord, Eq)]
pub struct RepoBlob {
  pub owner: String,
  pub repo: String,
  pub commit_sha: String,

  pub sha: String,
  pub path: String,
}

impl PartialEq for RepoBlob {
  fn eq(&self, other: &Self) -> bool {
    self.owner == other.owner && self.repo == other.repo && self.sha == other.sha
  }
}

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum RepoIconKind {
  UserAvatar,
  ReadmeImage,
  Blob(Option<RepoBlob>),
  Site(IconKind),
}

impl Display for RepoIconKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      RepoIconKind::ReadmeImage => write!(f, "readme_image"),
      RepoIconKind::UserAvatar => write!(f, "user_avatar"),
      RepoIconKind::Blob(_) => write!(f, "blob"),
      RepoIconKind::Site(kind) => write!(f, "{}", kind),
    }
  }
}

impl FromStr for RepoIconKind {
  type Err = String;

  fn from_str(kind: &str) -> Result<Self, Self::Err> {
    Ok(match kind {
      "readme_image" => RepoIconKind::ReadmeImage,
      "user_avatar" => RepoIconKind::UserAvatar,
      "blob" => RepoIconKind::Blob(None),
      kind => RepoIconKind::Site(IconKind::from_str(kind)?),
    })
  }
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, PartialEq, Eq)]
pub struct RepoIcon {
  pub url: Url,
  pub headers: HashMap<String, String>,

  #[serde(with = "serde_with::rust::display_fromstr")]
  pub kind: RepoIconKind,
  #[serde(flatten)]
  pub info: IconInfo,

  #[cfg(feature = "image")]
  #[serde(skip)]
  #[derivative(PartialEq = "ignore")]
  #[derivative(Debug = "ignore")]
  image: RefCell<Option<Rc<DynamicImage>>>,
}

impl RepoIcon {
  pub fn blob_set_private(&mut self, is_private: bool) {
    if let RepoIconKind::Blob(Some(blob)) = &mut self.kind {
      if !is_private {
        self.headers.clear();
        self.url = Url::parse(&format!(
          "https://raw.githubusercontent.com/{}/{}/{}/{}",
          blob.owner, blob.repo, blob.commit_sha, blob.path
        ))
        .unwrap();
      }
    }
  }

  pub async fn load_blob(blob: RepoBlob) -> Result<Self, Box<dyn Error>> {
    let url = Url::parse(&format!(
      "https://api.github.com/repos/{}/{}/git/blobs/{}",
      blob.owner, blob.repo, blob.sha
    ))
    .unwrap();

    let headers = hashmap! {
      "Authorization".to_string() => format!("Bearer {}", get_token().unwrap()),
      "Accept".to_string() => "application/vnd.github.raw".to_string(),
    };

    let info = IconInfo::load(url.clone(), (&headers).try_into()?, None).await?;

    Ok(Self::new_with_headers(
      url,
      headers,
      RepoIconKind::Blob(Some(blob)),
      info,
    ))
  }

  pub fn new(url: Url, kind: RepoIconKind, info: IconInfo) -> Self {
    Self::new_with_headers(url, HashMap::new(), kind, info)
  }

  pub fn new_with_headers(
    url: Url,
    headers: HashMap<String, String>,
    kind: RepoIconKind,
    info: IconInfo,
  ) -> Self {
    Self {
      url,
      headers,
      kind,
      info,
      #[cfg(feature = "image")]
      image: RefCell::new(None),
    }
  }

  pub async fn data(&self) -> Result<Bytes, Box<dyn Error>> {
    if self.url.scheme() == "data" {
      let url = self.url.to_string();
      let data = DataUrl::process(&url).map_err(|_| "failed to parse data uri")?;
      let (body, _fragment) = data
        .decode_to_vec()
        .map_err(|_| "invalid base64 in data uri")?;

      return Ok(body.into());
    }

    let res = reqwest::Client::new()
      .get(self.url.clone())
      .headers((&self.headers).try_into()?)
      .send()
      .await?;

    Ok(res.bytes().await?)
  }

  #[cfg(feature = "image")]
  pub async fn image(&self) -> Result<Rc<DynamicImage>, Box<dyn Error>> {
    if let Some(image) = self.image.borrow().clone() {
      return Ok(image);
    }

    let data = self.data().await?;
    let mut reader = ImageReader::new(Cursor::new(data));

    reader.set_format(match self.info {
      IconInfo::PNG { .. } => ImageFormat::Png,
      IconInfo::JPEG { .. } => ImageFormat::Jpeg,
      IconInfo::ICO { .. } => ImageFormat::Ico,
      IconInfo::SVG { .. } => return Err("not supported!".into()),
    });

    let image = Rc::new(reader.decode()?);
    *self.image.borrow_mut() = Some(image.clone());
    Ok(image)
  }
}
