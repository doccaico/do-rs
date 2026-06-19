use anyhow::{Result, bail};
use std::env;
use std::process::ExitCode;

mod delete_duplicate_path;
mod diary_search;
mod gitup;
mod shitaraba;
mod verse;
mod wiki;

#[path = "nightup/nightup.rs"]
mod nightup;

const HELP_MSG: &str = "
Usage:
    do.exe [OPTION] COMMAND [ARGS...]
OPTION:
    -h, --help                 ヘルプメッセージを表示
COMMAND:
    diary-search                環境変数(DIARY_DIR)にある日記を検索
    gitup                       GithubにPush
    shitaraba                   Shitarabaを閲覧
    delete-duplicate-path       環境変数PATHの重複を解消して表示
    verse                       聖書(新共同訳)を表示
    wiki                        ランダムWIKIのリストを表示
    nightup                     ソフトウェアアップデーター";

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut args = env::args();
    args.next();
    let args = args.collect::<Vec<_>>();

    if args.is_empty() {
        bail!("{}", HELP_MSG);
    }

    if args[0] == "-h" || args[0] == "--help" {
        println!("{}", HELP_MSG);
        return Ok(());
    }

    let command = &args[0];
    let sub_args = &args[1..];

    match command.as_str() {
        "diary-search" => diary_search::run(sub_args),
        "delete-duplicate-path" => delete_duplicate_path::run(),
        "gitup" => gitup::run(sub_args),
        "shitaraba" => shitaraba::run(sub_args),
        "verse" => verse::run(sub_args),
        "wiki" => wiki::run(sub_args),
        "nightup" => nightup::run(sub_args),
        _ => {
            eprintln!("{}", HELP_MSG);
            eprintln!();
            bail!("unknown command '{}'", command);
        }
    }
}
