use apisdk::{send, ApiResult};

use crate::{dto::Photo, TypicodeApi};

impl TypicodeApi {
    pub async fn list_album_photos(&self, album_id: u64) -> ApiResult<Vec<Photo>> {
        let req = self.get(format!("/album/{}/photos", album_id)).await?;
        send!(req).await
    }
}

#[cfg(test)]
mod tests {
    use apisdk::ApiResult;

    use crate::TypicodeApi;

    #[tokio::test]
    async fn test_list_album_photos() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let photos = api.list_album_photos(1).await?;
        println!("{:?}", photos);
        for photo in photos {
            assert_eq!(1, photo.album_id);
        }
        Ok(())
    }
}
