#[cfg(test)]
#[path = "validator_tests.rs"]
mod validator_tests;

use regex::Regex;

const SEPARATOR_MARK: &str = "---";

/// バリデーション違反の種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationCode {
    Separator,
    Bold,
    Heading,
    FullwidthColon,
    FullwidthParen,
    LeadingTime,
    ListMarker,
    Slash,
    Space,
    LeadingSymbol,
    Period,
    ExclamationQuestion,
    Newline,
}

/// バリデーション違反
pub struct ValidationIssue {
    /// 違反種別
    pub code: ValidationCode,
    /// ユーザー向けエラーメッセージ
    pub message: String,
}

/// 投稿テキストのバリデーション
///
/// 7つのルールをチェックし、全ての違反を収集して返す。
pub fn validate_post_text(text: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Some(issue) = check_separator(text) {
        issues.push(issue);
    }
    if let Some(issue) = check_bold(text) {
        issues.push(issue);
    }
    if let Some(issue) = check_heading(text) {
        issues.push(issue);
    }
    if let Some(issue) = check_fullwidth_colon(text) {
        issues.push(issue);
    }
    if let Some(issue) = check_fullwidth_paren(text) {
        issues.push(issue);
    }
    if let Some(issue) = check_leading_time(text) {
        issues.push(issue);
    }
    if let Some(issue) = check_list_marker(text) {
        issues.push(issue);
    }

    issues
}

/// タイトルのバリデーション
///
/// 6つのルールをチェックし、全ての違反を収集して返す。
pub fn validate_scratchpad_title(title: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Some(issue) = check_slash(title) {
        issues.push(issue);
    }
    if let Some(issue) = check_space(title) {
        issues.push(issue);
    }
    if let Some(issue) = check_leading_symbol(title) {
        issues.push(issue);
    }
    if let Some(issue) = check_period(title) {
        issues.push(issue);
    }
    if let Some(issue) = check_exclamation_question(title) {
        issues.push(issue);
    }
    if let Some(issue) = check_newline(title) {
        issues.push(issue);
    }

    issues
}

// --- validate_post_text 用の個別チェック関数 ---

fn check_separator(text: &str) -> Option<ValidationIssue> {
    let has_separator = text
        .lines()
        .any(|line| line.contains(SEPARATOR_MARK) && !line.contains('|'));
    if has_separator {
        Some(ValidationIssue {
            code: ValidationCode::Separator,
            message: format!("区切り線({})が含まれています", SEPARATOR_MARK),
        })
    } else {
        None
    }
}

fn check_bold(text: &str) -> Option<ValidationIssue> {
    let bold_re = Regex::new(r"\*\*[^*]+\*\*").unwrap();
    if bold_re.is_match(text) {
        Some(ValidationIssue {
            code: ValidationCode::Bold,
            message: "ボールド体(**テキスト**)が使用されています".to_string(),
        })
    } else {
        None
    }
}

fn check_heading(text: &str) -> Option<ValidationIssue> {
    let heading_re = Regex::new(r"(?m)^#{1,6}\s").unwrap();
    if heading_re.is_match(text) {
        Some(ValidationIssue {
            code: ValidationCode::Heading,
            message: "Markdown見出し(#)が使用されています".to_string(),
        })
    } else {
        None
    }
}

fn check_fullwidth_colon(text: &str) -> Option<ValidationIssue> {
    if text.contains('\u{FF1A}') {
        Some(ValidationIssue {
            code: ValidationCode::FullwidthColon,
            message: "全角コロン(：)が使用されています。半角コロン(:)を使ってください".to_string(),
        })
    } else {
        None
    }
}

fn check_fullwidth_paren(text: &str) -> Option<ValidationIssue> {
    if text.contains('\u{FF08}') || text.contains('\u{FF09}') {
        Some(ValidationIssue {
            code: ValidationCode::FullwidthParen,
            message: "全角括弧が使用されています。半角括弧を使ってください".to_string(),
        })
    } else {
        None
    }
}

fn check_leading_time(text: &str) -> Option<ValidationIssue> {
    let leading_time_re = Regex::new(r"^(?:[01]?\d|2[0-3]):[0-5]\d").unwrap();
    if leading_time_re.is_match(text.trim_start()) {
        Some(ValidationIssue {
            code: ValidationCode::LeadingTime,
            message: "冒頭に時刻が含まれています。時刻はシステムが自動挿入するため不要です"
                .to_string(),
        })
    } else {
        None
    }
}

fn check_list_marker(text: &str) -> Option<ValidationIssue> {
    let list_marker_re = Regex::new(r"^[-*]\s").unwrap();
    if list_marker_re.is_match(text.trim_start()) {
        Some(ValidationIssue {
            code: ValidationCode::ListMarker,
            message:
                "冒頭にマークダウンリスト記法(- / *)が使用されています。タイムスタンプ挿入でスタイルが崩れるため使用できません"
                    .to_string(),
        })
    } else {
        None
    }
}

// --- validate_scratchpad_title 用の個別チェック関数 ---

fn check_slash(title: &str) -> Option<ValidationIssue> {
    if title.contains('/') {
        Some(ValidationIssue {
            code: ValidationCode::Slash,
            message: "スラッシュ(/)はカテゴリ区切りとして解釈されるため使用できません".to_string(),
        })
    } else {
        None
    }
}

fn check_space(title: &str) -> Option<ValidationIssue> {
    if title.contains(' ') || title.contains('\u{3000}') {
        Some(ValidationIssue {
            code: ValidationCode::Space,
            message: "スペース（半角・全角）は使用できません".to_string(),
        })
    } else {
        None
    }
}

fn check_leading_symbol(title: &str) -> Option<ValidationIssue> {
    if let Some(first_char) = title.chars().next() {
        if !is_allowed_leading_char(first_char) {
            return Some(ValidationIssue {
                code: ValidationCode::LeadingSymbol,
                message: "先頭に記号は使用できません".to_string(),
            });
        }
    }
    None
}

fn check_period(title: &str) -> Option<ValidationIssue> {
    if title.contains('.') || title.contains('\u{3002}') {
        Some(ValidationIssue {
            code: ValidationCode::Period,
            message: "ピリオド(.・。)は使用できません。区切りには読点(、)を使ってください"
                .to_string(),
        })
    } else {
        None
    }
}

fn check_exclamation_question(title: &str) -> Option<ValidationIssue> {
    if title.contains('!')
        || title.contains('?')
        || title.contains('\u{FF01}')
        || title.contains('\u{FF1F}')
    {
        Some(ValidationIssue {
            code: ValidationCode::ExclamationQuestion,
            message: "感嘆符・疑問符(!?！？)は使用できません".to_string(),
        })
    } else {
        None
    }
}

fn check_newline(title: &str) -> Option<ValidationIssue> {
    if title.contains('\n') || title.contains('\r') {
        Some(ValidationIssue {
            code: ValidationCode::Newline,
            message: "改行文字は使用できません".to_string(),
        })
    } else {
        None
    }
}

/// 先頭文字として許可される文字かどうかを判定
///
/// 許可: 英数字、ひらがな(U+3040-U+309F)、カタカナ(U+30A0-U+30FF)、
/// 漢字(U+4E00-U+9FFF)、半角カナ(U+FF65-U+FF9F)
fn is_allowed_leading_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || ('\u{3040}'..='\u{309F}').contains(&c)
        || ('\u{30A0}'..='\u{30FF}').contains(&c)
        || ('\u{4E00}'..='\u{9FFF}').contains(&c)
        || ('\u{FF65}'..='\u{FF9F}').contains(&c)
}
