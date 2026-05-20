---
name: writing-scratchpad
description: esa-scratchpad CLIを使ってesa.ioのラクガキ帳に投稿・編集・削除・タイトル変更を行う手順
---

## 前提

- `esa-scratchpad` コマンドはPATHに入っており即実行可能。リポジトリのclone、cargo build、ソースコード探索は一切不要
- このスキルの情報だけで投稿・編集・削除が完結する

## 必要な環境変数

| 変数名 | 必須 | 説明 |
|--------|------|------|
| ESA_SCRATCHPAD_ACCESS_TOKEN | Yes | esa.io API アクセストークン(write権限が必要) |
| ESA_TEAM_NAME | Yes | esa.io チーム名 |
| ESA_CATEGORY_PREFIX | Yes | カテゴリプレフィックス(例: devin.ai/ラクガキ帳) |

## フォーマット制約

- `---`(区切り線)禁止
- `#`〜`######`(Markdown見出し)禁止
- `**太字**` 禁止
- 全角コロン(：)禁止 → 半角コロン(:)を使うこと
- 全角括弧(（）)禁止 → 半角括弧(())を使うこと
- 冒頭行にマークダウンリスト記法(`-` / `*`)禁止(2行目以降はOK)

## 品質ガイドライン

- 時刻はシステムが自動挿入するため、冒頭に「22:00」などと書く必要はない
- 日付もesa.ioで自動付与されるため、書く必要はない
- 箇条書きを使って情報を構造化すること
- 調査をした場合などは元記事へのリンクをmarkdownリンクとして入れること
- omiの会話ログをメモする場合、ConversationIDを必ず明記すること(省略せずフルUUID)

## 日付/時刻の扱い

- 投稿前に必ず `date '+%Y-%m-%d %H:%M %Z' --date='TZ="Asia/Tokyo" now'` を実行してJSTの現在日時を確認すること
- `--date`の日付計算は必ずJST(Asia/Tokyo)基準で行うこと

## CLIコマンド

```bash
# 書き込み
esa-scratchpad add --text "メモ内容" --json

# 日付とタイムスタンプを指定して書き込み
esa-scratchpad add --text "メモ内容" --date 2026-05-18 --timestamp 185700000000 --json

# 編集
esa-scratchpad edit --timestamp <timestamp_id> --text "修正内容" --json

# 削除
esa-scratchpad delete --timestamp <timestamp_id> --json

# タイトル変更
esa-scratchpad rename --name "新しいタイトル" --json
```

## 投稿後の確認・報告

- 投稿完了後、必ずesa.io APIで該当記事を取得し、投稿内容が意図通りに反映されているか確認すること
- ユーザーへの報告時は、タイムスタンプ付きパーマリンクを使うこと
  - 形式: `https://yasuhisa.esa.io/posts/<post_number>#<timestamp_id>`
