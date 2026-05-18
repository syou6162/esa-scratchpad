# esa-scratchpad

[esa.io](https://esa.io) のラクガキ帳（日報スクラッチパッド）を操作する CLI ツール。

日付ごとのスクラッチパッド投稿に対して、エントリの追加・編集・削除やタイトル変更を行えます。

## インストール

```bash
cargo install --git https://github.com/syou6162/esa-scratchpad.git
```

### 環境変数

| 変数名 | 必須 | 説明 |
|--------|------|------|
| `ESA_SCRATCHPAD_ACCESS_TOKEN` | Yes | esa.io API アクセストークン（write権限が必要） |
| `ESA_TEAM_NAME` | Yes | esa.io チーム名 |
| `ESA_CATEGORY_PREFIX` | Yes | カテゴリプレフィックス（例: `日報/ラクガキ帳`） |
| `ESA_POST_NAME` | No | 投稿名（デフォルト: `ラクガキ帳`） |

## 使い方

### add — エントリを投稿

```bash
# テキストを直接指定
esa-scratchpad add --text "今日のメモ"

# ファイルから読み込み
esa-scratchpad add --text-file memo.txt

# オプション指定
esa-scratchpad add --text "メモ" \
  --date 2026-05-18 \
  --timestamp 153000000000 \
  --category-prefix "日報/テスト" \
  --json
```

### edit — エントリを修正

```bash
esa-scratchpad edit --timestamp 153000123456 --text "修正テキスト"
```

### delete — エントリを削除

```bash
esa-scratchpad delete --timestamp 153000123456
```

### rename — タイトルを変更

```bash
esa-scratchpad rename --name "新しいタイトル"
```

### 共通オプション

| フラグ | 短縮 | 説明 | デフォルト |
|--------|------|------|-----------|
| `--date` | `-d` | 対象日付（YYYY-MM-DD） | 今日（JST） |
| `--category-prefix` | `-c` | カテゴリプレフィックス | 環境変数 `ESA_CATEGORY_PREFIX` |
| `--json` | | JSON 形式で出力 | false |

### テキスト入力

`--text` または `--text-file` のどちらかでテキストを指定します。同時指定はエラーになります。

### 出力

通常モード:
```
✓ エントリを投稿しました
  投稿: https://yasuhisa.esa.io/posts/12345
  タイムスタンプ: 15:30
```

JSON モード (`--json`):
```json
{
  "success": true,
  "post_url": "https://yasuhisa.esa.io/posts/12345",
  "post_number": 12345,
  "timestamp_id": "153000123456",
  "action": "add"
}
```

## 開発

```bash
# フォーマット
cargo fmt

# リント
cargo clippy -- -D warnings

# テスト
cargo test
```

## ライセンス

MIT
