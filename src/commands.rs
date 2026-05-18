#[cfg(test)]
#[path = "commands_tests.rs"]
mod commands_tests;

use std::process;

use chrono::{FixedOffset, NaiveDate, Utc};
use serde_json::json;

use crate::client::EsaClient;
use crate::config::Config;
use crate::entry::{
    self, create_scratchpad_entry, entries_to_body, get_scratchpad_category,
    get_tags_including_weekday, parse_scratchpad_entries, remove_entry, replace_entry_text,
    sort_entries_by_timestamp, validate_no_duplicate_timestamp, TimestampId,
};
use crate::validator::{validate_post_text, validate_scratchpad_title};

const EXIT_ERROR: i32 = 1;

pub fn jst_offset() -> FixedOffset {
    FixedOffset::east_opt(9 * 3600).unwrap()
}

pub fn today_jst() -> NaiveDate {
    Utc::now().with_timezone(&jst_offset()).date_naive()
}

fn now_jst() -> chrono::DateTime<FixedOffset> {
    Utc::now().with_timezone(&jst_offset())
}

pub fn resolve_text_input(
    text: &Option<String>,
    text_file: &Option<String>,
) -> Result<String, String> {
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

    Err("テキストが指定されていません。--text または --text-file を使用してください".to_string())
}

pub fn parse_date(date_str: &Option<String>) -> Result<NaiveDate, String> {
    match date_str {
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|e| format!("日付の形式が不正です (YYYY-MM-DD): {}", e)),
        None => Ok(today_jst()),
    }
}

pub fn resolve_category_prefix(cli_prefix: &Option<String>, config: &Config) -> String {
    cli_prefix
        .clone()
        .unwrap_or_else(|| config.category_prefix.clone())
}

pub fn resolve_post_name(cli_name: &Option<String>, config: &Config) -> String {
    cli_name.clone().unwrap_or_else(|| config.post_name.clone())
}

pub fn make_unique_timestamp(ts: &TimestampId, entries: &[entry::ScratchpadEntry]) -> TimestampId {
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

fn print_validation_errors(
    json_mode: bool,
    action: &str,
    issues: &[crate::validator::ValidationIssue],
) {
    if json_mode {
        let messages: Vec<&str> = issues.iter().map(|i| i.message.as_str()).collect();
        let obj = json!({
            "success": false,
            "action": action,
            "error": "バリデーションエラー",
            "issues": messages,
        });
        eprintln!("{}", serde_json::to_string_pretty(&obj).unwrap());
    } else {
        for issue in issues {
            eprintln!("Validation: {}", issue.message);
        }
    }
}

pub fn cmd_add(
    text: &Option<String>,
    text_file: &Option<String>,
    timestamp: &Option<String>,
    date: &Option<String>,
    category_prefix: &Option<String>,
    json_mode: bool,
) {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            print_error(json_mode, "add", &e.to_string());
            process::exit(EXIT_ERROR);
        }
    };

    let input = match resolve_text_input(text, text_file) {
        Ok(t) => t,
        Err(e) => {
            print_error(json_mode, "add", &e);
            process::exit(EXIT_ERROR);
        }
    };

    let trimmed = input.trim();
    if trimmed.is_empty() {
        print_error(json_mode, "add", "テキストが空です");
        process::exit(EXIT_ERROR);
    }

    let issues = validate_post_text(trimmed);
    if !issues.is_empty() {
        print_validation_errors(json_mode, "add", &issues);
        process::exit(EXIT_ERROR);
    }

    let date = match parse_date(date) {
        Ok(d) => d,
        Err(e) => {
            print_error(json_mode, "add", &e);
            process::exit(EXIT_ERROR);
        }
    };

    let prefix = resolve_category_prefix(category_prefix, &config);
    let category = get_scratchpad_category(&prefix, &date);
    let post_name = resolve_post_name(&None, &config);

    let client = EsaClient::new(config.team_name.clone(), config.access_token.clone());

    let ts = match timestamp {
        Some(ts_str) => match TimestampId::new(ts_str) {
            Ok(id) => id,
            Err(e) => {
                print_error(
                    json_mode,
                    "add",
                    &format!("タイムスタンプIDが不正です: {}", e),
                );
                process::exit(EXIT_ERROR);
            }
        },
        None => TimestampId::from_datetime(&now_jst()),
    };

    let existing = match client.search_by_category(&category) {
        Ok(post) => post,
        Err(e) => {
            print_error(json_mode, "add", &format!("API検索エラー: {}", e));
            process::exit(EXIT_ERROR);
        }
    };

    match existing {
        Some(post) => {
            let mut entries = match parse_scratchpad_entries(&post.body_md) {
                Ok(e) => e,
                Err(e) => {
                    print_error(json_mode, "add", &format!("本文パースエラー: {}", e));
                    process::exit(EXIT_ERROR);
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
                        "add",
                        "エントリを投稿しました",
                        &updated.url,
                        updated.number,
                        Some(&unique_ts),
                    );
                }
                Err(e) => {
                    print_error(json_mode, "add", &format!("API更新エラー: {}", e));
                    process::exit(EXIT_ERROR);
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
                        "add",
                        "エントリを投稿しました",
                        &created.url,
                        created.number,
                        Some(&ts),
                    );
                }
                Err(e) => {
                    print_error(json_mode, "add", &format!("API作成エラー: {}", e));
                    process::exit(EXIT_ERROR);
                }
            }
        }
    }
}

pub fn cmd_edit(
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
            print_error(json_mode, "edit", &e.to_string());
            process::exit(EXIT_ERROR);
        }
    };

    let input = match resolve_text_input(text, text_file) {
        Ok(t) => t,
        Err(e) => {
            print_error(json_mode, "edit", &e);
            process::exit(EXIT_ERROR);
        }
    };

    let trimmed = input.trim();
    if trimmed.is_empty() {
        print_error(json_mode, "edit", "テキストが空です");
        process::exit(EXIT_ERROR);
    }

    let issues = validate_post_text(trimmed);
    if !issues.is_empty() {
        print_validation_errors(json_mode, "edit", &issues);
        process::exit(EXIT_ERROR);
    }

    let ts = match TimestampId::new(timestamp) {
        Ok(id) => id,
        Err(e) => {
            print_error(
                json_mode,
                "edit",
                &format!("タイムスタンプIDが不正です: {}", e),
            );
            process::exit(EXIT_ERROR);
        }
    };

    let date = match parse_date(date) {
        Ok(d) => d,
        Err(e) => {
            print_error(json_mode, "edit", &e);
            process::exit(EXIT_ERROR);
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
                "edit",
                &format!("指定日付の投稿が見つかりません: {}", category),
            );
            process::exit(EXIT_ERROR);
        }
        Err(e) => {
            print_error(json_mode, "edit", &format!("API検索エラー: {}", e));
            process::exit(EXIT_ERROR);
        }
    };

    let entries = match parse_scratchpad_entries(&post.body_md) {
        Ok(e) => e,
        Err(e) => {
            print_error(json_mode, "edit", &format!("本文パースエラー: {}", e));
            process::exit(EXIT_ERROR);
        }
    };

    let updated_entries = match replace_entry_text(&entries, &ts, trimmed) {
        Ok(e) => e,
        Err(e) => {
            print_error(
                json_mode,
                "edit",
                &format!("エントリが見つかりません: {}", e),
            );
            process::exit(EXIT_ERROR);
        }
    };

    let new_body = entries_to_body(&updated_entries);
    let tags = get_tags_including_weekday(&post.tags, &date);

    match client.update_post_body(post.number, &new_body, &tags, "エントリ修正") {
        Ok(updated) => {
            print_success(
                json_mode,
                "edit",
                "エントリを修正しました",
                &updated.url,
                updated.number,
                Some(&ts),
            );
        }
        Err(e) => {
            print_error(json_mode, "edit", &format!("API更新エラー: {}", e));
            process::exit(EXIT_ERROR);
        }
    }
}

pub fn cmd_delete(
    timestamp: &str,
    date: &Option<String>,
    category_prefix: &Option<String>,
    json_mode: bool,
) {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            print_error(json_mode, "delete", &e.to_string());
            process::exit(EXIT_ERROR);
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
            process::exit(EXIT_ERROR);
        }
    };

    let date = match parse_date(date) {
        Ok(d) => d,
        Err(e) => {
            print_error(json_mode, "delete", &e);
            process::exit(EXIT_ERROR);
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
            process::exit(EXIT_ERROR);
        }
        Err(e) => {
            print_error(json_mode, "delete", &format!("API検索エラー: {}", e));
            process::exit(EXIT_ERROR);
        }
    };

    let entries = match parse_scratchpad_entries(&post.body_md) {
        Ok(e) => e,
        Err(e) => {
            print_error(json_mode, "delete", &format!("本文パースエラー: {}", e));
            process::exit(EXIT_ERROR);
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
            process::exit(EXIT_ERROR);
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
            process::exit(EXIT_ERROR);
        }
    }
}

pub fn cmd_rename(
    name: &str,
    date: &Option<String>,
    category_prefix: &Option<String>,
    json_mode: bool,
) {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            print_error(json_mode, "rename", &e.to_string());
            process::exit(EXIT_ERROR);
        }
    };

    let issues = validate_scratchpad_title(name);
    if !issues.is_empty() {
        print_validation_errors(json_mode, "rename", &issues);
        process::exit(EXIT_ERROR);
    }

    let date = match parse_date(date) {
        Ok(d) => d,
        Err(e) => {
            print_error(json_mode, "rename", &e);
            process::exit(EXIT_ERROR);
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
                "rename",
                &format!("指定日付の投稿が見つかりません: {}", category),
            );
            process::exit(EXIT_ERROR);
        }
        Err(e) => {
            print_error(json_mode, "rename", &format!("API検索エラー: {}", e));
            process::exit(EXIT_ERROR);
        }
    };

    match client.update_post_name(post.number, name, "タイトル変更") {
        Ok(updated) => {
            print_success(
                json_mode,
                "rename",
                "タイトルを変更しました",
                &updated.url,
                updated.number,
                None,
            );
        }
        Err(e) => {
            print_error(json_mode, "rename", &format!("API更新エラー: {}", e));
            process::exit(EXIT_ERROR);
        }
    }
}
