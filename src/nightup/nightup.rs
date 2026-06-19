use ini::Ini;
use std::env;
use std::process::ExitCode;

mod go;
mod odin;
mod v;
mod vim;
mod zig;

const HELP_MSG: &str = "
Usage:
    do.exe nightup go
    do.exe nightup odin
    do.exe nightup v
    do.exe nightup zig";

pub fn run(args: &[String]) -> ExitCode {
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        println!("{}", HELP_MSG);
        return ExitCode::SUCCESS;
    }
    if args.len() != 1 {
        eprintln!("{}", HELP_MSG);
        return ExitCode::FAILURE;
    }

    let conf = {
        let mut home_dir = match env::home_dir() {
            Some(path) => path,
            None => {
                eprintln!("Impossible to get your home dir!");
                return ExitCode::FAILURE;
            }
        };
        home_dir.push(".nightup");
        match Ini::load_from_file_noescape(&home_dir) {
            Ok(ini) => ini,
            Err(_) => {
                eprintln!("failed to open: {}", home_dir.display());
                return ExitCode::FAILURE;
            }
        }
    };

    let section = match conf.section(Some("Windows")) {
        Some(section) => section,
        None => {
            eprintln!(r#"nightup ini: not found section: "Windows""#);
            return ExitCode::FAILURE;
        }
    };

    let download_dir = env::var("TEMP").unwrap_or_else(|_| ".".to_string());

    match args[0].as_str() {
        "zig" => {
            let dist_dir = match section.get("zig") {
                Some(path) => path,
                None => {
                    eprintln!(r#"nightup ini: not found path: "zig""#);
                    return ExitCode::FAILURE;
                }
            };
            zig::run(dist_dir, &download_dir)
        }
        "odin" => {
            let dist_dir = match section.get("odin") {
                Some(path) => path,
                None => {
                    eprintln!(r#"nightup ini: not found path: "odin""#);
                    return ExitCode::FAILURE;
                }
            };
            odin::run(dist_dir, &download_dir)
        }
        "v" => {
            let dist_dir = match section.get("v") {
                Some(path) => path,
                None => {
                    eprintln!(r#"nightup ini: not found path: "v""#);
                    return ExitCode::FAILURE;
                }
            };
            v::run(dist_dir, &download_dir)
        }
        "go" => {
            let dist_dir = match section.get("go") {
                Some(path) => path,
                None => {
                    eprintln!(r#"nightup ini: not found path: "go""#);
                    return ExitCode::FAILURE;
                }
            };
            go::run(dist_dir, &download_dir)
        }
        "vim" => vim::run("", &download_dir),
        _ => {
            eprintln!("nightup: unknown command '{}'\n{}", args[0], HELP_MSG);
            ExitCode::FAILURE
        }
    }
}
