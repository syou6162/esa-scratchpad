#[allow(dead_code)]
mod client;
#[allow(dead_code)]
mod config;
#[allow(dead_code)]
mod entry;
#[allow(dead_code)]
mod error;
#[allow(dead_code)]
mod validator;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "esa-scratchpad", about = "esa.io ラクガキ帳 CLI ツール")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// エントリを投稿
    Write,
    /// エントリを修正
    Update,
    /// エントリを削除
    Delete,
    /// タイトルを変更
    Title,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Write => {
            todo!("write subcommand is not yet implemented")
        }
        Commands::Update => {
            todo!("update subcommand is not yet implemented")
        }
        Commands::Delete => {
            todo!("delete subcommand is not yet implemented")
        }
        Commands::Title => {
            todo!("title subcommand is not yet implemented")
        }
    }
}
