use crate::github_api::{get_redirected_user, stripped_owner_lowercase};
use data_url::DataUrl;
use futures::{
  join,
  stream::{self, LocalBoxStream},
  StreamExt,
};
use gh_api::get_token;
#[cfg(feature = "image")]
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
#[cfg(target_arch = "wasm32")]
use js_sys::Uint8Array;
use maplit::hashmap;
use reqwest::{IntoUrl, Response};
use serde::{de, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use site_icons::{IconInfo, IconKind};
use std::{
  cmp::Ordering,
  collections::HashMap,
  convert::TryInto,
  error::Error,
  fmt::{self, Display},
  iter,
};
use url::Url;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::ReadableStream;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Framework {
  Vue,
  CreateReactApp,
  Next,
  MdBook,
}

impl From<&RepoFile> for Option<Framework> {
  fn from(file: &RepoFile) -> Self {
    Some(match &file.sha[..] {
      "df36fcfb72584e00488330b560ebcf34a41c64c2" | "c7b9a43c8cd16d0b434adaf513fcacb340809a11" => {
        Framework::Vue
      }
      "718d6fea4835ec2d246af9800eddb7ffb276240c" => Framework::Next,
      "bcd5dfd67cd0361b78123e95c2dd96031f27f743" | "a11777cc471a4344702741ab1c8a588998b1311a" => {
        Framework::CreateReactApp
      }
      "a5b1aa16c4dcb6c872cb5af799bfc9b5552c7b9e" => Framework::MdBook,
      _ => return None,
    })
  }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct RepoFile {
  pub github: String,
  pub commit_sha: String,

  pub sha: String,
  pub path: String,
}

impl PartialOrd for RepoFile {
  fn partial_cmp(&self, _other: &RepoFile) -> Option<Ordering> {
    None
  }
}

impl Ord for RepoFile {
  fn cmp(&self, _other: &RepoFile) -> Ordering {
    Ordering::Equal
  }
}

impl PartialEq for RepoFile {
  fn eq(&self, other: &Self) -> bool {
    self.github == other.github && self.sha == other.sha
  }
}

// NOTE: When you change the order of these, make sure
// to update the order of the hardcoded if statements
// for the best_matches option, otherwise you'll get
// inconsistent results when that option is set to
// true or false
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum RepoIconKind {
  IconField {
    file: RepoFile,
  },
  Avatar,
  AppIcon {
    homepage: Url,
  },
  SiteFavicon {
    homepage: Url,
  },
  RepoFile {
    file: RepoFile,
  },
  ReadmeImage,
  OrgAvatar,
  SiteLogo {
    homepage: Url,
  },
  Framework {
    file: RepoFile,
    framework: Framework,
  },
  UserAvatarFallback,
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
      RepoIconKind::IconField { .. } => write!(f, "icon_field"),
      RepoIconKind::Avatar { .. } => write!(f, "avatar"),
      RepoIconKind::UserAvatarFallback => write!(f, "user_avatar_fallback"),
      RepoIconKind::OrgAvatar => write!(f, "org_avatar"),
      RepoIconKind::AppIcon { .. } => write!(f, "app_icon"),
      RepoIconKind::Framework { .. } => write!(f, "framework_icon"),
      RepoIconKind::RepoFile { .. } => write!(f, "repo_file"),
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
      RepoIconKind::Framework { framework, file } => {
        state.serialize_entry("framework", framework)?;
        state.serialize_entry("file", file)?;
      }
      RepoIconKind::RepoFile { file } | RepoIconKind::IconField { file } => {
        state.serialize_entry("file", file)?;
      }
      RepoIconKind::ReadmeImage
      | RepoIconKind::Avatar
      | RepoIconKind::OrgAvatar
      | RepoIconKind::UserAvatarFallback => {}
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
      file: Option<RepoFile>,
      framework: Option<Framework>,
    }

    let fields = RepoIconFields::deserialize(deserializer)?;

    Ok(match fields.kind.as_ref() {
      "icon_field" => RepoIconKind::IconField {
        file: fields.file.unwrap(),
      },
      "user_avatar_fallback" => RepoIconKind::UserAvatarFallback,
      "org_avatar" => RepoIconKind::OrgAvatar,
      "avatar" => RepoIconKind::Avatar,
      "app_icon" => RepoIconKind::AppIcon {
        homepage: fields.homepage.unwrap(),
      },
      "repo_file" => RepoIconKind::RepoFile {
        file: fields.file.unwrap(),
      },
      "framework_icon" => RepoIconKind::Framework {
        file: fields.file.unwrap(),
        framework: fields.framework.unwrap(),
      },
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

  pub async fn load<U: IntoUrl>(url: U, kind: RepoIconKind) -> Result<Self, Box<dyn Error>> {
    Self::load_with_headers(url.into_url()?, HashMap::new(), kind).await
  }

  pub async fn load_with_headers<U: IntoUrl>(
    url: U,
    headers: HashMap<String, String>,
    kind: RepoIconKind,
  ) -> Result<Self, Box<dyn Error>> {
    let url = url.into_url()?;
    let info = IconInfo::load(url.clone(), (&headers).try_into()?, None).await?;
    Ok(Self::new_with_headers(url, headers, kind, info))
  }

  pub async fn load_user_avatar(owner: &str, repo: &str) -> Result<Self, Box<dyn Error>> {
    let owner = owner.to_lowercase();
    let repo = repo.to_lowercase();

    let docs = regex!("^(docs|documentation)$");
    let fallback =
      !repo.contains(&stripped_owner_lowercase(&owner)) && !docs.is_match(&repo).unwrap();

    let (redirected_user, owner_avatar) = join!(
      get_redirected_user(&owner, &repo),
      RepoIcon::load(
        format!("https://github.com/{}.png", owner),
        RepoIconKind::Avatar,
      )
    );

    let (user, is_org) = redirected_user?;

    let kind = if fallback {
      if is_org {
        RepoIconKind::OrgAvatar
      } else {
        RepoIconKind::UserAvatarFallback
      }
    } else {
      RepoIconKind::Avatar
    };

    if user == owner {
      let mut avatar = owner_avatar?;
      avatar.kind = kind;
      return Ok(avatar);
    }

    RepoIcon::load(format!("https://github.com/{}.png", user), kind).await
  }

  pub async fn load_repo_file(file: RepoFile, is_icon_field: bool) -> Result<Self, Box<dyn Error>> {
    let url = Url::parse(&format!(
      "https://api.github.com/repos/{}/git/blobs/{}",
      file.github, file.sha
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
        RepoIconKind::IconField { file }
      } else {
        if let Some(framework) = (&file).into() {
          RepoIconKind::Framework { framework, file }
        } else {
          RepoIconKind::RepoFile { file }
        }
      },
    )
    .await
  }

  pub fn set_repo_private(&mut self, is_private: bool) {
    use RepoIconKind::*;

    if let Framework { file, .. } | RepoFile { file } | IconField { file } = &mut self.kind {
      if !is_private {
        self.headers.clear();
        self.url = Url::parse(&format!(
          "https://raw.githubusercontent.com/{}/{}/{}",
          file.github, file.commit_sha, file.path
        ))
        .unwrap();
      }
    }
  }

  async fn response(&self) -> Result<IconResponse, Box<dyn Error>> {
    if self.url.scheme() == "data" {
      let url = self.url.to_string();
      let data = DataUrl::process(&url).map_err(|_| "failed to parse data uri")?;
      let (body, _fragment) = data
        .decode_to_vec()
        .map_err(|_| "invalid base64 in data uri")?;

      return Ok(IconResponse::DataURI(body));
    }

    let res = reqwest::Client::new()
      .get(self.url.clone())
      .headers((&self.headers).try_into()?)
      .send()
      .await
      .map_err(|e| format!("{}: {:?}", self.url, e))?
      .error_for_status()
      .map_err(|e| format!("{}: {:?}", self.url, e))?;

    Ok(IconResponse::Network(res))
  }

  #[cfg(target_arch = "wasm32")]
  pub async fn js_stream(&self) -> Result<ReadableStream, Box<dyn Error>> {
    Ok(match self.response().await? {
      IconResponse::DataURI(body) => {
        let body = Uint8Array::from(&body[..]);

        wasm_streams::ReadableStream::from_stream(stream::iter(iter::once(Ok(body.into()))))
          .into_raw()
          .dyn_into()
          .unwrap()
      }
      IconResponse::Network(res) => res.js_stream(),
    })
  }

  pub async fn stream<'a>(
    &self,
  ) -> Result<LocalBoxStream<'a, Result<Vec<u8>, reqwest::Error>>, Box<dyn Error>> {
    Ok(match self.response().await? {
      IconResponse::DataURI(body) => stream::iter(iter::once(Ok(body))).boxed_local(),
      IconResponse::Network(res) => {
        let stream = res.bytes_stream();

        stream
          .map(|buf| buf.map(|bytes| bytes.to_vec()))
          .boxed_local()
      }
    })
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

enum IconResponse {
  Network(Response),
  DataURI(Vec<u8>),
}
