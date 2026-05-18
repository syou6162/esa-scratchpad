#[cfg(test)]
#[path = "entry_tests.rs"]
mod entry_tests;

use chrono::{DateTime, Datelike, FixedOffset, NaiveDate};
use regex::Regex;
use std::fmt;

const ENTRY_SEPARATOR: &str = "\n---";

/// バリデーション済みタイムスタンプID（HHMMSSffffff形式、12桁）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct TimestampId(String);

impl TimestampId {
    /// 文字列からバリデーション済みTimestampIdを生成
    pub fn new(value: &str) -> Result<Self, ValidationError> {
        Self::validate(value)?;
        Ok(Self(value.to_string()))
    }

    /// DateTime<FixedOffset>からTimestampIdを生成
    pub fn from_datetime(dt: &DateTime<FixedOffset>) -> Self {
        Self(dt.format("%H%M%S%6f").to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 表示用の "HH:MM" 文字列を返す
    pub fn display_time(&self) -> String {
        format!("{}:{}", &self.0[0..2], &self.0[2..4])
    }

    /// アンカーHTMLタグを生成
    pub fn to_anchor_html(&self) -> String {
        format!(
            r##"<a id="{id}" href="#{id}">{display}</a>"##,
            id = self.0,
            display = self.display_time()
        )
    }

    fn validate(value: &str) -> Result<(), ValidationError> {
        if value.len() != 12 || !value.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationError::InvalidFormat(format!(
                "must be 12 digits, got \"{}\"",
                value
            )));
        }

        let hours: u32 = value[0..2].parse().unwrap();
        let minutes: u32 = value[2..4].parse().unwrap();
        let seconds: u32 = value[4..6].parse().unwrap();

        if hours > 23 {
            return Err(ValidationError::InvalidFormat(format!(
                "hours out of range (0-23): {}",
                hours
            )));
        }
        if minutes > 59 {
            return Err(ValidationError::InvalidFormat(format!(
                "minutes out of range (0-59): {}",
                minutes
            )));
        }
        if seconds > 59 {
            return Err(ValidationError::InvalidFormat(format!(
                "seconds out of range (0-59): {}",
                seconds
            )));
        }

        Ok(())
    }
}

impl fmt::Display for TimestampId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// ラクガキ帳の1エントリ
#[derive(Debug, Clone, PartialEq)]
pub struct ScratchpadEntry {
    /// バリデーション済みタイムスタンプID
    pub timestamp_id: TimestampId,
    /// アンカーHTMLタグ全体（例: '<a id="153000123456" href="#153000123456">15:30</a>'）
    pub timestamp_html: String,
    /// エントリ本文（アンカータグ以降のテキスト）
    pub text: String,
}

#[derive(Debug, thiserror::Error)]
pub enum EntryError {
    #[error("entry not found: timestamp_id={0}")]
    NotFound(TimestampId),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("invalid timestamp ID format: {0}")]
    InvalidFormat(String),

    #[error("duplicate timestamp ID: {0}")]
    DuplicateTimestamp(TimestampId),
}

/// body_mdを行頭の `---` でsplitし、各ブロックからエントリを抽出
pub fn parse_scratchpad_entries(body_md: &str) -> Vec<ScratchpadEntry> {
    let normalized = body_md.replace("\r\n", "\n").replace('\r', "\n");

    let anchor_re = Regex::new(r##"(?s)^(<a id="(\d+)" href="#\d+">[^<]+</a>) (.*)"##).unwrap();

    normalized
        .split(ENTRY_SEPARATOR)
        .enumerate()
        .filter_map(|(i, block)| {
            let trimmed = if i == 0 {
                block.trim_end_matches('\n')
            } else {
                block.trim_matches('\n')
            };
            if trimmed.is_empty() {
                return None;
            }
            let caps = anchor_re.captures(trimmed)?;
            let timestamp_id = TimestampId::new(&caps[2]).ok()?;
            Some(ScratchpadEntry {
                timestamp_id,
                timestamp_html: caps[1].to_string(),
                text: caps[3].to_string(),
            })
        })
        .collect()
}

/// エントリリストをbody_md形式に再構成
pub fn entries_to_body(entries: &[ScratchpadEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    entries
        .iter()
        .map(|e| format!("{} {}\n\n---", e.timestamp_html, e.text))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// タイムスタンプIDとテキストからScratchpadEntryを生成
pub fn create_scratchpad_entry(timestamp_id: &TimestampId, text: &str) -> ScratchpadEntry {
    ScratchpadEntry {
        timestamp_id: timestamp_id.clone(),
        timestamp_html: timestamp_id.to_anchor_html(),
        text: text.to_string(),
    }
}

/// 指定タイムスタンプIDのエントリのテキストを置換
pub fn replace_entry_text(
    entries: &[ScratchpadEntry],
    timestamp_id: &TimestampId,
    new_text: &str,
) -> Result<Vec<ScratchpadEntry>, EntryError> {
    if !entries.iter().any(|e| e.timestamp_id == *timestamp_id) {
        return Err(EntryError::NotFound(timestamp_id.clone()));
    }

    Ok(entries
        .iter()
        .map(|e| {
            if e.timestamp_id == *timestamp_id {
                ScratchpadEntry {
                    timestamp_id: e.timestamp_id.clone(),
                    timestamp_html: e.timestamp_html.clone(),
                    text: new_text.to_string(),
                }
            } else {
                e.clone()
            }
        })
        .collect())
}

/// 指定タイムスタンプIDのエントリを削除
pub fn remove_entry(
    entries: &[ScratchpadEntry],
    timestamp_id: &TimestampId,
) -> Result<Vec<ScratchpadEntry>, EntryError> {
    if !entries.iter().any(|e| e.timestamp_id == *timestamp_id) {
        return Err(EntryError::NotFound(timestamp_id.clone()));
    }

    Ok(entries
        .iter()
        .filter(|e| e.timestamp_id != *timestamp_id)
        .cloned()
        .collect())
}

/// タイムスタンプIDの降順（新しい順）にソート
pub fn sort_entries_by_timestamp(entries: &mut [ScratchpadEntry]) {
    entries.sort_by(|a, b| b.timestamp_id.cmp(&a.timestamp_id));
}

/// タイムスタンプIDの重複チェック
pub fn validate_no_duplicate_timestamp(
    entries: &[ScratchpadEntry],
    timestamp_id: &TimestampId,
) -> Result<(), ValidationError> {
    if entries.iter().any(|e| e.timestamp_id == *timestamp_id) {
        return Err(ValidationError::DuplicateTimestamp(timestamp_id.clone()));
    }
    Ok(())
}

/// プレフィックスと日付からカテゴリパスを生成
pub fn get_scratchpad_category(prefix: &str, date: &NaiveDate) -> String {
    format!(
        "{}/{:04}/{:02}/{:02}",
        prefix,
        date.year(),
        date.month(),
        date.day()
    )
}

const WEEKDAY_TAGS: [&str; 7] = [
    "月曜日",
    "火曜日",
    "水曜日",
    "木曜日",
    "金曜日",
    "土曜日",
    "日曜日",
];

/// 日付から曜日タグを取得
pub fn get_weekday_tag(date: &NaiveDate) -> &'static str {
    let weekday = date.weekday().num_days_from_monday() as usize;
    WEEKDAY_TAGS[weekday]
}

/// 既存タグリストに曜日タグを追加（既に曜日タグがあれば追加しない）
pub fn get_tags_including_weekday(tags: &[String], date: &NaiveDate) -> Vec<String> {
    let weekday_tag = get_weekday_tag(date);

    if tags.iter().any(|t| WEEKDAY_TAGS.contains(&t.as_str())) {
        return tags.to_vec();
    }

    let mut result = tags.to_vec();
    result.push(weekday_tag.to_string());
    result
}
