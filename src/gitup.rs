use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process;

const HELP_MSG: &str = "
Usage:
    do.exe gitup [OPTION] DIR MESSAGE
    do.exe gitup [OPTION]     MESSAGE
OPTION:
    -h, --help                 ヘルプメッセージを表示";

pub fn run(args: &[String]) -> Result<()> {
    if args.is_empty() || args.len() > 2 {
        bail!("{}", HELP_MSG);
    }
    if args[0] == "-h" || args[0] == "--help" {
        println!("{}", HELP_MSG);
        return Ok(());
    }

    let (dir_path, commit_msg): (&str, &str) = {
        if args.len() == 2 {
            let path = Path::new(&args[0]);
            if !path.exists() {
                bail!("'{}' does not exist", path.display());
            }
            if !path.is_dir() {
                bail!("'{}' is not a directory", path.display());
            }
            (&args[0], &args[1])
        } else {
            (".", &args[0])
        }
    };

    {
        let output = process::Command::new("git")
            .current_dir(dir_path)
            .args(["status", "--porcelain"])
            .output()
            .context("failed to run 'git'")?;

        if !output.status.success() {
            bail!("not a git repository (or any of the parent directories): .git");
        }

        if output.stdout.is_empty() {
            println!("There is no need to update");
            return Ok(());
        }

        println!("==> Detected changes:");
        println!("{}", indent(&output.stdout, 4));
    }

    {
        println!("==> Running: git add .");

        let output = process::Command::new("git")
            .current_dir(dir_path)
            .args(["add", "."])
            .output()
            .context("failed to run 'git'")?;

        if !output.status.success() {
            bail!("'git add .' failed");
        }

        let mut all_output = output.stdout;
        all_output.extend(output.stderr);

        if !all_output.is_empty() {
            println!("{}", indent(&all_output, 4));
        }
    }

    {
        println!("==> Running: git commit -m \"{}\"", commit_msg);

        let output = process::Command::new("git")
            .current_dir(dir_path)
            .args(["commit", "-m", commit_msg])
            .output()
            .context("failed to run 'git'")?;

        if !output.status.success() {
            bail!(format!("'git commit -m \"{}\"' failed", commit_msg));
        }

        let mut all_output = output.stdout;
        all_output.extend(output.stderr);

        println!("{}", indent(&all_output, 4));
    }

    {
        println!("==> Running: git push");

        let output = process::Command::new("git")
            .current_dir(dir_path)
            .arg("push")
            .output()
            .context("failed to run 'git'")?;

        if !output.status.success() {
            bail!("'git push' failed");
        }

        let mut all_output = output.stdout;
        all_output.extend(output.stderr);

        println!("{}", indent(&all_output, 4));
    }

    Ok(())
}

fn indent(source: &[u8], size: usize) -> String {
    let indent_prefix = " ".repeat(size);
    let source_str = String::from_utf8_lossy(source);
    source_str
        .lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("{}{}", indent_prefix, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
