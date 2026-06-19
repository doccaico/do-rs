use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use regex::Regex;

pub fn run(dist_dir: &str, download_dir: &str) -> Result<()> {
    let json_url = "https://f001.backblazeb2.com/file/odin-binaries/nightly.json";

    // curlを実行して nightly.json を取得
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
    println!("Download (nightly.json) is done");

    let contents = String::from_utf8_lossy(&output.stdout);

    // ダウンロードURLを取得する
    let re_date = Regex::new(r#""([\d]{4}-[\d]{2}-[\d]{2})T"#)
        .context("failed to compile odin.re_date (regex)")?;

    let nightly_date = re_date
        .captures(&contents)
        .and_then(|caps| caps.get(1))
        .map(|mat| mat.as_str().to_string())
        .context("failed to find date for odin-windows-amd64 nightly")?;

    let zip_name = format!("odin-windows-amd64-nightly%2B{}.zip", nightly_date);
    let download_url = format!(
        "https://f001.backblazeb2.com/file/odin-binaries/nightly/{}",
        zip_name
    );
    println!("Download URL: {}", download_url);

    // ワークディレクトリの準備
    let work_dir_name = "odin-latest-upgrade-working";
    let work_dir_path = Path::new(download_dir).join(work_dir_name);

    // 既存の古いワークフォルダがあれば中身ごと再帰削除
    if work_dir_path.exists() {
        fs::remove_dir_all(&work_dir_path)
            .context(format!("failed to remove '{}'", work_dir_path.display()))?;
        println!("Removed: '{}'", work_dir_path.display());
    }

    // 新しくワークフォルダを作成
    fs::create_dir_all(&work_dir_path)
        .context(format!("failed to create {}", work_dir_path.display()))?;
    println!("Created: '{}'", work_dir_path.display());

    let local_zip_name = "odin-latest.zip";
    let local_zip_path = Path::new(&work_dir_path).join(local_zip_name);

    // curlで最新のZIPファイルをダウンロード
    {
        let status = Command::new("curl")
            .current_dir(&work_dir_path)
            .args([
                "-fsSL",
                "-A",
                "Mozilla/5.0",
                &download_url,
                "-o",
                local_zip_name,
            ])
            .status()
            .context("failed to run 'curl'")?;

        if !status.success() {
            bail!("'curl' failed with error: {}", status.code().unwrap());
        }
        println!("Download (ZIP) is done: {}", local_zip_name);
    }

    // tarコマンドでアーカイブを展開
    {
        let status = Command::new("tar")
            .current_dir(&work_dir_path)
            .args(["-xf", local_zip_name, "--strip-components=1"])
            .status()
            .context("failed to run 'tar'")?;

        if !status.success() {
            bail!("'tar' failed with error: {}", status.code().unwrap());
        }
        println!("Extraction is done");
    }

    // ダウンロードしたZIPを削除
    if local_zip_path.exists() {
        fs::remove_file(&local_zip_path)
            .context(format!("failed to remove '{}'", local_zip_path.display()))?;
        println!("Removed: '{}'", local_zip_path.display());
    }

    // アップデート処理
    let target_path = Path::new(dist_dir);

    // 既存のインストールされているフォルダを中身ごと再帰削除
    if target_path.exists() {
        fs::remove_dir_all(target_path).context("failed to remove old dist directory")?;
        println!("Removed: '{}'", target_path.display());
    }

    // ワークフォルダを配信先にリネーム (移動)
    fs::rename(&work_dir_path, target_path)
        .context("failed to move extracted directory to dist")?;

    println!(
        "Moved: '{}' to '{}'",
        work_dir_path.display(),
        target_path.display()
    );

    println!("💓The update was successful💓");

    Ok(())
}
