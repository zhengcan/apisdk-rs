use apisdk::{send, send_json, ApiResult};
use serde_json::json;

use crate::{
    dto::{Comment, Post},
    TypicodeApi,
};

impl TypicodeApi {
    pub async fn get_post(&self, post_id: u64) -> ApiResult<Post> {
        let req = self.get(format!("/posts/{}", post_id)).await?;
        send!(req).await
    }

    pub async fn list_posts(&self) -> ApiResult<Vec<Post>> {
        let req = self.get("/posts").await?;
        send!(req).await
    }

    pub async fn filter_posts(
        &self,
        filter: (impl ToString, impl ToString),
    ) -> ApiResult<Vec<Post>> {
        let req = self.get("/posts").await?;
        let req = req.query(&[(filter.0.to_string(), filter.1.to_string())]);
        send!(req).await
    }

    pub async fn create_post(
        &self,
        title: impl ToString,
        body: impl ToString,
        user_id: u64,
    ) -> ApiResult<Post> {
        let req = self.post("/posts").await?;
        let body = json!({
            "title": title.to_string(),
            "body": body.to_string(),
            "userId": user_id,
        });
        send_json!(req, body).await
    }

    pub async fn update_post(
        &self,
        post_id: u64,
        title: impl ToString,
        body: impl ToString,
        user_id: u64,
    ) -> ApiResult<Post> {
        let req = self.put(format!("/posts/{}", post_id)).await?;
        let body = json!({
            "postId": post_id,
            "title": title.to_string(),
            "body": body.to_string(),
            "userId": user_id,
        });
        send_json!(req, body).await
    }

    pub async fn patch_post(&self, post_id: u64, title: impl ToString) -> ApiResult<Post> {
        let req = self.patch(format!("/posts/{}", post_id)).await?;
        let body = json!({
            "title": title.to_string(),
        });
        send_json!(req, body).await
    }

    pub async fn list_post_comments(&self, post_id: u64) -> ApiResult<Vec<Comment>> {
        let req = self.get(format!("/posts/{}/comments", post_id)).await?;
        send!(req).await
    }
}

#[cfg(test)]
mod tests {
    use apisdk::ApiResult;

    use crate::TypicodeApi;

    #[tokio::test]
    async fn test_get_post() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let post = api.get_post(1).await?;
        println!("{:?}", post);
        assert_eq!(1, post.id);
        Ok(())
    }

    #[tokio::test]
    async fn test_list_post() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let posts = api.list_posts().await?;
        println!("{:?}", posts);
        assert!(!posts.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_post() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let posts = api.filter_posts(("userId", 1)).await?;
        println!("{:?}", posts);
        assert!(!posts.is_empty());
        for post in posts {
            assert_eq!(1, post.user_id);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_create_post() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let post = api.create_post("title", "body", 1).await?;
        println!("{:?}", post);
        assert_eq!("title", post.title);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_post() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let post = api.update_post(1, "title", "body", 1).await?;
        println!("{:?}", post);
        assert_eq!("title", post.title);
        Ok(())
    }

    #[tokio::test]
    async fn test_patch_post() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let post = api.patch_post(1, "title").await?;
        println!("{:?}", post);
        assert_eq!("title", post.title);
        Ok(())
    }

    #[tokio::test]
    async fn test_list_post_comments() -> ApiResult<()> {
        let api = TypicodeApi::default();
        let comments = api.list_post_comments(1).await?;
        println!("{:?}", comments);
        for comment in comments {
            assert_eq!(1, comment.post_id);
        }
        Ok(())
    }
}
