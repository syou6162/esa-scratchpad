use super::*;

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
