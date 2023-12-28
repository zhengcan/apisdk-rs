use apisdk::{send, ApiResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::AsRefStr;

use crate::InstagramBasicDisplayApi;

impl InstagramBasicDisplayApi {
    pub async fn get_media(&self, media_id: impl AsRef<str>) -> ApiResult<Media> {
        let req = self
            .get(format!("/{}/{}", self.api_version, media_id.as_ref()))
            .await?;
        send!(req).await
    }

    pub async fn get_media_with(
        &self,
        media_id: impl AsRef<str>,
        fields: &[MediaField],
    ) -> ApiResult<Value> {
        let mut req = self
            .get(format!("/{}/{}", self.api_version, media_id.as_ref()))
            .await?;
        req = req.query(&[(
            "fields",
            fields
                .iter()
                .map(|f| f.as_ref())
                .collect::<Vec<_>>()
                .join(","),
        )]);
        send!(req).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    id: String,
}

#[derive(Debug, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "snake_case")]
pub enum MediaField {
    #[strum(serialize = "id")]
    Id,
    #[strum(serialize = "media_type")]
    MediaType,
}
