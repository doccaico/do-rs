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
            "https://ziglang.org/download/index.json",
        ])
        .output()
    {
        Ok(out) => out,
        Err(_) => {
            eprintln!("failed to 'curl'");
            return ExitCode::FAILURE;
        }
    };
    println!("Download (index.json) is done");

    let contents = match String::from_utf8(output.stdout) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("failed to 'String::from_utf8'");
            return ExitCode::FAILURE;
        }
    };

    // ダウンロードURLを取得する
    let re_url =
        Regex::new(r#"(?ms)"master":\s*\{.*?"x86_64-windows":\s*\{.*?"tarball":\s*"([^"]+)""#)
            .expect("failed to compile zig.re_url (regex)");

    // 抽出できた URL を格納する変数
    let mut download_url = String::new();

    if let Some(caps) = re_url.captures(&contents)
        && let Some(mat) = caps.get(1)
    {
        download_url = mat.as_str().to_string();
    }

    if download_url.is_empty() {
        eprintln!("failed to find ZIP URL for x86_64-windows master");
        return ExitCode::FAILURE;
    }
    println!("Download URL: {}", download_url);

    let work_dir_name = "zig-master-upgrade-working";
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

    let local_zip = "zig-master-latest.zip";

    // ワークスペース内にダウンロードする
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

    println!("Download (ZIP) is done: {}", local_zip);

    // 解凍する
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

    // 後始末：展開が終わったのでZIPファイルを削除
    let zip_path = work_dir_path.join(local_zip);
    if zip_path.exists() {
        if let Err(e) = fs::remove_file(&zip_path) {
            eprintln!("failed to remove {}: {}", zip_path.display(), e);
            return ExitCode::FAILURE;
        }
        println!(r#"Removed: "{}""#, zip_path.display());
    }

    //  配置（アップデートの適用）
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
