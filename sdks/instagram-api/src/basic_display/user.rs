use apisdk::{send, ApiResult};
use serde::{Deserialize, Serialize};

use crate::InstagramBasicDisplayApi;

impl InstagramBasicDisplayApi {
    pub async fn get_me(&self, fields: Option<Vec<&'static str>>) -> ApiResult<()> {
        let mut req = self.get("/me").await?;
        if let Some(fields) = fields {
            req = req.query(&[(
                "fields",
                fields
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            )]);
        }
        send!(req).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    id: String,
    username: String,
    account_type: Option<AccountType>,
    media_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AccountType {}
