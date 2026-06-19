use std::env;
use std::process::Command;

use anyhow::{Context, Result, bail};
use regex::Regex;

pub fn run() -> Result<()> {
    let json_url = "https://api.github.com/repos/vim/vim-win32-installer/releases/latest";

    // curlを実行してモードjsonのテキストデータを取得
    let output = Command::new("curl")
        .args(["-sSL", "-A", "Mozilla/5.0", json_url])
        .output()
        .context("failed to run 'curl'")?;

    if !output.status.success() {
        bail!(
            "'curl' failed with error: {}",
            output.status.code().unwrap()
        );
    }
    println!("Download (json) is done");

    let contents = String::from_utf8_lossy(&output.stdout);

    // ダウンロードURLを取得する
    let re_url = Regex::new(
        r#""name":"gvim_.+?_x64_signed\.exe".+?"browser_download_url":"(https://github.com/vim/vim-win32-installer/releases/download/.+?_x64_signed\.exe)""#,
    )
    .context("failed to compile vim.re_url (regex)")?;

    // 抽出できた URL を格納する変数
    let download_url = re_url
        .captures(&contents)
        .and_then(|caps| caps.get(1))
        .map(|mat| mat.as_str().to_string())
        .context("failed to find ZIP URL for gvim-x64-signed")?;
    println!("Download URL: {}", download_url);

    let user_download_dir = {
        let mut dir = env::home_dir().context("Impossible to get your home dir!")?;
        dir.push("Downloads");
        dir
    };

    // curlで最新のZIPファイルをダウンロード
    {
        let status = Command::new("curl")
            .current_dir(&user_download_dir)
            .args(["-fsSLO", "-A", "Mozilla/5.0", &download_url])
            .status()
            .context("failed to run 'curl'")?;

        if !status.success() {
            bail!("'curl' failed with error: {}", status.code().unwrap());
        }
        println!("💓The download was successful💓");
    }

    {
        let status = Command::new("cmd")
            .current_dir(&user_download_dir)
            .args(["/C", "start", "explorer", "."])
            .status()
            .context("failed to run 'cmd'")?;

        if !status.success() {
            bail!("'start' failed with status: {}", status.code().unwrap());
        }
        println!("Opened EXPLORER.EXE");
    }

    Ok(())
}
