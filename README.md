# esa-scratchpad

[esa.io](https://esa.io) のラクガキ帳（日報スクラッチパッド）を操作する CLI ツール。

日付ごとのスクラッチパッド投稿に対して、エントリの追加・更新・削除やタイトル変更を行えます。

## セットアップ

### 必要なもの

- Rust 1.70+
- esa.io の API アクセストークン

### ビルド

```bash
cargo build --release
```

### 環境変数

| 変数名 | 必須 | 説明 |
|--------|------|------|
| `ESA_ACCESS_TOKEN` | Yes | esa.io API アクセストークン |
| `ESA_TEAM_NAME` | Yes | esa.io チーム名 |
| `ESA_CATEGORY_PREFIX` | No | カテゴリプレフィックス（デフォルト: `日報/ラクガキ帳`） |

## 使い方

### write — エントリを投稿

```bash
# テキストを直接指定
esa-scratchpad write --text "今日のメモ"

# ファイルから読み込み
esa-scratchpad write --text-file memo.txt

# stdin から入力（パイプ）
echo "パイプ入力" | esa-scratchpad write

# オプション指定
esa-scratchpad write --text "メモ" \
  --date 2026-05-18 \
  --timestamp 153000000000 \
  --category-prefix "日報/テスト" \
  --post-name "テスト帳" \
  --json
```

### update — エントリを修正

```bash
esa-scratchpad update --timestamp 153000123456 --text "修正テキスト"
```

### delete — エントリを削除

```bash
esa-scratchpad delete --timestamp 153000123456
```

### title — タイトルを変更

```bash
esa-scratchpad title --name "新しいタイトル"
```

### 共通オプション

| フラグ | 短縮 | 説明 | デフォルト |
|--------|------|------|-----------|
| `--date` | `-d` | 対象日付（YYYY-MM-DD） | 今日（JST） |
| `--category-prefix` | `-c` | カテゴリプレフィックス | 環境変数 `ESA_CATEGORY_PREFIX` |
| `--post-name` | `-n` | 投稿名（新規作成時、write のみ） | `ラクガキ帳` |
| `--json` | | JSON 形式で出力 | false |

### テキスト入力

テキスト入力は以下の優先順位で解決されます:

1. `--text` フラグ
2. `--text-file` フラグ（ファイルパス指定）
3. stdin（パイプ/リダイレクト）

`--text` と `--text-file` の同時指定はエラーになります。

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
  "action": "write"
}
```

### 終了コード

| コード | 意味 |
|--------|------|
| 0 | 成功 |
| 1 | バリデーションエラー |
| 2 | API エラー |
| 3 | 入力エラー |

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
