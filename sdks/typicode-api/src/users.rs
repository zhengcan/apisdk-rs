use apisdk::{send, ApiResult};

use crate::{
    dto::{Album, Photo, Todo},
    TypicodeApi,
};

impl TypicodeApi {
    pub async fn list_user_albums(&self, user_id: u64) -> ApiResult<Vec<Album>> {
        let req = self.get(format!("/user/{}/albums", user_id)).await?;
        send!(req).await
    }

    pub async fn list_user_todos(&self, user_id: u64) -> ApiResult<Vec<Todo>> {
        let req = self.get(format!("/user/{}/todos", user_id)).await?;
        send!(req).await
    }

    pub async fn list_user_photos(&self, user_id: u64) -> ApiResult<Vec<Photo>> {
        let req = self.get(format!("/user/{}/photos", user_id)).await?;
        send!(req).await
    }
}

#[cfg(test)]
mod tests {
    use apisdk::ApiResult;

    use crate::TypicodeApi;

    #[tokio::test]
    async fn test_list_user_albums() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let albums = api.list_user_albums(1).await?;
        println!("{:?}", albums);
        for album in albums {
            assert_eq!(1, album.user_id);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_list_user_todos() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let todos = api.list_user_todos(1).await?;
        println!("{:?}", todos);
        for todo in todos {
            assert_eq!(1, todo.user_id);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_list_user_photos() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let photos = api.list_user_photos(1).await?;
        println!("{:?}", photos);
        Ok(())
    }
}
