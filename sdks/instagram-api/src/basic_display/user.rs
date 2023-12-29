use apisdk::{send, ApiResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::AsRefStr;

use crate::InstagramBasicDisplayApi;

impl InstagramBasicDisplayApi {
    pub async fn get_me(&self) -> ApiResult<UserProfile> {
        let req = self.get("/me").await?;
        send!(req).await
    }

    pub async fn get_me_with(&self, fields: &[UserField]) -> ApiResult<Value> {
        let mut req = self.get("/me").await?;
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

    pub async fn get_me_media(&self) -> ApiResult<UserMedia> {
        let req = self.get("/me/media").await?;
        send!(req).await
    }
}

#[derive(Debug, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "snake_case")]
pub enum UserField {
    #[strum(serialize = "id")]
    Id,
    #[strum(serialize = "username")]
    Username,
    #[strum(serialize = "account_type")]
    AccountType,
    #[strum(serialize = "media_count")]
    MediaCount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    id: String,
    username: String,
    #[serde(default)]
    account_type: Option<AccountType>,
    #[serde(default)]
    media_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    Business,
    MediaCreator,
    Personal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMedia {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMediaList {
    data: Vec<UserMedia>,
    paging: Paging,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Paging {
    cursors: Cursors,
    next: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cursors {
    after: String,
    before: String,
}

#[cfg(test)]
mod tests {
    use crate::basic_display::tests::create_api;

    #[tokio::test]
    async fn test_get_me() {
        let api = create_api();
        let result = api.get_me().await;
        println!("result = {:?}", result);
    }
}
