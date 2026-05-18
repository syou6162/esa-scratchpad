#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;

use std::env;
use std::fmt;

/// CLI引数で渡されるオプション値
pub struct CliArgs {
    pub category_prefix: Option<String>,
    pub post_name: Option<String>,
}

/// アプリケーション設定
///
/// 優先順位: CLIフラグ > 環境変数 > デフォルト値
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
}

const DEFAULT_POST_NAME: &str = "ラクガキ帳";

impl Config {
    /// 環境変数とCLI引数から設定を構築
    ///
    /// 必須値が欠けている場合はエラーを返す。
    /// 優先順位: CLIフラグ > 環境変数 > デフォルト値
    pub fn load(cli_args: &CliArgs) -> Result<Config, ConfigError> {
        let team_name = require_env("ESA_TEAM_NAME", "export ESA_TEAM_NAME=\"your-team-name\"")?;
        let access_token = require_env(
            "ESA_ACCESS_TOKEN",
            &format!(
                "export ESA_ACCESS_TOKEN=\"your-token-here\"\n      Get your token at https://{}.esa.io/user/tokens",
                team_name
            ),
        )?;

        let category_prefix = match &cli_args.category_prefix {
            Some(v) => v.clone(),
            None => require_env(
                "ESA_CATEGORY_PREFIX",
                "export ESA_CATEGORY_PREFIX=\"日報/ラクガキ帳\"",
            )?,
        };

        let post_name = match &cli_args.post_name {
            Some(v) => v.clone(),
            None => env::var("ESA_POST_NAME").unwrap_or_else(|_| DEFAULT_POST_NAME.to_string()),
        };

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
