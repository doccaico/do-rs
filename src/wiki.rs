use std::io::ErrorKind;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use colored::Colorize;
use regex::Regex;

const HELP_MSG: &str = "
Usage:
    do.exe wiki [OPTION] COUNT
OPTION:
    -h, --help                 ヘルプメッセージを表示";

fn decode_unicode_escape(re: &Regex, title: &str) -> Result<String> {
    let mut err = None;

    let result = re
        .replace_all(title, |caps: &regex::Captures| {
            let raw_title = caps.get(1).map_or("", |m| m.as_str());

            // クロージャの内部では ? を使わず、Result型として処理する
            let c = u32::from_str_radix(raw_title, 16)
                .context("failed to 'u32::from_str_radix'")
                .and_then(|code| char::from_u32(code).context("failed to 'char::from_u32'"));

            match c {
                Ok(ch) => ch.to_string(),
                Err(e) => {
                    err = Some(e); // 外側の変数にエラーを退避
                    String::new()
                }
            }
        })
        .to_string();

    if let Some(e) = err {
        return Err(e);
    }

    Ok(result)
}

pub fn run(args: &[String]) -> Result<()> {
    if args.len() != 1 {
        bail!("{}", HELP_MSG);
    }
    if args[0] == "-h" || args[0] == "--help" {
        println!("{}", HELP_MSG);
        return Ok(());
    }

    let count = &args[0];

    let url = format!(
        "https://ja.wikipedia.org/w/api.php\
?format=json\
&action=query\
&list=random\
&rnnamespace=0\
&rnfilterredir=nonredirects\
&rnlimit={}\
",
        count
    );

    let output = Command::new("curl")
        .args(["-sSL", "-A", "Mozilla/5.0", { &url }])
        .output()
        .context("failed to 'curl'")?;

    let raw_json = String::from_utf8_lossy(&output.stdout);

    // let re = Regex::new(r#""id":\s*(\d+).*?"title":\s*"([^"]+?)""#)
    //     .expect("failed to compile wiki.run.re (regex)");
    let re = Regex::new(r#""id":(\d+?),"ns":0,"title":"(.+?)""#)
        .expect("failed to compile wiki.run.re (regex)");

    let re_hex = Regex::new(r"\\u([0-9a-fA-F]{4})")
        .context("failed to compile wiki.decode_unicode_escape.re_hex (regex)")?;

    let mut idx = 1;
    let mut output = String::new();
    let mut caps_iter = re.captures_iter(&raw_json).peekable();
    if caps_iter.peek().is_some() {
        for caps in caps_iter {
            let id = caps.get(1).map_or("", |m| m.as_str());
            let title = caps.get(2).map_or("", |m| m.as_str());

            let title = decode_unicode_escape(&re_hex, title)?;

            output.push_str(&format!(
                "{}:{}:{}\n",
                idx.to_string().magenta(),
                title.cyan(),
                format!("https://ja.wikipedia.org/?curid={}", id).green(),
            ));
            idx += 1;
        }
    } else {
        bail!(format!("failed to parse the json\nRaw: {}", raw_json));
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
    if let Err(e) = write!(stdin, "{}", output)
        && e.kind() != ErrorKind::BrokenPipe
    {
        let _ = child.kill();
        bail!("error occurred while writing to less: {}", e);
    }

    // 書き込みが完了したため、明示的に `stdin` を閉じて less に通知する
    drop(stdin);

    let _ = child.wait().context("'less' failed")?;

    Ok(())
}
