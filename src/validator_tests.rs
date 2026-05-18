use super::*;

// --- validate_post_text ---

#[test]
fn post_text_normal_text_no_issues() {
    let issues = validate_post_text("普通のテキストです");
    assert!(issues.is_empty());
}

#[test]
fn post_text_empty_string_no_issues() {
    let issues = validate_post_text("");
    assert!(issues.is_empty());
}

#[test]
fn post_text_separator_detected() {
    let issues = validate_post_text("テスト\n---\n続き");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "separator");
}

#[test]
fn post_text_separator_in_table_allowed() {
    let issues = validate_post_text("| col1 | col2 |\n| --- | --- |\n| a | b |");
    let separator_issues: Vec<_> = issues.iter().filter(|i| i.code == "separator").collect();
    assert!(separator_issues.is_empty());
}

#[test]
fn post_text_bold_detected() {
    let issues = validate_post_text("これは**太字**です");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "bold");
}

#[test]
fn post_text_heading_detected() {
    let issues = validate_post_text("# 見出し");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "heading");
}

#[test]
fn post_text_heading_h3_detected() {
    let issues = validate_post_text("テスト\n### 小見出し");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "heading");
}

#[test]
fn post_text_fullwidth_colon_detected() {
    let issues = validate_post_text("項目：値");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "fullwidth_colon");
}

#[test]
fn post_text_fullwidth_paren_detected() {
    let issues = validate_post_text("テスト（括弧）です");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "fullwidth_paren");
}

#[test]
fn post_text_leading_time_detected() {
    let issues = validate_post_text("15:30 ミーティング開始");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "leading_time");
}

#[test]
fn post_text_leading_time_with_whitespace() {
    let issues = validate_post_text("  9:05 朝のメモ");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "leading_time");
}

#[test]
fn post_text_leading_time_boundary_23_59() {
    let issues = validate_post_text("23:59 深夜");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "leading_time");
}

#[test]
fn post_text_list_marker_hyphen_detected() {
    let issues = validate_post_text("- リスト項目");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "list_marker");
}

#[test]
fn post_text_list_marker_asterisk_detected() {
    let issues = validate_post_text("* リスト項目");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "list_marker");
}

#[test]
fn post_text_multiple_violations() {
    let issues = validate_post_text("# 見出し\n---\n**太字**テスト");
    assert!(issues.len() >= 3);
    let codes: Vec<&str> = issues.iter().map(|i| i.code).collect();
    assert!(codes.contains(&"separator"));
    assert!(codes.contains(&"bold"));
    assert!(codes.contains(&"heading"));
}

#[test]
fn post_text_time_in_middle_not_detected() {
    let issues = validate_post_text("ミーティングは15:30から");
    let time_issues: Vec<_> = issues.iter().filter(|i| i.code == "leading_time").collect();
    assert!(time_issues.is_empty());
}

#[test]
fn post_text_hash_without_space_not_heading() {
    let issues = validate_post_text("#hashtag");
    let heading_issues: Vec<_> = issues.iter().filter(|i| i.code == "heading").collect();
    assert!(heading_issues.is_empty());
}

// --- validate_scratchpad_title ---

#[test]
fn title_normal_japanese_no_issues() {
    let issues = validate_scratchpad_title("今日のラクガキ");
    assert!(issues.is_empty());
}

#[test]
fn title_normal_english_no_issues() {
    let issues = validate_scratchpad_title("TodaysMemo");
    assert!(issues.is_empty());
}

#[test]
fn title_normal_number_no_issues() {
    let issues = validate_scratchpad_title("2026年メモ");
    assert!(issues.is_empty());
}

#[test]
fn title_normal_katakana_start_no_issues() {
    let issues = validate_scratchpad_title("カタカナタイトル");
    assert!(issues.is_empty());
}

#[test]
fn title_slash_detected() {
    let issues = validate_scratchpad_title("カテゴリ/タイトル");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "slash");
}

#[test]
fn title_halfwidth_space_detected() {
    let issues = validate_scratchpad_title("hello world");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "space");
}

#[test]
fn title_fullwidth_space_detected() {
    let issues = validate_scratchpad_title("hello\u{3000}world");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "space");
}

#[test]
fn title_leading_symbol_detected() {
    let issues = validate_scratchpad_title("#タイトル");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "leading_symbol");
}

#[test]
fn title_symbol_in_middle_ok() {
    let issues = validate_scratchpad_title("タイトル#含む");
    let leading_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "leading_symbol")
        .collect();
    assert!(leading_issues.is_empty());
}

#[test]
fn title_period_dot_detected() {
    let issues = validate_scratchpad_title("タイトル.区切り");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "period");
}

#[test]
fn title_period_maru_detected() {
    let issues = validate_scratchpad_title("タイトル。区切り");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "period");
}

#[test]
fn title_exclamation_detected() {
    let issues = validate_scratchpad_title("すごい!");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "exclamation_question");
}

#[test]
fn title_question_detected() {
    let issues = validate_scratchpad_title("なぜ?");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "exclamation_question");
}

#[test]
fn title_fullwidth_exclamation_detected() {
    let issues = validate_scratchpad_title("すごい！");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "exclamation_question");
}

#[test]
fn title_fullwidth_question_detected() {
    let issues = validate_scratchpad_title("なぜ？");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "exclamation_question");
}

#[test]
fn title_newline_detected() {
    let issues = validate_scratchpad_title("タイトル\n改行");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "newline");
}

#[test]
fn title_cr_detected() {
    let issues = validate_scratchpad_title("タイトル\r改行");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "newline");
}

#[test]
fn title_halfwidth_katakana_start_ok() {
    let issues = validate_scratchpad_title("\u{FF76}\u{FF80}\u{FF76}\u{FF85}");
    let leading_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "leading_symbol")
        .collect();
    assert!(leading_issues.is_empty());
}

#[test]
fn title_multiple_violations() {
    let issues = validate_scratchpad_title("#タイトル/パス 区切り.終");
    let codes: Vec<&str> = issues.iter().map(|i| i.code).collect();
    assert!(codes.contains(&"leading_symbol"));
    assert!(codes.contains(&"slash"));
    assert!(codes.contains(&"space"));
    assert!(codes.contains(&"period"));
}
