use regex::Regex;
use std::io::Write;
use std::process::{Command, ExitCode, Stdio};

const HELP_MSG: &str = "
Usage:
    do.exe shitaraba [OPTION] GENRE ID NUMBER
OPTION:
    -h, --help                 ヘルプメッセージを表示";

fn convert_cp(code_point: &str) -> Result<String, String> {
    let cp_u32 = code_point
        .parse::<u32>()
        .map_err(|_| "failed to 'parse::<u32>'".to_string())?;

    char::from_u32(cp_u32)
        .map(|c| c.to_string())
        .ok_or_else(|| "failed to 'char::from_u32'".to_string())
}

pub fn run(args: &[String]) -> ExitCode {
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        println!("{}", HELP_MSG);
        return ExitCode::SUCCESS;
    }
    if args.len() != 3 {
        eprintln!("{}", HELP_MSG);
        return ExitCode::FAILURE;
    }

    let (genre, id, number) = (&args[0], &args[1], &args[2]);

    let url = format!("https://jbbs.shitaraba.net/bbs/read.cgi/{genre}/{id}/{number}/l50");

    let output = Command::new("cmd")
        .args([
            "/C",
            &format!(r#"curl -sSL -A "Mozilla/5.0" {url} | busybox64u iconv -f EUC-JP -t UTF-8"#),
        ])
        .output()
        .expect("failed to 'curl and busybox64u iconv'");

    let re = Regex::new(r"<dt(?ms)\b.+?<b>(\w+?)</b>.+?：(.+?)</dt>.+?<dd>(.+?)</dd>")
        .expect("failed to compile shitaraba.run.re (regex)");
    let re_emoji =
        Regex::new(r"&#(\d+?);").expect("failed to compile shitaraba.run.re_emoji (regex)");
    let re_tag = Regex::new(r#"<[^>]*?>"#).expect("failed to compile shitaraba.run.re_tag (regex)");

    let contents = String::from_utf8(output.stdout).expect("failed to 'String::from_utf8'");

    let mut datum = vec![];
    for caps in re.captures_iter(&contents) {
        let name = caps.get(1).map_or("", |m| m.as_str());
        let date = caps.get(2).map_or("", |m| m.as_str());
        let post = caps.get(3).map_or("", |m| m.as_str());

        let name = re_tag.replace_all(name, "").trim().to_string();
        let date = date.trim_ascii_end().to_string();
        let post = post
            .trim_ascii()
            .replace("<br>          <br>", "\n")
            .replace("<br>", "\n");
        let post = re_tag.replace_all(&post, "");
        let final_post = re_emoji
            .replace_all(&post, |caps: &regex::Captures| match convert_cp(&caps[1]) {
                Ok(emoji) => emoji,
                Err(_) => caps[0].to_string(),
            })
            .into_owned();

        datum.push((name, date, final_post));
    }

    let mut child = Command::new("less")
        .args(["-R", "-i", "--silent"])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn 'less'");

    // `stdin` を `take()` で完全に外に切り出す
    let mut stdin = child.stdin.take().expect("failed to open stdin");

    for (name, date, post) in datum {
        if write!(
            stdin,
            "\x1b[36m{name}\x1b[0m: \x1b[32m{date}\x1b[0m\n{post}"
        )
        .is_err()
        {
            eprintln!("failed to write to less stdin");
            let _ = child.kill();
            let _ = child.wait();
            return ExitCode::FAILURE;
        }
    }

    // 書き込みが完了したため、明示的に `stdin` を閉じて less に通知する
    drop(stdin);

    match child.wait() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        _ => {
            eprintln!("less exited with an error");
            ExitCode::FAILURE
        }
    }
}
