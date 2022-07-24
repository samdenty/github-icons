use bytes::Bytes;
use data_url::DataUrl;
#[cfg(feature = "image")]
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use site_icons::{IconInfo, IconKind};
use std::{
  error::Error,
  fmt::{self, Display},
  str::FromStr,
};
use url::Url;

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum RepoIconKind {
  UserAvatar,
  ReadmeImage,
  Site(IconKind),
}

impl Display for RepoIconKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      RepoIconKind::ReadmeImage => write!(f, "readme_image"),
      RepoIconKind::UserAvatar => write!(f, "user_avatar"),
      RepoIconKind::Site(kind) => write!(f, "{}", kind),
    }
  }
}

impl FromStr for RepoIconKind {
  type Err = String;

  fn from_str(kind: &str) -> Result<Self, Self::Err> {
    match kind {
      "readme_image" => Ok(RepoIconKind::ReadmeImage),
      "user_avatar" => Ok(RepoIconKind::UserAvatar),
      kind => Ok(RepoIconKind::Site(IconKind::from_str(kind)?)),
    }
  }
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, PartialEq, Eq)]
pub struct RepoIcon {
  pub url: Url,
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
  pub fn new(url: Url, kind: RepoIconKind, info: IconInfo) -> Self {
    Self {
      url,
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

    let res = match self.url.domain().unwrap() {
      // fetch the response, with authorization headers included
      "raw.githubusercontent.com" => gh_get!("{}", self.url).send().await?,
      _ => reqwest::get(self.url.clone()).await?,
    };

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
