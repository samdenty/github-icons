use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryInto};
use worker::*;

#[derive(Serialize, Deserialize)]
pub struct SerializedResponse {
  status_code: u16,
  headers: HashMap<String, String>,
  body: Vec<u8>,
}

impl TryInto<Response> for SerializedResponse {
  type Error = worker::Error;

  fn try_into(self) -> Result<Response> {
    let mut headers = Headers::new();

    for (key, value) in &self.headers {
      headers.set(key, value)?;
    }

    Ok(
      Response::from_bytes(self.body)?
        .with_status(self.status_code)
        .with_headers(headers),
    )
  }
}

impl SerializedResponse {
  pub fn deserialize(response: String) -> Result<Response> {
    let serialized_response: SerializedResponse = serde_json::from_str(&response)?;

    serialized_response.try_into()
  }

  pub async fn from(mut response: Response) -> Result<Self> {
    let status_code = response.status_code();

    let mut headers = HashMap::new();
    for (key, value) in response.headers() {
      headers.insert(key.to_string(), value.to_string());
    }

    let body = response.bytes().await?;

    Ok(Self {
      status_code,
      headers,
      body,
    })
  }
}
