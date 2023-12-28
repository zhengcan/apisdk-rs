use std::collections::HashMap;

use apisdk::{send_form, ApiResult};
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use crate::InstagramBasicDisplayApi;

impl InstagramBasicDisplayApi {
    pub async fn build_authorize_url(
        &self,
        redirect_uri: impl AsRef<str>,
        scope: impl IntoIterator<Item = Scope>,
        state: Option<impl AsRef<str>>,
    ) -> String {
        let mut url = self.build_url("/oauth/authorize").await.unwrap();
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("client_id", &self.secret.app_id);
            query_pairs.append_pair("redirect_uri", redirect_uri.as_ref());
            query_pairs.append_pair(
                "scope",
                scope
                    .into_iter()
                    .map(|s| s.as_ref().to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
            query_pairs.append_pair("response_type", "code");
            if let Some(state) = state {
                query_pairs.append_pair("state", state.as_ref());
            }
        }
        url.to_string()
    }

    pub async fn get_access_token(
        &self,
        code: impl AsRef<str>,
        redirect_uri: impl AsRef<str>,
    ) -> ApiResult<ShortLiveUserdAccessToken> {
        let req = self.post("/oauth/access_token").await?;
        let form = HashMap::from([
            ("client_id", self.secret.app_id.as_ref()),
            ("client_secret", self.secret.app_secret.as_ref()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_ref()),
            ("code", code.as_ref()),
        ]);
        send_form!(req, form).await
    }
}

#[derive(Debug, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    #[strum(serialize = "user_profile")]
    UserProfile,
    #[strum(serialize = "user_media")]
    UserMedia,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShortLiveUserdAccessToken {
    user_id: u64,
    access_token: String,
}

#[cfg(test)]
mod tests {
    use crate::{basic_display::tests::create_api, Scope};

    #[tokio::test]
    async fn test_build_authorize_url() {
        let api = create_api();
        let url = api
            .build_authorize_url(
                "http://site/redirect_uri",
                vec![Scope::UserProfile, Scope::UserMedia],
                None::<&str>,
            )
            .await;
        println!("url = {}", url);
    }

    #[tokio::test]
    async fn test_get_access_token() {
        let api = create_api();
        let result = api
            .get_access_token("code", "http://site/redirect_uri")
            .await;
        println!("result = {:?}", result);
    }
}
