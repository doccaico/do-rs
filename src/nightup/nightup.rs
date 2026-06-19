use ini::Ini;
use std::env;

use anyhow::{Context, Result, bail};

mod zig;
// mod odin;
// mod go;
// mod v;
// mod vim;

const HELP_MSG: &str = r"
Usage:
    do.exe nightup zig
    do.exe nightup odin
    do.exe nightup go
    do.exe nightup v
    do.exe nightup vim
OPTION:
    -h, --help                 ヘルプメッセージを表示

下記のような%USERPROFILE%\.nightupを必ず作成してください(PATHは変更可能)

[Windows]
zig=C:\Langs\zig
odin=C:\Langs\odin
v=C:\Langs\v
go=C:\Langs\go";

pub fn run(args: &[String]) -> Result<()> {
    if args.len() != 1 {
        bail!("{}", HELP_MSG);
    }
    if args[0] == "-h" || args[0] == "--help" {
        println!("{}", HELP_MSG);
        return Ok(());
    }

    let conf = {
        let mut ini_path = env::home_dir().context("Impossible to get your home dir!")?;
        ini_path.push(".nightup");
        Ini::load_from_file_noescape(&ini_path)
            .context(format!("failed to open: {}", ini_path.display()))?
    };

    let section = conf
        .section(Some("Windows"))
        .context("not found section: 'Windows'")?;

    let download_dir = env::var("TEMP").unwrap_or_else(|_| ".".to_string());

    let command = &args[0];

    match command.as_str() {
        cmd @ "zig" => {
            let dist_dir = section
                .get(cmd)
                .context(format!("not found key: '{}'", cmd))?;
            zig::run(dist_dir, &download_dir)
        }
        // "odin" => {
        //     let dist_dir = match section.get("odin") {
        //         Some(path) => path,
        //         None => {
        //             eprintln!(r#"nightup ini: not found path: "odin""#);
        //             return ExitCode::FAILURE;
        //         }
        //     };
        //     odin::run(dist_dir, &download_dir)
        // }
        // "v" => {
        //     let dist_dir = match section.get("v") {
        //         Some(path) => path,
        //         None => {
        //             eprintln!(r#"nightup ini: not found path: "v""#);
        //             return ExitCode::FAILURE;
        //         }
        //     };
        //     v::run(dist_dir, &download_dir)
        // }
        // "go" => {
        //     let dist_dir = match section.get("go") {
        //         Some(path) => path,
        //         None => {
        //             eprintln!(r#"nightup ini: not found path: "go""#);
        //             return ExitCode::FAILURE;
        //         }
        //     };
        //     go::run(dist_dir, &download_dir)
        // }
        // "vim" => vim::run("", &download_dir),
        _ => {
            eprintln!("nightup: unknown command '{}'", command);
            bail!(HELP_MSG);
        }
    }
}
