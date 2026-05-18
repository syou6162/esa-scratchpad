#[cfg(test)]
#[path = "client_tests.rs"]
mod client_tests;

use crate::error::ApiError;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.esa.io/v1/teams";
const RATE_LIMIT_WARNING_THRESHOLD: u64 = 20;

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    #[allow(dead_code)]
    pub total_count: u32,
    pub posts: Vec<Post>,
}

#[derive(Debug, Deserialize)]
pub struct Post {
    pub number: u64,
    #[allow(dead_code)]
    pub name: String,
    pub body_md: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub url: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreatePostRequest {
    post: CreatePostBody,
}

#[derive(Debug, Serialize)]
struct CreatePostBody {
    name: String,
    category: String,
    body_md: String,
    wip: bool,
    tags: Vec<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct UpdatePostBodyRequest {
    post: UpdatePostBodyBody,
}

#[derive(Debug, Serialize)]
struct UpdatePostBodyBody {
    body_md: String,
    tags: Vec<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct UpdatePostNameRequest {
    post: UpdatePostNameBody,
}

#[derive(Debug, Serialize)]
struct UpdatePostNameBody {
    name: String,
    message: String,
    wip: bool,
}

pub struct EsaClient {
    team_name: String,
    access_token: String,
    client: reqwest::blocking::Client,
}

impl EsaClient {
    pub fn new(team_name: String, access_token: String) -> Self {
        Self {
            team_name,
            access_token,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn check_rate_limit(response: &reqwest::blocking::Response) {
        if let Some(remaining) = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
        {
            if remaining < RATE_LIMIT_WARNING_THRESHOLD {
                eprintln!("Warning: esa.io API rate limit remaining: {}", remaining);
            }
        }
    }

    fn handle_response<T: serde::de::DeserializeOwned>(
        response: reqwest::blocking::Response,
    ) -> Result<T, ApiError> {
        Self::check_rate_limit(&response);

        let status = response.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ApiError::RateLimitExceeded);
        }
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(ApiError::ApiResponse {
                status: status.as_u16(),
                body,
            });
        }

        Ok(response.json::<T>()?)
    }

    pub fn search_by_category(&self, category: &str) -> Result<Option<Post>, ApiError> {
        let url = format!("{}/{}/posts", BASE_URL, self.team_name);
        let query = format!("category:{}", category);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .query(&[("q", &query), ("per_page", &"1".to_string())])
            .send()?;

        let search: SearchResponse = Self::handle_response(response)?;
        Ok(search.posts.into_iter().next())
    }

    pub fn create_post(
        &self,
        name: &str,
        category: &str,
        body_md: &str,
        wip: bool,
        tags: &[String],
        message: &str,
    ) -> Result<Post, ApiError> {
        let url = format!("{}/{}/posts", BASE_URL, self.team_name);

        let request_body = CreatePostRequest {
            post: CreatePostBody {
                name: name.to_string(),
                category: category.to_string(),
                body_md: body_md.to_string(),
                wip,
                tags: tags.to_vec(),
                message: message.to_string(),
            },
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&request_body)
            .send()?;

        Self::handle_response(response)
    }

    pub fn update_post_body(
        &self,
        post_number: u64,
        body_md: &str,
        tags: &[String],
        message: &str,
    ) -> Result<Post, ApiError> {
        let url = format!("{}/{}/posts/{}", BASE_URL, self.team_name, post_number);

        let request_body = UpdatePostBodyRequest {
            post: UpdatePostBodyBody {
                body_md: body_md.to_string(),
                tags: tags.to_vec(),
                message: message.to_string(),
            },
        };

        let response = self
            .client
            .patch(&url)
            .bearer_auth(&self.access_token)
            .json(&request_body)
            .send()?;

        Self::handle_response(response)
    }

    pub fn update_post_name(
        &self,
        post_number: u64,
        name: &str,
        message: &str,
    ) -> Result<Post, ApiError> {
        let url = format!("{}/{}/posts/{}", BASE_URL, self.team_name, post_number);

        let request_body = UpdatePostNameRequest {
            post: UpdatePostNameBody {
                name: name.to_string(),
                message: message.to_string(),
                wip: false,
            },
        };

        let response = self
            .client
            .patch(&url)
            .bearer_auth(&self.access_token)
            .json(&request_body)
            .send()?;

        Self::handle_response(response)
    }
}
