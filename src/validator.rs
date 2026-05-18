#[cfg(test)]
#[path = "validator_tests.rs"]
mod validator_tests;

use regex::Regex;

/// バリデーション違反
pub struct ValidationIssue {
    /// 違反種別の識別子（プログラムから判別するために使用）
    pub code: &'static str,
    /// ユーザー向けエラーメッセージ
    pub message: String,
}

/// 投稿テキストのバリデーション
///
/// 7つのルールをチェックし、全ての違反を収集して返す。
pub fn validate_post_text(text: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // 1. 区切り線(---)チェック（テーブル行は除外）
    let separator_re = Regex::new(r"---").unwrap();
    let has_separator = text
        .lines()
        .any(|line| separator_re.is_match(line) && !line.contains('|'));
    if has_separator {
        issues.push(ValidationIssue {
            code: "separator",
            message: "区切り線(---)が含まれています".to_string(),
        });
    }

    // 2. ボールド体チェック
    let bold_re = Regex::new(r"\*\*[^*]+\*\*").unwrap();
    if bold_re.is_match(text) {
        issues.push(ValidationIssue {
            code: "bold",
            message: "ボールド体(**テキスト**)が使用されています".to_string(),
        });
    }

    // 3. Markdown見出しチェック
    let heading_re = Regex::new(r"(?m)^#{1,6}\s").unwrap();
    if heading_re.is_match(text) {
        issues.push(ValidationIssue {
            code: "heading",
            message: "Markdown見出し(#)が使用されています".to_string(),
        });
    }

    // 4. 全角コロンチェック
    if text.contains('\u{FF1A}') {
        issues.push(ValidationIssue {
            code: "fullwidth_colon",
            message: "全角コロン(：)が使用されています。半角コロン(:)を使ってください".to_string(),
        });
    }

    // 5. 全角括弧チェック
    if text.contains('\u{FF08}') || text.contains('\u{FF09}') {
        issues.push(ValidationIssue {
            code: "fullwidth_paren",
            message: "全角括弧が使用されています。半角括弧を使ってください".to_string(),
        });
    }

    // 6. 冒頭の時刻パターンチェック
    let leading_time_re = Regex::new(r"^(?:[01]?\d|2[0-3]):[0-5]\d").unwrap();
    if leading_time_re.is_match(text.trim_start()) {
        issues.push(ValidationIssue {
            code: "leading_time",
            message: "冒頭に時刻が含まれています。時刻はシステムが自動挿入するため不要です"
                .to_string(),
        });
    }

    // 7. 冒頭のリスト記法チェック
    let list_marker_re = Regex::new(r"^[-*]\s").unwrap();
    if list_marker_re.is_match(text.trim_start()) {
        issues.push(ValidationIssue {
            code: "list_marker",
            message:
                "冒頭にマークダウンリスト記法(- / *)が使用されています。タイムスタンプ挿入でスタイルが崩れるため使用できません"
                    .to_string(),
        });
    }

    issues
}

/// タイトルのバリデーション
///
/// 6つのルールをチェックし、全ての違反を収集して返す。
pub fn validate_scratchpad_title(title: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // 1. スラッシュチェック
    if title.contains('/') {
        issues.push(ValidationIssue {
            code: "slash",
            message: "スラッシュ(/)はカテゴリ区切りとして解釈されるため使用できません".to_string(),
        });
    }

    // 2. スペースチェック（半角・全角）
    if title.contains(' ') || title.contains('\u{3000}') {
        issues.push(ValidationIssue {
            code: "space",
            message: "スペース（半角・全角）は使用できません".to_string(),
        });
    }

    // 3. 先頭記号チェック
    if let Some(first_char) = title.chars().next() {
        if !is_allowed_leading_char(first_char) {
            issues.push(ValidationIssue {
                code: "leading_symbol",
                message: "先頭に記号は使用できません".to_string(),
            });
        }
    }

    // 4. ピリオドチェック
    if title.contains('.') || title.contains('\u{3002}') {
        issues.push(ValidationIssue {
            code: "period",
            message: "ピリオド(.・。)は使用できません。区切りには読点(、)を使ってください"
                .to_string(),
        });
    }

    // 5. 感嘆符・疑問符チェック
    if title.contains('!')
        || title.contains('?')
        || title.contains('\u{FF01}')
        || title.contains('\u{FF1F}')
    {
        issues.push(ValidationIssue {
            code: "exclamation_question",
            message: "感嘆符・疑問符(!?！？)は使用できません".to_string(),
        });
    }

    // 6. 改行文字チェック
    if title.contains('\n') || title.contains('\r') {
        issues.push(ValidationIssue {
            code: "newline",
            message: "改行文字は使用できません".to_string(),
        });
    }

    issues
}

/// 先頭文字として許可される文字かどうかを判定
///
/// 許可: 英数字、ひらがな(U+3040-U+309F)、漢字(U+4E00-U+9FFF)、
/// カタカナ(U+30A0-U+30FF)、半角カナ(U+FF65-U+FF9F)
fn is_allowed_leading_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || ('\u{3040}'..='\u{9FFF}').contains(&c)
        || ('\u{30A0}'..='\u{30FF}').contains(&c)
        || ('\u{FF65}'..='\u{FF9F}').contains(&c)
}
