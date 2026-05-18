#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: status={status}, body={body}")]
    ApiResponse { status: u16, body: String },

    #[error("rate limit exceeded")]
    RateLimitExceeded,
}
