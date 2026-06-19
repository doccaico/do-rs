use std::io::ErrorKind;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use colored::Colorize;
use regex::Regex;

const HELP_MSG: &str = "
Usage:
    do.exe shitaraba [OPTION] GENRE ID NUMBER
OPTION:
    -h, --help                 ヘルプメッセージを表示";

fn convert_cp(code_point: &str) -> Result<String> {
    let cp_u32 = code_point
        .parse::<u32>()
        .context("failed to 'parse::<u32>'".to_string())?;

    char::from_u32(cp_u32)
        .map(|c| c.to_string())
        .context("failed to 'char::from_u32'".to_string())
}

pub fn run(args: &[String]) -> Result<()> {
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        println!("{}", HELP_MSG);
        return Ok(());
    }
    if args.len() != 3 {
        bail!("{}", HELP_MSG);
    }

    let (genre, id, number) = (&args[0], &args[1], &args[2]);

    let url = format!("https://jbbs.shitaraba.net/bbs/read.cgi/{genre}/{id}/{number}/l50");

    let output = Command::new("cmd")
        .args([
            "/C",
            &format!(r#"curl -sSL -A "Mozilla/5.0" {url} | busybox64u iconv -f EUC-JP -t UTF-8"#),
        ])
        .output()
        .context("failed to 'curl or busybox64u'")?;

    let re = Regex::new(r"(?ms)<dt.+?<b>(\w+?)</b>.+?：(.+?)</dt>.+?<dd>(.+?)<br>          <br>")
        .context("failed to compile shitaraba.run.re (regex)")?;
    let re_emoji =
        Regex::new(r"&#(\d+?);").context("failed to compile shitaraba.run.re_emoji (regex)")?;

    let contents = String::from_utf8_lossy(&output.stdout);

    let mut datum = vec![];
    let mut caps_iter = re.captures_iter(&contents).peekable();
    if caps_iter.peek().is_some() {
        for caps in caps_iter {
            let name = caps.get(1).map_or("", |m| m.as_str());
            let date = caps.get(2).map_or("", |m| m.as_str());
            let post = caps.get(3).map_or("", |m| m.as_str());

            let name = name.trim().to_string();
            let date = date.trim_ascii_end().to_string();
            let post = post.trim_ascii_start().replace("<br>", "");
            let post = re_emoji
                .replace_all(&post, |caps: &regex::Captures| match convert_cp(&caps[1]) {
                    Ok(emoji) => emoji,
                    Err(_) => caps[0].to_string(),
                })
                .to_string();

            datum.push((name, date, post));
        }
    } else {
        bail!("failed to parse the html. Please check if the GENRE, ID, or NUMBER is correct");
    }

    let mut child = Command::new("less")
        .args(["-R", "-i", "--silent"])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to spawn 'less'")?;

    // `stdin` を `take()` で完全に外に切り出す
    let mut stdin = child.stdin.take().context("failed to open stdin")?;

    // less への書き込み (途中で q で閉じられた場合の BrokenPipe エラーを許容する)
    for (name, date, post) in datum {
        if let Err(e) = write!(stdin, "{}:{}\n{}\n", name.cyan(), date.green(), post)
            && e.kind() != ErrorKind::BrokenPipe
        {
            let _ = child.kill();
            bail!("error occurred while writing to less: {}", e);
        }
    }

    // 書き込みが完了したため、明示的に `stdin` を閉じて less に通知する
    drop(stdin);

    let _ = child.wait().context("'less' failed")?;

    Ok(())
}
