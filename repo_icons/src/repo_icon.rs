use data_url::DataUrl;
use gh_api::get_token;
#[cfg(feature = "image")]
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use maplit::hashmap;
use serde::{de, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use site_icons::{IconInfo, IconKind};
use std::{
  cmp::Ordering,
  collections::HashMap,
  convert::TryInto,
  error::Error,
  fmt::{self, Display},
};
use url::Url;

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct RepoBlob {
  pub slug: String,
  pub commit_sha: String,

  pub sha: String,
  pub path: String,
}

impl PartialOrd for RepoBlob {
  fn partial_cmp(&self, _other: &RepoBlob) -> Option<Ordering> {
    None
  }
}

impl Ord for RepoBlob {
  fn cmp(&self, _other: &RepoBlob) -> Ordering {
    Ordering::Equal
  }
}

impl PartialEq for RepoBlob {
  fn eq(&self, other: &Self) -> bool {
    self.slug == other.slug && self.sha == other.sha
  }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]

pub enum RepoIconKind {
  IconField(RepoBlob),
  UserAvatar,
  AppIcon { homepage: Url },
  SiteFavicon { homepage: Url },
  RepoFile(RepoBlob),
  ReadmeImage,
  SiteLogo { homepage: Url },
}

impl From<(Url, IconKind)> for RepoIconKind {
  fn from((homepage, kind): (Url, IconKind)) -> Self {
    match kind {
      IconKind::AppIcon => RepoIconKind::AppIcon { homepage },
      IconKind::SiteLogo => RepoIconKind::SiteLogo { homepage },
      IconKind::SiteFavicon => RepoIconKind::SiteFavicon { homepage },
    }
  }
}

impl Display for RepoIconKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      RepoIconKind::IconField(_) => write!(f, "icon_field"),
      RepoIconKind::UserAvatar => write!(f, "user_avatar"),
      RepoIconKind::AppIcon { .. } => write!(f, "app_icon"),
      RepoIconKind::RepoFile(_) => write!(f, "repo_file"),
      RepoIconKind::SiteFavicon { .. } => write!(f, "site_favicon"),
      RepoIconKind::ReadmeImage => write!(f, "readme_image"),
      RepoIconKind::SiteLogo { .. } => write!(f, "site_logo"),
    }
  }
}

impl Serialize for RepoIconKind {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_map(None)?;

    state.serialize_entry("kind", &self.to_string())?;

    match self {
      RepoIconKind::AppIcon { homepage }
      | RepoIconKind::SiteFavicon { homepage }
      | RepoIconKind::SiteLogo { homepage } => {
        state.serialize_entry("homepage", homepage)?;
      }
      RepoIconKind::RepoFile(blob) | RepoIconKind::IconField(blob) => {
        state.serialize_entry("slug", &blob.slug)?;
        state.serialize_entry("commit_sha", &blob.commit_sha)?;
        state.serialize_entry("sha", &blob.sha)?;
        state.serialize_entry("path", &blob.path)?;
      }
      _ => {}
    }

    state.end()
  }
}

impl<'de> Deserialize<'de> for RepoIconKind {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct RepoIconFields {
      kind: String,
      homepage: Option<Url>,
      slug: Option<String>,
      commit_sha: Option<String>,
      sha: Option<String>,
      path: Option<String>,
    }

    let fields = RepoIconFields::deserialize(deserializer)?;

    Ok(match fields.kind.as_ref() {
      "icon_field" => RepoIconKind::IconField(RepoBlob {
        slug: fields.slug.unwrap(),
        commit_sha: fields.commit_sha.unwrap(),
        sha: fields.sha.unwrap(),
        path: fields.path.unwrap(),
      }),
      "user_avatar" => RepoIconKind::UserAvatar,
      "app_icon" => RepoIconKind::AppIcon {
        homepage: fields.homepage.unwrap(),
      },
      "repo_file" => RepoIconKind::RepoFile(RepoBlob {
        slug: fields.slug.unwrap(),
        commit_sha: fields.commit_sha.unwrap(),
        sha: fields.sha.unwrap(),
        path: fields.path.unwrap(),
      }),
      "site_favicon" => RepoIconKind::SiteFavicon {
        homepage: fields.homepage.unwrap(),
      },
      "readme_image" => RepoIconKind::ReadmeImage,
      "site_logo" => RepoIconKind::SiteLogo {
        homepage: fields.homepage.unwrap(),
      },

      _ => return Err(de::Error::custom("unknown icon kind".to_string())),
    })
  }
}

#[derive(Derivative, Clone, Serialize, Deserialize)]
#[derivative(Debug, PartialEq, Eq)]
pub struct RepoIcon {
  pub url: Url,
  pub headers: HashMap<String, String>,

  #[serde(flatten)]
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

  pub async fn load(url: Url, kind: RepoIconKind) -> Result<Self, Box<dyn Error>> {
    Self::load_with_headers(url, HashMap::new(), kind).await
  }

  pub async fn load_with_headers(
    url: Url,
    headers: HashMap<String, String>,
    kind: RepoIconKind,
  ) -> Result<Self, Box<dyn Error>> {
    let info = IconInfo::load(url.clone(), (&headers).try_into()?, None).await?;
    Ok(Self::new_with_headers(url, headers, kind, info))
  }

  pub async fn load_user_avatar(owner: &str) -> Option<Self> {
    let user_avatar_url: Url = format!("https://github.com/{}.png", owner).parse().unwrap();

    RepoIcon::load(user_avatar_url.clone(), RepoIconKind::UserAvatar)
      .await
      .ok()
  }

  pub async fn load_blob(blob: RepoBlob, is_icon_field: bool) -> Result<Self, Box<dyn Error>> {
    let url = Url::parse(&format!(
      "https://api.github.com/repos/{}/git/blobs/{}",
      blob.slug, blob.sha
    ))
    .unwrap();

    let mut headers = hashmap! {
      "Accept".to_string() => "application/vnd.github.raw".to_string(),
    };

    if let Some(token) = get_token() {
      headers.insert("Authorization".to_string(), format!("Bearer {}", token));
    }

    RepoIcon::load_with_headers(
      url,
      headers,
      if is_icon_field {
        RepoIconKind::IconField(blob)
      } else {
        RepoIconKind::RepoFile(blob)
      },
    )
    .await
  }

  pub fn set_repo_private(&mut self, is_private: bool) {
    use RepoIconKind::*;

    if let RepoFile(blob) | IconField(blob) = &mut self.kind {
      if !is_private {
        self.headers.clear();
        self.url = Url::parse(&format!(
          "https://raw.githubusercontent.com/{}/{}/{}",
          blob.slug, blob.commit_sha, blob.path
        ))
        .unwrap();
      }
    }
  }

  pub async fn data(&self) -> Result<Vec<u8>, Box<dyn Error>> {
    if self.url.scheme() == "data" {
      let url = self.url.to_string();
      let data = DataUrl::process(&url).map_err(|_| "failed to parse data uri")?;
      let (body, _fragment) = data
        .decode_to_vec()
        .map_err(|_| "invalid base64 in data uri")?;

      return Ok(body);
    }

    let res = reqwest::Client::new()
      .get(self.url.clone())
      .headers((&self.headers).try_into()?)
      .send()
      .await?;

    Ok(res.bytes().await?.to_vec())
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
