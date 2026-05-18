use super::*;

// --- CLI argument parsing ---

#[test]
fn cli_add_with_text() {
    let cli = Cli::try_parse_from(["esa-scratchpad", "add", "--text", "テスト"]).unwrap();
    match cli.command {
        Commands::Add { text, .. } => assert_eq!(text, Some("テスト".to_string())),
        _ => panic!("expected Add command"),
    }
}

#[test]
fn cli_add_with_all_options() {
    let cli = Cli::try_parse_from([
        "esa-scratchpad",
        "add",
        "--text",
        "テスト",
        "--timestamp",
        "153000123456",
        "--date",
        "2026-05-18",
        "--category-prefix",
        "日報/テスト",
        "--json",
    ])
    .unwrap();
    match cli.command {
        Commands::Add {
            text,
            timestamp,
            date,
            category_prefix,
            json,
            ..
        } => {
            assert_eq!(text, Some("テスト".to_string()));
            assert_eq!(timestamp, Some("153000123456".to_string()));
            assert_eq!(date, Some("2026-05-18".to_string()));
            assert_eq!(category_prefix, Some("日報/テスト".to_string()));
            assert!(json);
        }
        _ => panic!("expected Add command"),
    }
}

#[test]
fn cli_add_short_flags() {
    let cli = Cli::try_parse_from([
        "esa-scratchpad",
        "add",
        "--text",
        "メモ",
        "-t",
        "120000000000",
        "-d",
        "2026-01-01",
        "-c",
        "prefix",
    ])
    .unwrap();
    match cli.command {
        Commands::Add {
            timestamp,
            date,
            category_prefix,
            ..
        } => {
            assert_eq!(timestamp, Some("120000000000".to_string()));
            assert_eq!(date, Some("2026-01-01".to_string()));
            assert_eq!(category_prefix, Some("prefix".to_string()));
        }
        _ => panic!("expected Add command"),
    }
}

#[test]
fn cli_edit_requires_timestamp() {
    let result = Cli::try_parse_from(["esa-scratchpad", "edit", "--text", "テスト"]);
    assert!(result.is_err());
}

#[test]
fn cli_edit_with_required_args() {
    let cli = Cli::try_parse_from([
        "esa-scratchpad",
        "edit",
        "--timestamp",
        "153000123456",
        "--text",
        "新しいテキスト",
    ])
    .unwrap();
    match cli.command {
        Commands::Edit {
            timestamp, text, ..
        } => {
            assert_eq!(timestamp, "153000123456");
            assert_eq!(text, Some("新しいテキスト".to_string()));
        }
        _ => panic!("expected Edit command"),
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
fn cli_rename_requires_name() {
    let result = Cli::try_parse_from(["esa-scratchpad", "rename"]);
    assert!(result.is_err());
}

#[test]
fn cli_rename_with_name() {
    let cli = Cli::try_parse_from(["esa-scratchpad", "rename", "--name", "新タイトル"]).unwrap();
    match cli.command {
        Commands::Rename { name, .. } => {
            assert_eq!(name, "新タイトル");
        }
        _ => panic!("expected Rename command"),
    }
}

#[test]
fn cli_rename_with_all_options() {
    let cli = Cli::try_parse_from([
        "esa-scratchpad",
        "rename",
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
        Commands::Rename {
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
        _ => panic!("expected Rename command"),
    }
}
