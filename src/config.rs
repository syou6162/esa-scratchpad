#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;

use chrono::{FixedOffset, NaiveDate, Utc};
use std::env;
use std::fmt;

const JST_OFFSET_SECONDS: i32 = 9 * 3600;

/// CLI引数で渡されるオプション値
pub struct CliArgs {
    /// 操作対象の日付（YYYY/MM/DD形式）。未指定時はJSTの今日。
    pub date: Option<String>,
}

/// アプリケーション設定
///
/// 環境変数から構築される。category_prefix / post_name はCLIフラグでは変更できない（スコープ固定）。
pub struct Config {
    pub team_name: String,
    pub access_token: String,
    pub category_prefix: String,
    pub post_name: String,
    pub date: NaiveDate,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("team_name", &self.team_name)
            .field("access_token", &"[REDACTED]")
            .field("category_prefix", &self.category_prefix)
            .field("post_name", &self.post_name)
            .field("date", &self.date)
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
    /// 環境変数とCLI引数から設定を構築
    ///
    /// 必須値が欠けている場合はエラーを返す。
    /// category_prefix と post_name は環境変数のみで設定（スコープ固定）。
    /// date はCLIフラグで指定可能、未指定時はJSTの今日。
    pub fn load(cli_args: &CliArgs) -> Result<Config, ConfigError> {
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

        let date = resolve_date(&cli_args.date)?;

        Ok(Config {
            team_name,
            access_token,
            category_prefix,
            post_name,
            date,
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

fn resolve_date(date_str: &Option<String>) -> Result<NaiveDate, ConfigError> {
    match date_str {
        Some(s) => NaiveDate::parse_from_str(s, "%Y/%m/%d").map_err(|_| {
            ConfigError::InvalidValue(format!(
                "日付の形式が不正です: '{}'. YYYY/MM/DD形式で指定してください",
                s
            ))
        }),
        None => {
            let jst = FixedOffset::east_opt(JST_OFFSET_SECONDS).unwrap();
            Ok(Utc::now().with_timezone(&jst).date_naive())
        }
    }
}
