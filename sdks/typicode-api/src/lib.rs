use apisdk::http_api;

mod albums;
mod dto;
mod posts;
mod users;

#[http_api("https://jsonplaceholder.typicode.com/")]
pub struct TypicodeApi;
