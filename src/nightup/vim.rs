use regex::Regex;
use std::env;

use std::process::{Command, ExitCode};

pub fn run(_dist_dir: &str, _download_dir: &str) -> ExitCode {
    let output = match Command::new("curl")
        .args([
            "-sSL",
            "-A",
            "Mozilla/5.0",
            "https://api.github.com/repos/vim/vim-win32-installer/releases/latest",
        ])
        .output()
    {
        Ok(out) => out,
        Err(_) => {
            eprintln!("failed to 'curl'");
            return ExitCode::FAILURE;
        }
    };
    println!("Download (json) is done");

    let contents = match String::from_utf8(output.stdout) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("failed to 'String::from_utf8'");
            return ExitCode::FAILURE;
        }
    };

    // ダウンロードURLを取得する
    let re_url = Regex::new(r#""browser_download_url":\s*"(https://[^"]+_x64_signed\.exe)""#)
        .expect("failed to compile vim.re_url (regex)");

    // 抽出できた URL を格納する変数
    let mut download_url = String::new();

    if let Some(caps) = re_url.captures(&contents)
        && let Some(mat) = caps.get(1)
    {
        download_url = mat.as_str().to_string();
    }

    if download_url.is_empty() {
        eprintln!("failed to find ZIP URL for gvim-x64-signed");
        return ExitCode::FAILURE;
    }

    println!("Download URL: {}", download_url);

    let download_dir = {
        let mut dir = match env::home_dir() {
            Some(path) => path,
            None => {
                eprintln!("Impossible to get your home dir!");
                return ExitCode::FAILURE;
            }
        };
        dir.push("Downloads");
        dir
    };

    let output = match Command::new("curl")
        .current_dir(&download_dir)
        .args(["-fsSOL", "-A", "Mozilla/5.0", &download_url])
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

    let start_status = match Command::new("cmd")
        .current_dir(&download_dir)
        .args(["/C", "start", "explorer", "."])
        .status()
    {
        Ok(out) => out,
        Err(_) => {
            eprintln!("failed to 'start explorer .'");
            return ExitCode::FAILURE;
        }
    };

    if !start_status.success() {
        eprintln!("start failed with status: {:?}", output.status);
        return ExitCode::FAILURE;
    }
    println!("Opened EXPLORER.EXE");

    ExitCode::SUCCESS
}
