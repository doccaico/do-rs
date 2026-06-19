// use std::fs;
// use std::io::Write;
// use std::path::PathBuf;
use std::process;
// use std::process::ExitCode;
// use std::process::{Command, ExitCode, Stdio};

pub fn print_and_exit(msg: &str) -> ! {
    println!("{}", msg);
    process::exit(0);
}

pub fn eprint_and_exit(msg: &str) -> ! {
    eprintln!("{}", msg);
    process::exit(1);
}

// pub fn view_help_msg(msg: &str) -> ExitCode {
//     let mut child = match Command::new("less")
//         .args(["-R", "-i", "--silent"])
//         .stdin(Stdio::piped())
//         .stdout(Stdio::inherit())
//         .stderr(Stdio::inherit())
//         .spawn()
//     {
//         Ok(c) => c,
//         Err(_) => {
//             eprintln!("failed to spawn 'less'");
//             return ExitCode::FAILURE;
//         }
//     };
//
//     if let Some(mut stdin) = child.stdin.take()
//         && write!(stdin, "{}", msg).is_err()
//     {
//         eprintln!("failed to write to less stdin");
//         return ExitCode::FAILURE;
//     }
//
//     match child.wait() {
//         Ok(status) if status.success() => ExitCode::SUCCESS,
//         _ => {
//             eprintln!("less exited with an error");
//             ExitCode::FAILURE
//         }
//     }
// }
//
// // スコープを抜ける時に自動でディレクトリを削除する構造体 (deferの代わり)
// pub struct CleanupDir {
//     pub path: PathBuf,
//     pub active: bool, // 最後にrenameで移動した場合は削除をスキップするためのフラグ
// }
//
// impl Drop for CleanupDir {
//     fn drop(&mut self) {
//         if self.active && self.path.exists() {
//             fs::remove_dir_all(&self.path).ok();
//             println!(r#"Automatically cleaned up: "{}""#, self.path.display());
//         }
//     }
// }
