use anyhow::{Context, Result, bail};
use std::io::Write;
use std::path::Path;
use std::process;

// use crate::utils::{eprint_and_exit, print_and_exit};

const HELP_MSG: &str = "
Usage:
    do.exe diary_search [OPTION] 検索キーワード
OPTION:
    -h, --help                 ヘルプメッセージを表示
REQUIRED:
    環境変数(DIARY_DIR)に日記が入っているディレクトリを設定すること";

pub fn run(args: &[String]) -> Result<()> {
    if args.len() != 1 {
        eprintln!("{}", HELP_MSG);
        bail!("");
    }

    if args[0] == "-h" || args[0] == "--help" {
        println!("{}", HELP_MSG);
        return Ok(());
    }

    let diary_dir_raw =
        std::env::var("DIARY_DIR").context("not found 'DIARY_DIR' in env variable")?;

    let diary_dir_path = Path::new(&diary_dir_raw);
    if !diary_dir_path.exists() {
        bail!("'{}' does not exist", diary_dir_path.display());
    }
    if !diary_dir_path.is_dir() {
        bail!("'{}' is not a directory", diary_dir_path.display());
    }

    let keyword = &args[0];

    let output = process::Command::new("rg")
        .args([
            "--color",
            "always",
            "--heading",
            "--line-number",
            "--ignore-case",
            "--sort=path",
            keyword,
        ])
        .arg(diary_dir_path)
        .output()
        .context("failed to run 'rg'")?;

    if output.stdout.is_empty() {
        println!("no matches found for '{}'", keyword);
        return Ok(());
    }

    let mut child = process::Command::new("less")
        .args(["-R", "-i", "--silent"])
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .spawn()
        .context("failed to spawn 'less'")?;

    // 'stdin' を 'take()' で完全に外に切り出す
    let mut stdin = child.stdin.take().context("failed to open stdin")?;

    // less への書き込み (途中で q で閉じられた場合の BrokenPipe エラーを許容する)
    if let Err(e) = stdin.write_all(&output.stdout)
        && e.kind() != std::io::ErrorKind::BrokenPipe
    {
        let _ = child.kill();
        bail!("error occurred while writing to less: {}", e);
    }

    // 書き込みが完了したため、明示的に 'stdin' を閉じて less に通知する
    drop(stdin);

    let _ = child.wait().context("'less' failed with")?;

    Ok(())
}
