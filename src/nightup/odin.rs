use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

use crate::utils;

pub fn run(dist_dir: &str, download_dir: &str) -> ExitCode {
    let output = match Command::new("curl")
        .args([
            "-sSL",
            "-A",
            "Mozilla/5.0",
            "https://f001.backblazeb2.com/file/odin-binaries/nightly.json",
        ])
        .output()
    {
        Ok(out) => out,
        Err(_) => {
            eprintln!("failed to 'curl'");
            return ExitCode::FAILURE;
        }
    };
    println!("Download (nightly.json) is done");

    let contents = match String::from_utf8(output.stdout) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("failed to 'String::from_utf8'");
            return ExitCode::FAILURE;
        }
    };

    // ダウンロードURLを取得する
    let re_date = Regex::new(r#""([\d]{4}-[\d]{2}-[\d]{2})T"#)
        .expect("failed to compile odin.re_date (regex)");

    // 抽出できた URL を格納する変数
    let mut nightly_date = String::new();

    if let Some(caps) = re_date.captures(&contents)
        && let Some(mat) = caps.get(1)
    {
        nightly_date = mat.as_str().to_string();
    }

    if nightly_date.is_empty() {
        eprintln!("failed to find ZIP URL for odin-windows-amd64 nightly");
        return ExitCode::FAILURE;
    }

    let zip_name = format!("odin-windows-amd64-nightly%2B{}.zip", nightly_date);
    let download_url = format!(
        "https://f001.backblazeb2.com/file/odin-binaries/nightly/{}",
        zip_name
    );
    println!("Download URL: {}", download_url);

    let work_dir_name = "odin-nightly-upgrade-working";
    let work_dir_path = Path::new(download_dir).join(work_dir_name);

    if work_dir_path.exists() {
        if let Err(e) = fs::remove_dir_all(&work_dir_path) {
            eprintln!("failed to remove {}: {}", work_dir_path.display(), e);
            return ExitCode::FAILURE;
        }
        println!(r#"Removed: "{}""#, work_dir_path.display());
    }

    if let Err(e) = fs::create_dir_all(&work_dir_path) {
        eprintln!("failed to create {}: {}", work_dir_path.display(), e);
        return ExitCode::FAILURE;
    }
    println!(r#"Created: "{}""#, work_dir_path.display());

    // これ以降、どこで `return` しても `work_dir_path` は自動削除される
    let mut cleanup = utils::CleanupDir {
        path: work_dir_path.clone(),
        active: true,
    };

    let local_zip = "odin-nightly-latest.zip";

    let output = match Command::new("curl")
        .current_dir(&work_dir_path)
        .args(["-fsSL", "-A", "Mozilla/5.0", &download_url, "-o", local_zip])
        .output()
    {
        Ok(out) => out,
        Err(_) => {
            eprintln!("failed to 'curl ZIP file'");
            return ExitCode::FAILURE;
        }
    };

    if !output.status.success() {
        eprintln!("curl failed with status: {:?}", output.status);
        return ExitCode::FAILURE;
    }
    println!("Download (ZIP) is done");

    let output = match Command::new("tar")
        .current_dir(&work_dir_path)
        .args(["-xf", local_zip, "--strip-components=1"])
        .output()
    {
        Ok(out) => out,
        Err(_) => {
            eprintln!("failed to run OS native 'tar'. Make sure Windows 10/11 is updated.");
            return ExitCode::FAILURE;
        }
    };

    if !output.status.success() {
        eprintln!("tar failed with status: {:?}", output.status);
        return ExitCode::FAILURE;
    }
    println!("Extraction is done");

    // 後始末と配置
    let zip_path = work_dir_path.join(local_zip);
    if zip_path.exists() {
        if let Err(e) = fs::remove_file(&zip_path) {
            eprintln!("failed to remove {}: {}", zip_path.display(), e);
            return ExitCode::FAILURE;
        }
        println!(r#"Removed: "{}""#, zip_path.display());
    }

    // 配置（アップデートの適用）
    let target_path = Path::new(dist_dir);
    // 既存の古いインストールディレクトリがあれば削除
    if target_path.exists() {
        if let Err(e) = fs::remove_dir_all(target_path) {
            eprintln!("failed to remove old dist directory: {}", e);
            return ExitCode::FAILURE;
        }
        println!(r#"Removed: "{}""#, target_path.display());
    }

    // 中身がファイル群だけになったワークスペースそのものを
    // そのまま dist_dir のパスへ移動リネームする
    if let Err(e) = fs::rename(&work_dir_path, target_path) {
        eprintln!("failed to move extracted directory to dist: {}", e);
        return ExitCode::FAILURE;
    }

    cleanup.active = false; // 移動に成功したため、Drop時の削除を免除する

    println!(
        r#"Moved: "{}" to "{}""#,
        work_dir_path.display(),
        target_path.display()
    );

    println!(r#"Updated: "{}""#, dist_dir);

    ExitCode::SUCCESS
}
