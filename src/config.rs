#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;

use std::env;
use std::fmt;

/// アプリケーション設定
///
/// 環境変数から構築される。CLI引数はサブコマンド側で定義する。
pub struct Config {
    pub team_name: String,
    pub access_token: String,
    pub category_prefix: String,
    pub post_name: String,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("team_name", &self.team_name)
            .field("access_token", &"[REDACTED]")
            .field("category_prefix", &self.category_prefix)
            .field("post_name", &self.post_name)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Required environment variable {name} is not set. {hint}")]
    MissingEnvVar { name: &'static str, hint: String },

    #[error("Configuration file error: {0}")]
    FileError(String),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

const DEFAULT_POST_NAME: &str = "ラクガキ帳";

impl Config {
    /// 環境変数から設定を構築
    ///
    /// 必須値が欠けている場合はエラーを返す。
    pub fn load() -> Result<Config, ConfigError> {
        let team_name = require_env("ESA_TEAM_NAME", "export ESA_TEAM_NAME=\"your-team-name\"")?;
        let access_token = require_env(
            "ESA_ACCESS_TOKEN",
            &format!(
                "export ESA_ACCESS_TOKEN=\"your-token-here\"\n      Get your token at https://{}.esa.io/user/tokens",
                team_name
            ),
        )?;

        let category_prefix = require_env(
            "ESA_CATEGORY_PREFIX",
            "export ESA_CATEGORY_PREFIX=\"日報/ラクガキ帳\"",
        )?;
        validate_category_prefix(&category_prefix)?;

        let post_name = env::var("ESA_POST_NAME").unwrap_or_else(|_| DEFAULT_POST_NAME.to_string());

        Ok(Config {
            team_name,
            access_token,
            category_prefix,
            post_name,
        })
    }
}

fn require_env(name: &'static str, hint: &str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingEnvVar {
        name,
        hint: format!("Hint: {}", hint),
    })
}

fn validate_category_prefix(value: &str) -> Result<(), ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::InvalidValue(
            "category_prefixは空にできません".to_string(),
        ));
    }
    if trimmed.starts_with('/') || trimmed.ends_with('/') {
        return Err(ConfigError::InvalidValue(
            "category_prefixの先頭/末尾に/は使えません".to_string(),
        ));
    }
    if trimmed.contains("..") {
        return Err(ConfigError::InvalidValue(
            "category_prefixに..は使えません".to_string(),
        ));
    }
    if trimmed.contains("//") {
        return Err(ConfigError::InvalidValue(
            "category_prefixに連続する//は使えません".to_string(),
        ));
    }
    Ok(())
}
