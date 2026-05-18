use super::*;
use chrono::{FixedOffset, NaiveDate, TimeZone};

// --- parse / entries_to_body round-trip ---

#[test]
fn round_trip_single_entry() {
    let body = "<a id=\"170000123456\" href=\"#170000123456\">17:00</a> 夕方のメモ\n\n---";
    let entries = parse_scratchpad_entries(body);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries_to_body(&entries), body);
}

#[test]
fn round_trip_multiple_entries() {
    let body = "\
<a id=\"170000123456\" href=\"#170000123456\">17:00</a> 夕方のメモ\n\n\
---\n\n\
<a id=\"130000654321\" href=\"#130000654321\">13:00</a> 昼のメモ\n\n\
---";
    let entries = parse_scratchpad_entries(body);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries_to_body(&entries), body);
}

// --- parse: empty / single / multiple ---

#[test]
fn parse_empty_body() {
    let entries = parse_scratchpad_entries("");
    assert!(entries.is_empty());
}

#[test]
fn parse_single_entry() {
    let body = "<a id=\"153000123456\" href=\"#153000123456\">15:30</a> テスト\n\n---";
    let entries = parse_scratchpad_entries(body);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].timestamp_id, "153000123456");
    assert_eq!(
        entries[0].timestamp_html,
        "<a id=\"153000123456\" href=\"#153000123456\">15:30</a>"
    );
    assert_eq!(entries[0].text, "テスト");
}

#[test]
fn parse_multiple_entries() {
    let body = "\
<a id=\"170000123456\" href=\"#170000123456\">17:00</a> 夕方のメモ\n\n\
---\n\n\
<a id=\"130000654321\" href=\"#130000654321\">13:00</a> 昼のメモ\n\n\
---";
    let entries = parse_scratchpad_entries(body);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].timestamp_id, "170000123456");
    assert_eq!(entries[0].text, "夕方のメモ");
    assert_eq!(entries[1].timestamp_id, "130000654321");
    assert_eq!(entries[1].text, "昼のメモ");
}

// --- CRLF / CR normalization ---

#[test]
fn parse_crlf_normalization() {
    let body = "<a id=\"100000000000\" href=\"#100000000000\">10:00</a> memo\r\n\r\n---\r\n";
    let entries = parse_scratchpad_entries(body);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].text, "memo");
}

#[test]
fn parse_cr_normalization() {
    let body = "<a id=\"100000000000\" href=\"#100000000000\">10:00</a> memo\r\r---\r";
    let entries = parse_scratchpad_entries(body);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].text, "memo");
}

// --- entries_to_body ---

#[test]
fn entries_to_body_empty() {
    assert_eq!(entries_to_body(&[]), "");
}

// --- create_scratchpad_entry ---

#[test]
fn create_entry_generates_correct_html() {
    let entry = create_scratchpad_entry("153000123456", "テストメモ");
    assert_eq!(entry.timestamp_id, "153000123456");
    assert_eq!(
        entry.timestamp_html,
        "<a id=\"153000123456\" href=\"#153000123456\">15:30</a>"
    );
    assert_eq!(entry.text, "テストメモ");
}

#[test]
fn create_entry_midnight() {
    let entry = create_scratchpad_entry("000000000000", "深夜メモ");
    assert_eq!(
        entry.timestamp_html,
        "<a id=\"000000000000\" href=\"#000000000000\">00:00</a>"
    );
}

// --- replace_entry_text ---

#[test]
fn replace_entry_text_success() {
    let entries = vec![
        create_scratchpad_entry("170000000000", "元テキスト"),
        create_scratchpad_entry("130000000000", "昼のメモ"),
    ];
    let result = replace_entry_text(&entries, "170000000000", "新テキスト").unwrap();
    assert_eq!(result[0].text, "新テキスト");
    assert_eq!(result[1].text, "昼のメモ");
}

#[test]
fn replace_entry_text_not_found() {
    let entries = vec![create_scratchpad_entry("170000000000", "テスト")];
    let err = replace_entry_text(&entries, "999999999999", "新テキスト").unwrap_err();
    assert!(matches!(err, EntryError::NotFound(_)));
}

// --- remove_entry ---

#[test]
fn remove_entry_success() {
    let entries = vec![
        create_scratchpad_entry("170000000000", "夕方"),
        create_scratchpad_entry("130000000000", "昼"),
    ];
    let result = remove_entry(&entries, "170000000000").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].timestamp_id, "130000000000");
}

#[test]
fn remove_entry_not_found() {
    let entries = vec![create_scratchpad_entry("170000000000", "テスト")];
    let err = remove_entry(&entries, "999999999999").unwrap_err();
    assert!(matches!(err, EntryError::NotFound(_)));
}

// --- sort_entries_by_timestamp ---

#[test]
fn sort_entries_descending() {
    let mut entries = vec![
        create_scratchpad_entry("090000000000", "朝"),
        create_scratchpad_entry("170000000000", "夕方"),
        create_scratchpad_entry("130000000000", "昼"),
    ];
    sort_entries_by_timestamp(&mut entries);
    assert_eq!(entries[0].timestamp_id, "170000000000");
    assert_eq!(entries[1].timestamp_id, "130000000000");
    assert_eq!(entries[2].timestamp_id, "090000000000");
}

// --- generate_timestamp_id ---

#[test]
fn generate_timestamp_id_format() {
    let jst = FixedOffset::east_opt(9 * 3600).unwrap();
    let dt = jst.with_ymd_and_hms(2026, 5, 18, 15, 30, 45).unwrap();
    let id = generate_timestamp_id(&dt);
    assert_eq!(&id[0..6], "153045");
    assert_eq!(id.len(), 12);
}

// --- validate_timestamp_id ---

#[test]
fn validate_timestamp_id_valid() {
    assert!(validate_timestamp_id("153045123456").is_ok());
    assert!(validate_timestamp_id("000000000000").is_ok());
    assert!(validate_timestamp_id("235959999999").is_ok());
}

#[test]
fn validate_timestamp_id_not_12_digits() {
    assert!(validate_timestamp_id("12345").is_err());
    assert!(validate_timestamp_id("1234567890123").is_err());
    assert!(validate_timestamp_id("").is_err());
}

#[test]
fn validate_timestamp_id_non_digits() {
    assert!(validate_timestamp_id("15304512345a").is_err());
    assert!(validate_timestamp_id("abcdefghijkl").is_err());
}

#[test]
fn validate_timestamp_id_hours_out_of_range() {
    assert!(validate_timestamp_id("240000000000").is_err());
    assert!(validate_timestamp_id("990000000000").is_err());
}

#[test]
fn validate_timestamp_id_minutes_out_of_range() {
    assert!(validate_timestamp_id("006000000000").is_err());
    assert!(validate_timestamp_id("009900000000").is_err());
}

#[test]
fn validate_timestamp_id_seconds_out_of_range() {
    assert!(validate_timestamp_id("000060000000").is_err());
    assert!(validate_timestamp_id("000099000000").is_err());
}

// --- validate_no_duplicate_timestamp ---

#[test]
fn validate_no_duplicate_no_conflict() {
    let entries = vec![create_scratchpad_entry("170000000000", "テスト")];
    assert!(validate_no_duplicate_timestamp(&entries, "130000000000").is_ok());
}

#[test]
fn validate_no_duplicate_conflict() {
    let entries = vec![create_scratchpad_entry("170000000000", "テスト")];
    let err = validate_no_duplicate_timestamp(&entries, "170000000000").unwrap_err();
    assert!(matches!(err, ValidationError::DuplicateTimestamp(_)));
}

// --- get_scratchpad_category ---

#[test]
fn scratchpad_category_format() {
    let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
    assert_eq!(
        get_scratchpad_category("scratchpad", &date),
        "scratchpad/2026/03/05"
    );
}

#[test]
fn scratchpad_category_double_digit_month() {
    let date = NaiveDate::from_ymd_opt(2026, 12, 25).unwrap();
    assert_eq!(get_scratchpad_category("日報", &date), "日報/2026/12/25");
}

// --- get_weekday_tag ---

#[test]
fn weekday_tag_monday() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 18).unwrap(); // Monday
    assert_eq!(get_weekday_tag(&date), "月曜日");
}

#[test]
fn weekday_tag_sunday() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap(); // Sunday
    assert_eq!(get_weekday_tag(&date), "日曜日");
}

#[test]
fn weekday_tag_all_days() {
    let expected = [
        "月曜日",
        "火曜日",
        "水曜日",
        "木曜日",
        "金曜日",
        "土曜日",
        "日曜日",
    ];
    for (i, &tag) in expected.iter().enumerate() {
        // 2026-05-18 is Monday, so +i gives each day of the week
        let date = NaiveDate::from_ymd_opt(2026, 5, 18 + i as u32).unwrap();
        assert_eq!(get_weekday_tag(&date), tag);
    }
}

// --- get_tags_including_weekday ---

#[test]
fn tags_including_weekday_adds_tag() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 18).unwrap();
    let tags = vec!["scratchpad".to_string()];
    let result = get_tags_including_weekday(&tags, &date);
    assert_eq!(result, vec!["scratchpad", "月曜日"]);
}

#[test]
fn tags_including_weekday_does_not_duplicate() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 18).unwrap();
    let tags = vec!["scratchpad".to_string(), "月曜日".to_string()];
    let result = get_tags_including_weekday(&tags, &date);
    assert_eq!(result, vec!["scratchpad", "月曜日"]);
}

#[test]
fn tags_including_weekday_does_not_add_if_other_weekday_exists() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 18).unwrap(); // Monday
    let tags = vec!["scratchpad".to_string(), "火曜日".to_string()];
    let result = get_tags_including_weekday(&tags, &date);
    assert_eq!(result, vec!["scratchpad", "火曜日"]);
}

#[test]
fn tags_including_weekday_empty_tags() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 18).unwrap();
    let tags: Vec<String> = vec![];
    let result = get_tags_including_weekday(&tags, &date);
    assert_eq!(result, vec!["月曜日"]);
}

// --- parse multiline entry text ---

#[test]
fn parse_entry_with_multiline_text() {
    let body = "<a id=\"170000123456\" href=\"#170000123456\">17:00</a> 1行目\n2行目\n3行目\n\n---";
    let entries = parse_scratchpad_entries(body);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].text, "1行目\n2行目\n3行目");
}
