mod client;
mod commands;
mod config;
mod entry;
#[allow(dead_code)]
mod error;
mod validator;

#[cfg(test)]
mod main_tests;

use clap::{Parser, Subcommand};

use commands::{cmd_add, cmd_delete, cmd_edit, cmd_rename};

#[derive(Parser)]
#[command(name = "esa-scratchpad", about = "esa.io ラクガキ帳 CLI ツール")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// エントリを投稿
    Add {
        /// 投稿テキスト
        #[arg(long)]
        text: Option<String>,

        /// テキストファイルのパス
        #[arg(long, value_name = "PATH")]
        text_file: Option<String>,

        /// タイムスタンプID (HHMMSSffffff)
        #[arg(short = 't', long)]
        timestamp: Option<String>,

        /// 対象日付 (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// カテゴリプレフィックス
        #[arg(short = 'c', long)]
        category_prefix: Option<String>,

        /// 投稿名(新規作成時)
        #[arg(short = 'n', long)]
        post_name: Option<String>,

        /// JSON形式で出力
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// エントリを修正
    Edit {
        /// 投稿テキスト
        #[arg(long)]
        text: Option<String>,

        /// テキストファイルのパス
        #[arg(long, value_name = "PATH")]
        text_file: Option<String>,

        /// 修正対象のタイムスタンプID (HHMMSSffffff)
        #[arg(short = 't', long)]
        timestamp: String,

        /// 対象日付 (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// カテゴリプレフィックス
        #[arg(short = 'c', long)]
        category_prefix: Option<String>,

        /// JSON形式で出力
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// エントリを削除
    Delete {
        /// 削除対象のタイムスタンプID (HHMMSSffffff)
        #[arg(short = 't', long)]
        timestamp: String,

        /// 対象日付 (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// カテゴリプレフィックス
        #[arg(short = 'c', long)]
        category_prefix: Option<String>,

        /// JSON形式で出力
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// タイトルを変更
    Rename {
        /// 新しいタイトル
        #[arg(long)]
        name: String,

        /// 対象日付 (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// カテゴリプレフィックス
        #[arg(short = 'c', long)]
        category_prefix: Option<String>,

        /// JSON形式で出力
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add {
            text,
            text_file,
            timestamp,
            date,
            category_prefix,
            post_name,
            json,
        } => cmd_add(
            &text,
            &text_file,
            &timestamp,
            &date,
            &category_prefix,
            &post_name,
            json,
        ),
        Commands::Edit {
            text,
            text_file,
            timestamp,
            date,
            category_prefix,
            json,
        } => cmd_edit(&text, &text_file, &timestamp, &date, &category_prefix, json),
        Commands::Delete {
            timestamp,
            date,
            category_prefix,
            json,
        } => cmd_delete(&timestamp, &date, &category_prefix, json),
        Commands::Rename {
            name,
            date,
            category_prefix,
            json,
        } => cmd_rename(&name, &date, &category_prefix, json),
    }
}
