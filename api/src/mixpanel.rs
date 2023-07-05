use serde_json::json;
use worker::*;

pub async fn track(event: &str, id: &str, mut properties: serde_json::Value) -> () {
  properties["distinct_id"] = json!(id);
  properties["token"] = json!("072daf79f7202d35687b51b431ef2816");

  let data = json!({
      "event": event,
      "properties": properties,
  })
  .to_string();

  let data_base64 = base64::encode(data.as_bytes());

  let _ = Fetch::Url(
    format!("https://api.mixpanel.com/track/?data={}", data_base64)
      .parse()
      .unwrap(),
  )
  .send()
  .await;
}
