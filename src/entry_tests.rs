use super::*;
use chrono::{FixedOffset, NaiveDate, TimeZone};
use indoc::indoc;

fn tid(s: &str) -> TimestampId {
    TimestampId::new(s).unwrap()
}

// --- TimestampId ---

#[test]
fn timestamp_id_new_valid() {
    assert!(TimestampId::new("153045123456").is_ok());
    assert!(TimestampId::new("000000000000").is_ok());
    assert!(TimestampId::new("235959999999").is_ok());
}

#[test]
fn timestamp_id_new_invalid() {
    assert!(TimestampId::new("12345").is_err());
    assert!(TimestampId::new("1234567890123").is_err());
    assert!(TimestampId::new("").is_err());
    assert!(TimestampId::new("15304512345a").is_err());
    assert!(TimestampId::new("abcdefghijkl").is_err());
}

#[test]
fn timestamp_id_hours_out_of_range() {
    assert!(TimestampId::new("240000000000").is_err());
    assert!(TimestampId::new("990000000000").is_err());
}

#[test]
fn timestamp_id_minutes_out_of_range() {
    assert!(TimestampId::new("006000000000").is_err());
    assert!(TimestampId::new("009900000000").is_err());
}

#[test]
fn timestamp_id_seconds_out_of_range() {
    assert!(TimestampId::new("000060000000").is_err());
    assert!(TimestampId::new("000099000000").is_err());
}

#[test]
fn timestamp_id_from_datetime() {
    let jst = FixedOffset::east_opt(9 * 3600).unwrap();
    let dt = jst.with_ymd_and_hms(2026, 5, 18, 15, 30, 45).unwrap();
    let id = TimestampId::from_datetime(&dt);
    assert_eq!(&id.as_str()[0..6], "153045");
    assert_eq!(id.as_str().len(), 12);
}

#[test]
fn timestamp_id_display_time() {
    let id = tid("153045123456");
    assert_eq!(id.display_time(), "15:30");

    let id = tid("000000000000");
    assert_eq!(id.display_time(), "00:00");
}

#[test]
fn timestamp_id_to_anchor_html() {
    let id = tid("153000123456");
    assert_eq!(
        id.to_anchor_html(),
        r##"<a id="153000123456" href="#153000123456">15:30</a>"##
    );
}

#[test]
fn timestamp_id_ordering() {
    let a = tid("090000000000");
    let b = tid("130000000000");
    let c = tid("170000000000");
    assert!(a < b);
    assert!(b < c);
}

// --- parse / entries_to_body round-trip ---

#[test]
fn round_trip_single_entry() {
    let body = indoc! {r##"
        <a id="170000123456" href="#170000123456">17:00</a> 夕方のメモ

        ---"##
    };
    let entries = parse_scratchpad_entries(body).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries_to_body(&entries), body);
}

#[test]
fn round_trip_multiple_entries() {
    let body = indoc! {r##"
        <a id="170000123456" href="#170000123456">17:00</a> 夕方のメモ

        ---

        <a id="130000654321" href="#130000654321">13:00</a> 昼のメモ

        ---"##
    };
    let entries = parse_scratchpad_entries(body).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries_to_body(&entries), body);
}

// --- parse: empty / single / multiple ---

#[test]
fn parse_empty_body() {
    let entries = parse_scratchpad_entries("").unwrap();
    assert!(entries.is_empty());
}

#[test]
fn parse_single_entry() {
    let body = indoc! {r##"
        <a id="153000123456" href="#153000123456">15:30</a> テスト

        ---"##
    };
    let entries = parse_scratchpad_entries(body).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].timestamp_id, tid("153000123456"));
    assert_eq!(entries[0].text, "テスト");
}

#[test]
fn parse_multiple_entries() {
    let body = indoc! {r##"
        <a id="170000123456" href="#170000123456">17:00</a> 夕方のメモ

        ---

        <a id="130000654321" href="#130000654321">13:00</a> 昼のメモ

        ---"##
    };
    let entries = parse_scratchpad_entries(body).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].timestamp_id, tid("170000123456"));
    assert_eq!(entries[0].text, "夕方のメモ");
    assert_eq!(entries[1].timestamp_id, tid("130000654321"));
    assert_eq!(entries[1].text, "昼のメモ");
}

// --- CRLF / CR normalization ---

#[test]
fn parse_crlf_normalization() {
    let body = "<a id=\"100000000000\" href=\"#100000000000\">10:00</a> memo\r\n\r\n---\r\n";
    let entries = parse_scratchpad_entries(body).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].text, "memo");
}

#[test]
fn parse_cr_normalization() {
    let body = "<a id=\"100000000000\" href=\"#100000000000\">10:00</a> memo\r\r---\r";
    let entries = parse_scratchpad_entries(body).unwrap();
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
fn create_entry_generates_correct_fields() {
    let id = tid("153000123456");
    let entry = create_scratchpad_entry(&id, "テストメモ");
    assert_eq!(entry.timestamp_id, id);
    assert_eq!(entry.text, "テストメモ");
}

#[test]
fn create_entry_midnight() {
    let id = tid("000000000000");
    let entry = create_scratchpad_entry(&id, "深夜メモ");
    assert_eq!(entry.timestamp_id, id);
    assert_eq!(entry.text, "深夜メモ");
}

// --- replace_entry_text ---

#[test]
fn replace_entry_text_success() {
    let id1 = tid("170000000000");
    let id2 = tid("130000000000");
    let entries = vec![
        create_scratchpad_entry(&id1, "元テキスト"),
        create_scratchpad_entry(&id2, "昼のメモ"),
    ];
    let result = replace_entry_text(&entries, &id1, "新テキスト").unwrap();
    assert_eq!(result[0].text, "新テキスト");
    assert_eq!(result[1].text, "昼のメモ");
}

#[test]
fn replace_entry_text_not_found() {
    let id = tid("170000000000");
    let entries = vec![create_scratchpad_entry(&id, "テスト")];
    let not_found = tid("120000000000");
    let err = replace_entry_text(&entries, &not_found, "新テキスト").unwrap_err();
    assert!(matches!(err, EntryError::NotFound(_)));
}

// --- remove_entry ---

#[test]
fn remove_entry_success() {
    let id1 = tid("170000000000");
    let id2 = tid("130000000000");
    let entries = vec![
        create_scratchpad_entry(&id1, "夕方"),
        create_scratchpad_entry(&id2, "昼"),
    ];
    let result = remove_entry(&entries, &id1).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].timestamp_id, id2);
}

#[test]
fn remove_entry_not_found() {
    let id = tid("170000000000");
    let entries = vec![create_scratchpad_entry(&id, "テスト")];
    let not_found = tid("120000000000");
    let err = remove_entry(&entries, &not_found).unwrap_err();
    assert!(matches!(err, EntryError::NotFound(_)));
}

// --- sort_entries_by_timestamp ---

#[test]
fn sort_entries_descending() {
    let id_morning = tid("090000000000");
    let id_noon = tid("130000000000");
    let id_evening = tid("170000000000");
    let mut entries = vec![
        create_scratchpad_entry(&id_morning, "朝"),
        create_scratchpad_entry(&id_evening, "夕方"),
        create_scratchpad_entry(&id_noon, "昼"),
    ];
    sort_entries_by_timestamp(&mut entries);
    assert_eq!(entries[0].timestamp_id, id_evening);
    assert_eq!(entries[1].timestamp_id, id_noon);
    assert_eq!(entries[2].timestamp_id, id_morning);
}

// --- validate_no_duplicate_timestamp ---

#[test]
fn validate_no_duplicate_no_conflict() {
    let id = tid("170000000000");
    let entries = vec![create_scratchpad_entry(&id, "テスト")];
    let other = tid("130000000000");
    assert!(validate_no_duplicate_timestamp(&entries, &other).is_ok());
}

#[test]
fn validate_no_duplicate_conflict() {
    let id = tid("170000000000");
    let entries = vec![create_scratchpad_entry(&id, "テスト")];
    let err = validate_no_duplicate_timestamp(&entries, &id).unwrap_err();
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
    let date = NaiveDate::from_ymd_opt(2026, 5, 18).unwrap();
    assert_eq!(get_weekday_tag(&date), "月曜日");
}

#[test]
fn weekday_tag_sunday() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
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
    let date = NaiveDate::from_ymd_opt(2026, 5, 18).unwrap();
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

// --- parse: strict error on invalid timestamp ---

#[test]
fn parse_errors_on_invalid_timestamp_in_anchor() {
    let body = indoc! {r##"
        <a id="990000000000" href="#990000000000">99:00</a> 壊れたエントリ

        ---"##
    };
    let err = parse_scratchpad_entries(body).unwrap_err();
    assert!(matches!(err, EntryError::InvalidTimestamp { .. }));
}

#[test]
fn parse_skips_non_anchor_blocks() {
    let body = indoc! {r##"
        <a id="170000123456" href="#170000123456">17:00</a> 正常エントリ

        ---

        これはアンカーのないブロック

        ---"##
    };
    let entries = parse_scratchpad_entries(body).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].timestamp_id, tid("170000123456"));
}

#[test]
fn parse_consecutive_separators() {
    let body = indoc! {r##"
        <a id="170000123456" href="#170000123456">17:00</a> first

        ---

        ---

        <a id="130000000000" href="#130000000000">13:00</a> second

        ---"##
    };
    let entries = parse_scratchpad_entries(body).unwrap();
    assert_eq!(entries.len(), 2);
}

// --- parse multiline entry text ---

#[test]
fn parse_entry_with_multiline_text() {
    let body = indoc! {r##"
        <a id="170000123456" href="#170000123456">17:00</a> 1行目
        2行目
        3行目

        ---"##
    };
    let entries = parse_scratchpad_entries(body).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].text, "1行目\n2行目\n3行目");
}
