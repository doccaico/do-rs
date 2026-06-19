use regex::Regex;
use std::io::Write;
use std::process::{Command, ExitCode, Stdio};
use std::str::from_utf8;

const HELP_MSG: &str = "
Usage:
    do.exe wiki COUNT";

fn decode_unicode_escape(s: &str) -> String {
    let re = Regex::new(r"\\u([0-9a-fA-F]{4})")
        .expect("failed to compile wiki.decode_unicode_escape.re (regex)");
    re.replace_all(s, |caps: &regex::Captures| {
        let hex_str = &caps[1];
        if let Ok(code_point) = u32::from_str_radix(hex_str, 16)
            && let Some(c) = std::char::from_u32(code_point)
        {
            return c.to_string();
        }
        caps[0].to_string()
    })
    .into_owned()
}

pub fn run(args: &[String]) -> ExitCode {
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        println!("{}", HELP_MSG);
        return ExitCode::SUCCESS;
    }
    if args.len() != 1 {
        eprintln!("{}", HELP_MSG);
        return ExitCode::FAILURE;
    }

    let url = format!(
        "https://ja.wikipedia.org/w/api.php\
?format=json\
&action=query\
&list=random\
&rnnamespace=0\
&rnfilterredir=nonredirects\
&rnlimit={}\
",
        args[0]
    );

    let output = match Command::new("curl")
        .args(["-sSL", "-A", "Mozilla/5.0", { &url }])
        .output()
    {
        Ok(out) => out,
        Err(_) => {
            eprintln!("failed to 'curl'");
            return ExitCode::FAILURE;
        }
    };

    let response_text = match from_utf8(&output.stdout) {
        Ok(output) => output,
        Err(_) => {
            eprintln!("failed to 'from_utf8'");
            return ExitCode::FAILURE;
        }
    };

    let re = Regex::new(r#""id":\s*(\d+).*?"title":\s*"([^"]+?)""#)
        .expect("failed to compile wiki.run.re (regex)");

    let mut articles = vec![];
    for (idx, caps) in re.captures_iter(response_text).enumerate() {
        let (_, [id, title]) = caps.extract();
        let clean_title = decode_unicode_escape(title);
        articles.push(format!(
            "\x1b[35m{}\x1b[0m:\x1b[36m{}\x1b[0m: \x1b[32mhttps://ja.wikipedia.org/?curid={}\x1b[0m",
            idx+1, clean_title, id
        ));
    }

    if articles.is_empty() {
        eprintln!("No random articles found. Raw response might be unexpected.");
        eprintln!("Raw: {}", response_text);
        return ExitCode::FAILURE;
    }

    let mut child = match Command::new("less")
        .args(["-R", "--silent"])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            eprintln!("failed to spawn 'less'");
            return ExitCode::FAILURE;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        for article in articles {
            if writeln!(stdin, "{}", article).is_err() {
                eprintln!("failed to write to less stdin");
                return ExitCode::FAILURE;
            }
        }
    }

    match child.wait() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        _ => {
            eprintln!("less exited with an error");
            ExitCode::FAILURE
        }
    }
}
