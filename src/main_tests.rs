use super::*;

// --- resolve_text_input ---

#[test]
fn resolve_text_both_specified_error() {
    let result = resolve_text_input(&Some("hello".to_string()), &Some("file.txt".to_string()));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("同時に指定できません"));
}

#[test]
fn resolve_text_from_text_flag() {
    let result = resolve_text_input(&Some("テスト".to_string()), &None);
    assert_eq!(result.unwrap(), "テスト");
}

#[test]
fn resolve_text_from_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("esa_scratchpad_test_input.txt");
    std::fs::write(&path, "ファイルからのテキスト").unwrap();

    let result = resolve_text_input(&None, &Some(path.to_str().unwrap().to_string()));
    assert_eq!(result.unwrap(), "ファイルからのテキスト");

    std::fs::remove_file(&path).ok();
}

#[test]
fn resolve_text_file_not_found() {
    let result = resolve_text_input(&None, &Some("/nonexistent/path.txt".to_string()));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("ファイル読み込みに失敗しました"));
}

// --- parse_date ---

#[test]
fn parse_date_valid() {
    let result = parse_date(&Some("2026-05-18".to_string()));
    assert!(result.is_ok());
    let date = result.unwrap();
    assert_eq!(date.to_string(), "2026-05-18");
}

#[test]
fn parse_date_invalid_format() {
    let result = parse_date(&Some("2026/05/18".to_string()));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("日付の形式が不正です"));
}

#[test]
fn parse_date_none_returns_today() {
    let result = parse_date(&None);
    assert!(result.is_ok());
    let date = result.unwrap();
    assert_eq!(date, today_jst());
}

// --- make_unique_timestamp ---

#[test]
fn make_unique_no_conflict() {
    let ts = TimestampId::new("153000000000").unwrap();
    let entries = vec![];
    let result = make_unique_timestamp(&ts, &entries);
    assert_eq!(result.as_str(), "153000000000");
}

#[test]
fn make_unique_with_conflict() {
    let ts = TimestampId::new("153000000000").unwrap();
    let entries = vec![entry::ScratchpadEntry {
        timestamp_id: TimestampId::new("153000000000").unwrap(),
        text: "既存".to_string(),
    }];
    let result = make_unique_timestamp(&ts, &entries);
    assert_ne!(result.as_str(), "153000000000");
    assert_eq!(result.as_str(), "153000000001");
}

#[test]
fn make_unique_with_multiple_conflicts() {
    let ts = TimestampId::new("153000000000").unwrap();
    let entries = vec![
        entry::ScratchpadEntry {
            timestamp_id: TimestampId::new("153000000000").unwrap(),
            text: "既存1".to_string(),
        },
        entry::ScratchpadEntry {
            timestamp_id: TimestampId::new("153000000001").unwrap(),
            text: "既存2".to_string(),
        },
    ];
    let result = make_unique_timestamp(&ts, &entries);
    assert_eq!(result.as_str(), "153000000002");
}

// --- CLI argument parsing ---

#[test]
fn cli_write_with_text() {
    let cli = Cli::try_parse_from(["esa-scratchpad", "write", "--text", "テスト"]).unwrap();
    match cli.command {
        Commands::Write { text, .. } => assert_eq!(text, Some("テスト".to_string())),
        _ => panic!("expected Write command"),
    }
}

#[test]
fn cli_write_with_all_options() {
    let cli = Cli::try_parse_from([
        "esa-scratchpad",
        "write",
        "--text",
        "テスト",
        "--timestamp",
        "153000123456",
        "--date",
        "2026-05-18",
        "--category-prefix",
        "日報/テスト",
        "--post-name",
        "テスト帳",
        "--json",
    ])
    .unwrap();
    match cli.command {
        Commands::Write {
            text,
            timestamp,
            date,
            category_prefix,
            post_name,
            json,
            ..
        } => {
            assert_eq!(text, Some("テスト".to_string()));
            assert_eq!(timestamp, Some("153000123456".to_string()));
            assert_eq!(date, Some("2026-05-18".to_string()));
            assert_eq!(category_prefix, Some("日報/テスト".to_string()));
            assert_eq!(post_name, Some("テスト帳".to_string()));
            assert!(json);
        }
        _ => panic!("expected Write command"),
    }
}

#[test]
fn cli_write_short_flags() {
    let cli = Cli::try_parse_from([
        "esa-scratchpad",
        "write",
        "--text",
        "メモ",
        "-t",
        "120000000000",
        "-d",
        "2026-01-01",
        "-c",
        "prefix",
        "-n",
        "名前",
    ])
    .unwrap();
    match cli.command {
        Commands::Write {
            timestamp,
            date,
            category_prefix,
            post_name,
            ..
        } => {
            assert_eq!(timestamp, Some("120000000000".to_string()));
            assert_eq!(date, Some("2026-01-01".to_string()));
            assert_eq!(category_prefix, Some("prefix".to_string()));
            assert_eq!(post_name, Some("名前".to_string()));
        }
        _ => panic!("expected Write command"),
    }
}

#[test]
fn cli_update_requires_timestamp() {
    let result = Cli::try_parse_from(["esa-scratchpad", "update", "--text", "テスト"]);
    assert!(result.is_err());
}

#[test]
fn cli_update_with_required_args() {
    let cli = Cli::try_parse_from([
        "esa-scratchpad",
        "update",
        "--timestamp",
        "153000123456",
        "--text",
        "新しいテキスト",
    ])
    .unwrap();
    match cli.command {
        Commands::Update {
            timestamp, text, ..
        } => {
            assert_eq!(timestamp, "153000123456");
            assert_eq!(text, Some("新しいテキスト".to_string()));
        }
        _ => panic!("expected Update command"),
    }
}

#[test]
fn cli_delete_requires_timestamp() {
    let result = Cli::try_parse_from(["esa-scratchpad", "delete"]);
    assert!(result.is_err());
}

#[test]
fn cli_delete_with_timestamp() {
    let cli =
        Cli::try_parse_from(["esa-scratchpad", "delete", "--timestamp", "153000123456"]).unwrap();
    match cli.command {
        Commands::Delete { timestamp, .. } => {
            assert_eq!(timestamp, "153000123456");
        }
        _ => panic!("expected Delete command"),
    }
}

#[test]
fn cli_delete_with_all_options() {
    let cli = Cli::try_parse_from([
        "esa-scratchpad",
        "delete",
        "-t",
        "153000123456",
        "-d",
        "2026-05-18",
        "-c",
        "prefix",
        "--json",
    ])
    .unwrap();
    match cli.command {
        Commands::Delete {
            timestamp,
            date,
            category_prefix,
            json,
        } => {
            assert_eq!(timestamp, "153000123456");
            assert_eq!(date, Some("2026-05-18".to_string()));
            assert_eq!(category_prefix, Some("prefix".to_string()));
            assert!(json);
        }
        _ => panic!("expected Delete command"),
    }
}

#[test]
fn cli_title_requires_name() {
    let result = Cli::try_parse_from(["esa-scratchpad", "title"]);
    assert!(result.is_err());
}

#[test]
fn cli_title_with_name() {
    let cli = Cli::try_parse_from(["esa-scratchpad", "title", "--name", "新タイトル"]).unwrap();
    match cli.command {
        Commands::Title { name, .. } => {
            assert_eq!(name, "新タイトル");
        }
        _ => panic!("expected Title command"),
    }
}

#[test]
fn cli_title_with_all_options() {
    let cli = Cli::try_parse_from([
        "esa-scratchpad",
        "title",
        "--name",
        "新タイトル",
        "-d",
        "2026-05-18",
        "-c",
        "prefix",
        "--json",
    ])
    .unwrap();
    match cli.command {
        Commands::Title {
            name,
            date,
            category_prefix,
            json,
        } => {
            assert_eq!(name, "新タイトル");
            assert_eq!(date, Some("2026-05-18".to_string()));
            assert_eq!(category_prefix, Some("prefix".to_string()));
            assert!(json);
        }
        _ => panic!("expected Title command"),
    }
}

// --- resolve_category_prefix ---

#[test]
fn resolve_category_prefix_cli_overrides_config() {
    use std::env;

    env::set_var("ESA_TEAM_NAME", "test");
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "from-env");

    let config = Config::load().unwrap();
    let result = resolve_category_prefix(&Some("from-cli".to_string()), &config);
    assert_eq!(result, "from-cli");

    env::remove_var("ESA_TEAM_NAME");
    env::remove_var("ESA_ACCESS_TOKEN");
    env::remove_var("ESA_CATEGORY_PREFIX");
}

#[test]
fn resolve_category_prefix_falls_back_to_config() {
    use std::env;

    env::set_var("ESA_TEAM_NAME", "test");
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "from-env");

    let config = Config::load().unwrap();
    let result = resolve_category_prefix(&None, &config);
    assert_eq!(result, "from-env");

    env::remove_var("ESA_TEAM_NAME");
    env::remove_var("ESA_ACCESS_TOKEN");
    env::remove_var("ESA_CATEGORY_PREFIX");
}

// --- resolve_post_name ---

#[test]
fn resolve_post_name_cli_overrides_config() {
    use std::env;

    env::set_var("ESA_TEAM_NAME", "test");
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "prefix");

    let config = Config::load().unwrap();
    let result = resolve_post_name(&Some("カスタム名".to_string()), &config);
    assert_eq!(result, "カスタム名");

    env::remove_var("ESA_TEAM_NAME");
    env::remove_var("ESA_ACCESS_TOKEN");
    env::remove_var("ESA_CATEGORY_PREFIX");
}

#[test]
fn resolve_post_name_falls_back_to_config() {
    use std::env;

    env::set_var("ESA_TEAM_NAME", "test");
    env::set_var("ESA_ACCESS_TOKEN", "token");
    env::set_var("ESA_CATEGORY_PREFIX", "prefix");

    let config = Config::load().unwrap();
    let result = resolve_post_name(&None, &config);
    assert_eq!(result, "ラクガキ帳");

    env::remove_var("ESA_TEAM_NAME");
    env::remove_var("ESA_ACCESS_TOKEN");
    env::remove_var("ESA_CATEGORY_PREFIX");
}

// --- jst_offset ---

#[test]
fn jst_offset_is_plus_9() {
    let offset = jst_offset();
    assert_eq!(offset.local_minus_utc(), 9 * 3600);
}
