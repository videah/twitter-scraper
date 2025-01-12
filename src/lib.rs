mod constants;
mod headers;
mod models;
mod parser;
mod request_builder;
use constants::TWITTER_BASE_URL;
use headers::get_headers;
pub use models::{Tweet, User};
use parser::{get_next_cursor, get_tweets, get_users};
use request_builder::RequestConfig;
use reqwest::{Client, Response};
use serde_json::Value;

#[derive(Debug)]
pub struct TwitterResults {
    pub cursor: Option<String>,
    pub guest_token: String,
    pub tweets: Option<Vec<Tweet>>,
    pub users: Option<Vec<User>>,
}

#[tokio::main]
pub async fn run(
    query: String,
    auth_token_option: Option<&'static str>,
    guest_token_option: Option<&'static str>,
    cursor: Option<String>,
    ignore_cursor: bool,
) -> Result<TwitterResults, Box<dyn std::error::Error>> {
    run_async(query, auth_token_option, guest_token_option, cursor, ignore_cursor).await
}

pub async fn run_async(
    query: String,
    auth_token_option: Option<&'static str>,
    guest_token_option: Option<&'static str>,
    cursor: Option<String>,
    ignore_cursor: bool,
) -> Result<TwitterResults, Box<dyn std::error::Error>> {
    let headers_tuples: [(&'static str, &'static str); 2] =
        get_headers(auth_token_option, guest_token_option).await?;
    let request_config: RequestConfig =
        request_builder::build_request_config(&headers_tuples, query, cursor.clone());
    let client: Client = Client::new();
    let response: Response = client
        .get(TWITTER_BASE_URL)
        .query(&request_config.path_query)
        .headers(request_config.headers)
        .send()
        .await?
        .error_for_status()?;
    let body_data: Value = response.json::<Value>().await?;
    let next_cursor: Option<String> = match ignore_cursor {
        true => None,
        false => Some(get_next_cursor(&body_data, cursor)?)
    };
    let tweets: Vec<Tweet> = get_tweets(&body_data);
    let users: Vec<User> = get_users(&body_data);
    let guest_token: String = headers_tuples[1].1.to_string();
    let twitter_results = TwitterResults {
        cursor: next_cursor,
        guest_token,
        tweets: Some(tweets),
        users: Some(users),
    };
    Ok(twitter_results)
}