use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

pub fn run(dist_dir: &str, download_dir: &str) -> Result<()> {
    let download_url =
        "https://github.com/nim-lang/nightlies/releases/download/latest-devel/windows_x64.zip";
    println!("Download URL: {}", download_url);

    // ワークディレクトリの準備
    let work_dir_name = "nim-latest-upgrade-working";
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

    let local_zip_name = "nim-latest.zip";
    let local_zip_path = Path::new(&work_dir_path).join(local_zip_name);

    // curlで最新のZIPファイルをダウンロード
    {
        let status = Command::new("curl")
            .current_dir(&work_dir_path)
            .args([
                "-fsSL",
                "-A",
                "Mozilla/5.0",
                download_url,
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
