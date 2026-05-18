use super::*;

#[test]
fn deserialize_search_response_with_posts() {
    let json = r#"{
        "total_count": 1,
        "posts": [
            {
                "number": 3714,
                "name": "2025/05/18のラクガキ",
                "body_md": "テスト本文",
                "tags": ["scratchpad", "日報"],
                "url": "https://yasuhisa.esa.io/posts/3714",
                "category": "日報/2025/05"
            }
        ]
    }"#;

    let response: SearchResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.total_count, 1);
    assert_eq!(response.posts.len(), 1);

    let post = &response.posts[0];
    assert_eq!(post.number, 3714);
    assert_eq!(post.name, "2025/05/18のラクガキ");
    assert_eq!(post.body_md, "テスト本文");
    assert_eq!(post.tags, vec!["scratchpad", "日報"]);
    assert_eq!(post.url, "https://yasuhisa.esa.io/posts/3714");
    assert_eq!(post.category, Some("日報/2025/05".to_string()));
}

#[test]
fn deserialize_search_response_empty() {
    let json = r#"{
        "total_count": 0,
        "posts": []
    }"#;

    let response: SearchResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.total_count, 0);
    assert!(response.posts.is_empty());
}

#[test]
fn deserialize_post_with_optional_fields_missing() {
    let json = r#"{
        "number": 100,
        "name": "テスト投稿",
        "body_md": "本文",
        "url": "https://yasuhisa.esa.io/posts/100"
    }"#;

    let post: Post = serde_json::from_str(json).unwrap();
    assert_eq!(post.number, 100);
    assert_eq!(post.name, "テスト投稿");
    assert_eq!(post.body_md, "本文");
    assert!(post.tags.is_empty());
    assert_eq!(post.url, "https://yasuhisa.esa.io/posts/100");
    assert_eq!(post.category, None);
}

#[test]
fn deserialize_post_ignores_unknown_fields() {
    let json = r##"{
        "number": 200,
        "name": "投稿名",
        "body_md": "# 本文",
        "tags": [],
        "url": "https://yasuhisa.esa.io/posts/200",
        "category": "memo",
        "created_at": "2025-05-18T10:00:00+09:00",
        "updated_at": "2025-05-18T11:00:00+09:00",
        "full_name": "memo/投稿名",
        "wip": false,
        "stars_count": 0,
        "watchers_count": 1,
        "star": false,
        "watch": false
    }"##;

    let post: Post = serde_json::from_str(json).unwrap();
    assert_eq!(post.number, 200);
    assert_eq!(post.name, "投稿名");
    assert_eq!(post.category, Some("memo".to_string()));
}

#[test]
fn deserialize_search_response_ignores_extra_fields() {
    let json = r#"{
        "total_count": 5,
        "prev_page": null,
        "next_page": 2,
        "page": 1,
        "per_page": 1,
        "max_per_page": 100,
        "posts": [
            {
                "number": 42,
                "name": "テスト",
                "body_md": "",
                "url": "https://yasuhisa.esa.io/posts/42"
            }
        ]
    }"#;

    let response: SearchResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.total_count, 5);
    assert_eq!(response.posts.len(), 1);
    assert_eq!(response.posts[0].number, 42);
}

#[test]
fn serialize_create_post_request() {
    let request = CreatePostRequest {
        post: CreatePostBody {
            name: "今日のラクガキ".to_string(),
            category: "日報/2025/05".to_string(),
            body_md: "テスト本文".to_string(),
            wip: false,
            tags: vec!["scratchpad".to_string()],
            message: "新規作成".to_string(),
        },
    };

    let json = serde_json::to_value(&request).unwrap();
    let post = &json["post"];
    assert_eq!(post["name"], "今日のラクガキ");
    assert_eq!(post["category"], "日報/2025/05");
    assert_eq!(post["body_md"], "テスト本文");
    assert_eq!(post["wip"], false);
    assert_eq!(post["tags"][0], "scratchpad");
    assert_eq!(post["message"], "新規作成");
}

#[test]
fn serialize_update_post_body_request() {
    let request = UpdatePostBodyRequest {
        post: UpdatePostBodyBody {
            body_md: "更新後の本文".to_string(),
            tags: vec!["scratchpad".to_string(), "更新".to_string()],
            message: "本文更新".to_string(),
        },
    };

    let json = serde_json::to_value(&request).unwrap();
    let post = &json["post"];
    assert_eq!(post["body_md"], "更新後の本文");
    assert_eq!(post["tags"][0], "scratchpad");
    assert_eq!(post["tags"][1], "更新");
    assert_eq!(post["message"], "本文更新");
}

#[test]
fn serialize_update_post_name_request() {
    let request = UpdatePostNameRequest {
        post: UpdatePostNameBody {
            name: "新しいタイトル".to_string(),
            message: "タイトル変更".to_string(),
            wip: false,
        },
    };

    let json = serde_json::to_value(&request).unwrap();
    let post = &json["post"];
    assert_eq!(post["name"], "新しいタイトル");
    assert_eq!(post["message"], "タイトル変更");
    assert_eq!(post["wip"], false);
}
