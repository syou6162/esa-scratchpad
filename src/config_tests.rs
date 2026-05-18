use super::*;
use serial_test::serial;
use std::env;

fn clear_env_vars() {
    env::remove_var("ESA_TEAM_NAME");
    env::remove_var("ESA_ACCESS_TOKEN");
    env::remove_var("ESA_CATEGORY_PREFIX");
    env::remove_var("ESA_POST_NAME");
}

fn set_all_required_env_vars() {
    env::set_var("ESA_TEAM_NAME", "myteam");
    env::set_var("ESA_ACCESS_TOKEN", "test-token-123");
    env::set_var("ESA_CATEGORY_PREFIX", "日報/ラクガキ帳");
}

// --- 全環境変数が設定されている場合 ---

#[test]
#[serial]
fn load_all_env_vars_set() {
    clear_env_vars();
    set_all_required_env_vars();

    let config = Config::load().unwrap();
    assert_eq!(config.team_name, "myteam");
    assert_eq!(config.access_token, "test-token-123");
    assert_eq!(config.category_prefix, "日報/ラクガキ帳");
    assert_eq!(config.post_name, "ラクガキ帳");

    clear_env_vars();
}

#[test]
#[serial]
fn load_all_env_vars_including_optional() {
    clear_env_vars();
    set_all_required_env_vars();
    env::set_var("ESA_POST_NAME", "カスタム名");

    let config = Config::load().unwrap();
    assert_eq!(config.post_name, "カスタム名");

    clear_env_vars();
}

// --- 必須環境変数が欠けている場合 ---

#[test]
#[serial]
fn load_missing_team_name() {
    clear_env_vars();
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "prefix");

    let err = Config::load().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("ESA_TEAM_NAME"), "got: {}", msg);
    assert!(msg.contains("Hint:"), "got: {}", msg);

    clear_env_vars();
}

#[test]
#[serial]
fn load_missing_access_token() {
    clear_env_vars();
    env::set_var("ESA_TEAM_NAME", "myteam");
    env::set_var("ESA_CATEGORY_PREFIX", "prefix");

    let err = Config::load().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("ESA_ACCESS_TOKEN"), "got: {}", msg);
    assert!(msg.contains("esa.io/user/tokens"), "got: {}", msg);

    clear_env_vars();
}

#[test]
#[serial]
fn load_missing_category_prefix() {
    clear_env_vars();
    env::set_var("ESA_TEAM_NAME", "myteam");
    env::set_var("ESA_ACCESS_TOKEN", "token");

    let err = Config::load().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("ESA_CATEGORY_PREFIX"), "got: {}", msg);

    clear_env_vars();
}

// --- デフォルト値 ---

#[test]
#[serial]
fn default_post_name_applied() {
    clear_env_vars();
    set_all_required_env_vars();

    let config = Config::load().unwrap();
    assert_eq!(config.post_name, "ラクガキ帳");

    clear_env_vars();
}

// --- category_prefix バリデーション ---

#[test]
#[serial]
fn category_prefix_valid() {
    clear_env_vars();
    env::set_var("ESA_TEAM_NAME", "myteam");
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "Claude Code/秘書の極秘メモ帳");

    let config = Config::load().unwrap();
    assert_eq!(config.category_prefix, "Claude Code/秘書の極秘メモ帳");

    clear_env_vars();
}

#[test]
#[serial]
fn category_prefix_leading_slash_rejected() {
    clear_env_vars();
    env::set_var("ESA_TEAM_NAME", "myteam");
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "/日報/ラクガキ帳");

    let err = Config::load().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("先頭/末尾に/は使えません"), "got: {}", msg);

    clear_env_vars();
}

#[test]
#[serial]
fn category_prefix_trailing_slash_rejected() {
    clear_env_vars();
    env::set_var("ESA_TEAM_NAME", "myteam");
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "日報/ラクガキ帳/");

    let err = Config::load().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("先頭/末尾に/は使えません"), "got: {}", msg);

    clear_env_vars();
}

#[test]
#[serial]
fn category_prefix_double_dot_rejected() {
    clear_env_vars();
    env::set_var("ESA_TEAM_NAME", "myteam");
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "日報/../secret");

    let err = Config::load().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("..は使えません"), "got: {}", msg);

    clear_env_vars();
}

#[test]
#[serial]
fn category_prefix_double_slash_rejected() {
    clear_env_vars();
    env::set_var("ESA_TEAM_NAME", "myteam");
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "日報//ラクガキ帳");

    let err = Config::load().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("連続する//は使えません"), "got: {}", msg);

    clear_env_vars();
}

#[test]
#[serial]
fn category_prefix_empty_rejected() {
    clear_env_vars();
    env::set_var("ESA_TEAM_NAME", "myteam");
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "  ");

    let err = Config::load().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("空にできません"), "got: {}", msg);

    clear_env_vars();
}

// --- Debug表示でトークンがマスクされる ---

#[test]
#[serial]
fn debug_display_masks_token() {
    clear_env_vars();
    set_all_required_env_vars();

    let config = Config::load().unwrap();
    let debug_output = format!("{:?}", config);
    assert!(debug_output.contains("[REDACTED]"), "got: {}", debug_output);
    assert!(
        !debug_output.contains("test-token-123"),
        "token leaked in debug: {}",
        debug_output
    );
    assert!(debug_output.contains("myteam"), "got: {}", debug_output);

    clear_env_vars();
}

// --- ConfigError のメッセージ確認 ---

#[test]
fn config_error_missing_env_var_message() {
    let err = ConfigError::MissingEnvVar {
        name: "ESA_ACCESS_TOKEN",
        hint: "Hint: export ESA_ACCESS_TOKEN=\"your-token-here\"".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("ESA_ACCESS_TOKEN"), "got: {}", msg);
    assert!(msg.contains("Hint:"), "got: {}", msg);
}

#[test]
fn config_error_file_error_message() {
    let err = ConfigError::FileError("file not found".to_string());
    assert!(err.to_string().contains("file not found"));
}

#[test]
fn config_error_invalid_value_message() {
    let err = ConfigError::InvalidValue("category_prefixは空にできません".to_string());
    assert!(err.to_string().contains("空にできません"));
}
