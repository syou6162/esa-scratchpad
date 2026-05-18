mod client;
mod config;
mod entry;
#[allow(dead_code)]
mod error;
mod validator;

#[cfg(test)]
mod main_tests;

use std::io::{self, IsTerminal, Read};
use std::process;

use chrono::{FixedOffset, NaiveDate, Utc};
use clap::{Parser, Subcommand};
use serde_json::json;

use client::EsaClient;
use config::Config;
use entry::{
    create_scratchpad_entry, entries_to_body, get_scratchpad_category, get_tags_including_weekday,
    parse_scratchpad_entries, remove_entry, replace_entry_text, sort_entries_by_timestamp,
    validate_no_duplicate_timestamp, TimestampId,
};
use validator::{validate_post_text, validate_scratchpad_title};

const EXIT_VALIDATION: i32 = 1;
const EXIT_API: i32 = 2;
const EXIT_INPUT: i32 = 3;

fn jst_offset() -> FixedOffset {
    FixedOffset::east_opt(9 * 3600).unwrap()
}

fn today_jst() -> NaiveDate {
    Utc::now().with_timezone(&jst_offset()).date_naive()
}

fn now_jst() -> chrono::DateTime<FixedOffset> {
    Utc::now().with_timezone(&jst_offset())
}

#[derive(Parser)]
#[command(name = "esa-scratchpad", about = "esa.io ラクガキ帳 CLI ツール")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// エントリを投稿
    Write {
        /// 投稿テキスト
        #[arg(long)]
        text: Option<String>,

        /// テキストファイルのパス
        #[arg(long, value_name = "PATH")]
        text_file: Option<String>,

        /// タイムスタンプID (HHMMSSffffff)
        #[arg(short = 't', long)]
        timestamp: Option<String>,

        /// 対象日付 (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// カテゴリプレフィックス
        #[arg(short = 'c', long)]
        category_prefix: Option<String>,

        /// 投稿名(新規作成時)
        #[arg(short = 'n', long)]
        post_name: Option<String>,

        /// JSON形式で出力
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// エントリを修正
    Update {
        /// 投稿テキスト
        #[arg(long)]
        text: Option<String>,

        /// テキストファイルのパス
        #[arg(long, value_name = "PATH")]
        text_file: Option<String>,

        /// 修正対象のタイムスタンプID (HHMMSSffffff)
        #[arg(short = 't', long)]
        timestamp: String,

        /// 対象日付 (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// カテゴリプレフィックス
        #[arg(short = 'c', long)]
        category_prefix: Option<String>,

        /// JSON形式で出力
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// エントリを削除
    Delete {
        /// 削除対象のタイムスタンプID (HHMMSSffffff)
        #[arg(short = 't', long)]
        timestamp: String,

        /// 対象日付 (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// カテゴリプレフィックス
        #[arg(short = 'c', long)]
        category_prefix: Option<String>,

        /// JSON形式で出力
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// タイトルを変更
    Title {
        /// 新しいタイトル
        #[arg(long)]
        name: String,

        /// 対象日付 (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// カテゴリプレフィックス
        #[arg(short = 'c', long)]
        category_prefix: Option<String>,

        /// JSON形式で出力
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

fn resolve_text_input(text: &Option<String>, text_file: &Option<String>) -> Result<String, String> {
    if text.is_some() && text_file.is_some() {
        return Err("--text と --text-file は同時に指定できません".to_string());
    }

    if let Some(t) = text {
        return Ok(t.clone());
    }

    if let Some(path) = text_file {
        return std::fs::read_to_string(path)
            .map_err(|e| format!("ファイル読み込みに失敗しました: {}: {}", path, e));
    }

    if io::stdin().is_terminal() {
        return Err(
            "テキストが指定されていません。--text, --text-file, またはstdinパイプを使用してください"
                .to_string(),
        );
    }

    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("stdinの読み込みに失敗しました: {}", e))?;
    Ok(buf)
}

fn parse_date(date_str: &Option<String>) -> Result<NaiveDate, String> {
    match date_str {
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|e| format!("日付の形式が不正です (YYYY-MM-DD): {}", e)),
        None => Ok(today_jst()),
    }
}

fn resolve_category_prefix(cli_prefix: &Option<String>, config: &Config) -> String {
    cli_prefix
        .clone()
        .unwrap_or_else(|| config.category_prefix.clone())
}

fn resolve_post_name(cli_name: &Option<String>, config: &Config) -> String {
    cli_name.clone().unwrap_or_else(|| config.post_name.clone())
}

fn make_unique_timestamp(ts: &TimestampId, entries: &[entry::ScratchpadEntry]) -> TimestampId {
    if validate_no_duplicate_timestamp(entries, ts).is_ok() {
        return ts.clone();
    }

    let base: u64 = ts.as_str().parse().unwrap_or(0);
    for offset in 1..=999999 {
        let candidate_val = base + offset;
        let candidate_str = format!("{:012}", candidate_val);
        if let Ok(candidate) = TimestampId::new(&candidate_str) {
            if validate_no_duplicate_timestamp(entries, &candidate).is_ok() {
                return candidate;
            }
        }
    }
    ts.clone()
}

fn print_success(
    json_mode: bool,
    action: &str,
    message: &str,
    post_url: &str,
    post_number: u64,
    timestamp_id: Option<&TimestampId>,
) {
    if json_mode {
        let mut obj = json!({
            "success": true,
            "post_url": post_url,
            "post_number": post_number,
            "action": action,
        });
        if let Some(ts) = timestamp_id {
            obj["timestamp_id"] = json!(ts.as_str());
        }
        println!("{}", serde_json::to_string_pretty(&obj).unwrap());
    } else {
        println!("\u{2713} {}", message);
        println!("  投稿: {}", post_url);
        if let Some(ts) = timestamp_id {
            println!("  タイムスタンプ: {}", ts.display_time());
        }
    }
}

fn print_error(json_mode: bool, action: &str, message: &str) {
    if json_mode {
        let obj = json!({
            "success": false,
            "action": action,
            "error": message,
        });
        eprintln!("{}", serde_json::to_string_pretty(&obj).unwrap());
    } else {
        eprintln!("Error: {}", message);
    }
}

fn cmd_write(
    text: &Option<String>,
    text_file: &Option<String>,
    timestamp: &Option<String>,
    date: &Option<String>,
    category_prefix: &Option<String>,
    post_name_opt: &Option<String>,
    json_mode: bool,
) {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            print_error(json_mode, "write", &e.to_string());
            process::exit(EXIT_INPUT);
        }
    };

    let input = match resolve_text_input(text, text_file) {
        Ok(t) => t,
        Err(e) => {
            print_error(json_mode, "write", &e);
            process::exit(EXIT_INPUT);
        }
    };

    let trimmed = input.trim();
    if trimmed.is_empty() {
        print_error(json_mode, "write", "テキストが空です");
        process::exit(EXIT_INPUT);
    }

    let issues = validate_post_text(trimmed);
    if !issues.is_empty() {
        for issue in &issues {
            eprintln!("Validation: {}", issue.message);
        }
        process::exit(EXIT_VALIDATION);
    }

    let date = match parse_date(date) {
        Ok(d) => d,
        Err(e) => {
            print_error(json_mode, "write", &e);
            process::exit(EXIT_INPUT);
        }
    };

    let prefix = resolve_category_prefix(category_prefix, &config);
    let category = get_scratchpad_category(&prefix, &date);
    let post_name = resolve_post_name(post_name_opt, &config);

    let client = EsaClient::new(config.team_name.clone(), config.access_token.clone());

    let ts = match timestamp {
        Some(ts_str) => match TimestampId::new(ts_str) {
            Ok(id) => id,
            Err(e) => {
                print_error(
                    json_mode,
                    "write",
                    &format!("タイムスタンプIDが不正です: {}", e),
                );
                process::exit(EXIT_INPUT);
            }
        },
        None => TimestampId::from_datetime(&now_jst()),
    };

    let existing = match client.search_by_category(&category) {
        Ok(post) => post,
        Err(e) => {
            print_error(json_mode, "write", &format!("API検索エラー: {}", e));
            process::exit(EXIT_API);
        }
    };

    match existing {
        Some(post) => {
            let mut entries = match parse_scratchpad_entries(&post.body_md) {
                Ok(e) => e,
                Err(e) => {
                    print_error(json_mode, "write", &format!("本文パースエラー: {}", e));
                    process::exit(EXIT_API);
                }
            };

            let unique_ts = make_unique_timestamp(&ts, &entries);
            let new_entry = create_scratchpad_entry(&unique_ts, trimmed);
            entries.push(new_entry);
            sort_entries_by_timestamp(&mut entries);

            let new_body = entries_to_body(&entries);
            let tags = get_tags_including_weekday(&post.tags, &date);

            match client.update_post_body(post.number, &new_body, &tags, "エントリ追加") {
                Ok(updated) => {
                    print_success(
                        json_mode,
                        "write",
                        "エントリを投稿しました",
                        &updated.url,
                        updated.number,
                        Some(&unique_ts),
                    );
                }
                Err(e) => {
                    print_error(json_mode, "write", &format!("API更新エラー: {}", e));
                    process::exit(EXIT_API);
                }
            }
        }
        None => {
            let new_entry = create_scratchpad_entry(&ts, trimmed);
            let body = entries_to_body(&[new_entry]);
            let tags = get_tags_including_weekday(&[], &date);

            match client.create_post(&post_name, &category, &body, false, &tags, "新規作成") {
                Ok(created) => {
                    print_success(
                        json_mode,
                        "write",
                        "エントリを投稿しました",
                        &created.url,
                        created.number,
                        Some(&ts),
                    );
                }
                Err(e) => {
                    print_error(json_mode, "write", &format!("API作成エラー: {}", e));
                    process::exit(EXIT_API);
                }
            }
        }
    }
}

fn cmd_update(
    text: &Option<String>,
    text_file: &Option<String>,
    timestamp: &str,
    date: &Option<String>,
    category_prefix: &Option<String>,
    json_mode: bool,
) {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            print_error(json_mode, "update", &e.to_string());
            process::exit(EXIT_INPUT);
        }
    };

    let input = match resolve_text_input(text, text_file) {
        Ok(t) => t,
        Err(e) => {
            print_error(json_mode, "update", &e);
            process::exit(EXIT_INPUT);
        }
    };

    let trimmed = input.trim();
    if trimmed.is_empty() {
        print_error(json_mode, "update", "テキストが空です");
        process::exit(EXIT_INPUT);
    }

    let issues = validate_post_text(trimmed);
    if !issues.is_empty() {
        for issue in &issues {
            eprintln!("Validation: {}", issue.message);
        }
        process::exit(EXIT_VALIDATION);
    }

    let ts = match TimestampId::new(timestamp) {
        Ok(id) => id,
        Err(e) => {
            print_error(
                json_mode,
                "update",
                &format!("タイムスタンプIDが不正です: {}", e),
            );
            process::exit(EXIT_INPUT);
        }
    };

    let date = match parse_date(date) {
        Ok(d) => d,
        Err(e) => {
            print_error(json_mode, "update", &e);
            process::exit(EXIT_INPUT);
        }
    };

    let prefix = resolve_category_prefix(category_prefix, &config);
    let category = get_scratchpad_category(&prefix, &date);
    let client = EsaClient::new(config.team_name.clone(), config.access_token.clone());

    let post = match client.search_by_category(&category) {
        Ok(Some(p)) => p,
        Ok(None) => {
            print_error(
                json_mode,
                "update",
                &format!("指定日付の投稿が見つかりません: {}", category),
            );
            process::exit(EXIT_API);
        }
        Err(e) => {
            print_error(json_mode, "update", &format!("API検索エラー: {}", e));
            process::exit(EXIT_API);
        }
    };

    let entries = match parse_scratchpad_entries(&post.body_md) {
        Ok(e) => e,
        Err(e) => {
            print_error(json_mode, "update", &format!("本文パースエラー: {}", e));
            process::exit(EXIT_API);
        }
    };

    let updated_entries = match replace_entry_text(&entries, &ts, trimmed) {
        Ok(e) => e,
        Err(e) => {
            print_error(
                json_mode,
                "update",
                &format!("エントリが見つかりません: {}", e),
            );
            process::exit(EXIT_INPUT);
        }
    };

    let new_body = entries_to_body(&updated_entries);
    let tags = get_tags_including_weekday(&post.tags, &date);

    match client.update_post_body(post.number, &new_body, &tags, "エントリ修正") {
        Ok(updated) => {
            print_success(
                json_mode,
                "update",
                "エントリを修正しました",
                &updated.url,
                updated.number,
                Some(&ts),
            );
        }
        Err(e) => {
            print_error(json_mode, "update", &format!("API更新エラー: {}", e));
            process::exit(EXIT_API);
        }
    }
}

fn cmd_delete(
    timestamp: &str,
    date: &Option<String>,
    category_prefix: &Option<String>,
    json_mode: bool,
) {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            print_error(json_mode, "delete", &e.to_string());
            process::exit(EXIT_INPUT);
        }
    };

    let ts = match TimestampId::new(timestamp) {
        Ok(id) => id,
        Err(e) => {
            print_error(
                json_mode,
                "delete",
                &format!("タイムスタンプIDが不正です: {}", e),
            );
            process::exit(EXIT_INPUT);
        }
    };

    let date = match parse_date(date) {
        Ok(d) => d,
        Err(e) => {
            print_error(json_mode, "delete", &e);
            process::exit(EXIT_INPUT);
        }
    };

    let prefix = resolve_category_prefix(category_prefix, &config);
    let category = get_scratchpad_category(&prefix, &date);
    let client = EsaClient::new(config.team_name.clone(), config.access_token.clone());

    let post = match client.search_by_category(&category) {
        Ok(Some(p)) => p,
        Ok(None) => {
            print_error(
                json_mode,
                "delete",
                &format!("指定日付の投稿が見つかりません: {}", category),
            );
            process::exit(EXIT_API);
        }
        Err(e) => {
            print_error(json_mode, "delete", &format!("API検索エラー: {}", e));
            process::exit(EXIT_API);
        }
    };

    let entries = match parse_scratchpad_entries(&post.body_md) {
        Ok(e) => e,
        Err(e) => {
            print_error(json_mode, "delete", &format!("本文パースエラー: {}", e));
            process::exit(EXIT_API);
        }
    };

    let remaining = match remove_entry(&entries, &ts) {
        Ok(e) => e,
        Err(e) => {
            print_error(
                json_mode,
                "delete",
                &format!("エントリが見つかりません: {}", e),
            );
            process::exit(EXIT_INPUT);
        }
    };

    let new_body = entries_to_body(&remaining);
    let tags = get_tags_including_weekday(&post.tags, &date);

    match client.update_post_body(post.number, &new_body, &tags, "エントリ削除") {
        Ok(updated) => {
            print_success(
                json_mode,
                "delete",
                "エントリを削除しました",
                &updated.url,
                updated.number,
                Some(&ts),
            );
        }
        Err(e) => {
            print_error(json_mode, "delete", &format!("API更新エラー: {}", e));
            process::exit(EXIT_API);
        }
    }
}

fn cmd_title(name: &str, date: &Option<String>, category_prefix: &Option<String>, json_mode: bool) {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            print_error(json_mode, "title", &e.to_string());
            process::exit(EXIT_INPUT);
        }
    };

    let issues = validate_scratchpad_title(name);
    if !issues.is_empty() {
        for issue in &issues {
            eprintln!("Validation: {}", issue.message);
        }
        process::exit(EXIT_VALIDATION);
    }

    let date = match parse_date(date) {
        Ok(d) => d,
        Err(e) => {
            print_error(json_mode, "title", &e);
            process::exit(EXIT_INPUT);
        }
    };

    let prefix = resolve_category_prefix(category_prefix, &config);
    let category = get_scratchpad_category(&prefix, &date);
    let client = EsaClient::new(config.team_name.clone(), config.access_token.clone());

    let post = match client.search_by_category(&category) {
        Ok(Some(p)) => p,
        Ok(None) => {
            print_error(
                json_mode,
                "title",
                &format!("指定日付の投稿が見つかりません: {}", category),
            );
            process::exit(EXIT_API);
        }
        Err(e) => {
            print_error(json_mode, "title", &format!("API検索エラー: {}", e));
            process::exit(EXIT_API);
        }
    };

    match client.update_post_name(post.number, name, "タイトル変更") {
        Ok(updated) => {
            print_success(
                json_mode,
                "title",
                "タイトルを変更しました",
                &updated.url,
                updated.number,
                None,
            );
        }
        Err(e) => {
            print_error(json_mode, "title", &format!("API更新エラー: {}", e));
            process::exit(EXIT_API);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Write {
            text,
            text_file,
            timestamp,
            date,
            category_prefix,
            post_name,
            json,
        } => cmd_write(
            &text,
            &text_file,
            &timestamp,
            &date,
            &category_prefix,
            &post_name,
            json,
        ),
        Commands::Update {
            text,
            text_file,
            timestamp,
            date,
            category_prefix,
            json,
        } => cmd_update(&text, &text_file, &timestamp, &date, &category_prefix, json),
        Commands::Delete {
            timestamp,
            date,
            category_prefix,
            json,
        } => cmd_delete(&timestamp, &date, &category_prefix, json),
        Commands::Title {
            name,
            date,
            category_prefix,
            json,
        } => cmd_title(&name, &date, &category_prefix, json),
    }
}
