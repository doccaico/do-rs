use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use colored::Colorize;
use regex::Regex;

const HELP_MSG: &str = "
Usage:
    do.exe verse [OPTION] BOOK CHAPTER
OPTION:
    -h, --help                 ヘルプメッセージを表示

Old-Testament:
    創世記:GEN(1:50)
    出エジプト記:EXO(1:40)
    レビ記:LEV(1:27)
    民数記:NUM(1:36)
    申命記:DEU(1:34)
    ヨシュア記:JOS(1:24)
    士師記:JDG(1:21)
    ルツ記:RUT(1:4)
    サムエル記(上):1SA(1:31)
    サムエル記(下):2SA(1:24)
    列王記(上):1KI(1:22)
    列王記(下):2KI(1:25)
    歴代誌(上):1CH(1:29)
    歴代誌(下):2CH(1:36)
    エズラ記:EZR(1:10)
    ネヘミヤ記:NEH(1:13)
    エステル記:EST(1:10)
    ヨブ記:JOB(1:42)
    詩編:PSA(1:150)
    箴言:PRO(1:31)
    コヘレトの言葉:ECC(1:12)
    雅歌:SNG(1:8)
    イザヤ書:ISA(1:66)
    エレミヤ書:JER(1:52)
    哀歌:LAM(1:5)
    エゼキエル書:EZK(1:48)
    ダニエル書:DAN(1:12)
    ホセア書:HOS(1:14)
    ヨエル書:JOL(1:4)
    アモス書:AMO(1:9)
    オバデヤ書:OBA(1)
    ヨナ書:JON(1:4)
    ミカ書:MIC(1:7)
    ナホム書:NAM(1:3)
    ハバクク書:HAB(1:3)
    ゼファニヤ書:ZEP(1:3)
    ハガイ書:HAG(1:2)
    ゼカリヤ書:ZEC(1:14)
    マラキ書:MAL(1:3)
    ユディト記:JDT(1:16)
    知恵の書:WIS(1:19)
    トビト記:TOB(1:14)
    シラ:SIR(1:51)
    バルク書:BAR(1:5)
    エレミヤの手紙:LJE(1)
    マカバイ記(一):1MA(1:16)
    マカバイ記(二 書簡):2MA(1:15)
    エステル記(ギリシア語):ESG(1:10 + 10_1)
    ダニエル書補遺 スザンナ:SUS(1)
    ダニエル書補遺 ベルと竜:BEL(1)
    ダニエル書補遺 アザルヤの祈りと三人の若者の賛歌:S3Y(1)
    エズラ記(ギリシア語):1ES(1:9)
    エズラ記(ラテン語):2ES(1:16)
    マナセの祈り:MAN(1)

:New-Testament:
    マタイによる福音書:MAT(1:28)
    マルコによる福音書:MRK(1:16)
    ルカによる福音書:LUK(1:24)
    ヨハネによる福音書:JHN(1:21)
    使徒言行録:ACT(1:28)
    ローマの信徒への手紙:ROM(1:16)
    コリントの信徒への手紙(一):1CO(1:16)
    コリントの信徒への手紙(二):2CO(1:13)
    ガラテヤの信徒への手紙:GAL(1:6)
    エフェソの信徒への手紙:EPH(1:6)
    フィリピの信徒への手紙:PHP(1:4)
    コロサイの信徒への手紙:COL(1:4)
    テサロニケの信徒への手紙(一):1TH(1:5)
    テサロニケの信徒への手紙(二):2TH(1:3)
    テモテへの手紙(一):1TI(1:6)
    テモテへの手紙(二):2TI(1:4)
    テトスへの手紙:TIT(1:3)
    フィレモンへの手紙:PHM(1)
    ペトロの手紙(一):1PE(1:5)
    ペトロの手紙(二):2PE(1:3)
    ヨハネの手紙(一):1JN(1:5)
    ヨハネの手紙(二):2JN(1)
    ヨハネの手紙(三):3JN(1)
    ヘブライ人への手紙:HEB(1:13)
    ヤコブの手紙:JAS(1:5)
    ユダの手紙:JUD(1)
    ヨハネの黙示録:REV(1:22)";

pub fn run(args: &[String]) -> Result<()> {
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        println!("{}", HELP_MSG);
        return Ok(());
    }
    if args.len() != 2 {
        bail!("{}", HELP_MSG);
    }

    let (book, chapter) = (&args[0].to_uppercase(), &args[1]);

    let url = format!("https://www.bible.com/ja/bible/1819/{book}.{chapter}/");

    let output = Command::new("curl")
        .args(["-sSL", "-A", "Mozilla/5.0", &url])
        .output()
        .context("failed to run 'curl'")?;

    let contents = String::from_utf8_lossy(&output.stdout);

    let re = Regex::new(r#"(?ms)content">(.+?)</span>"#)
        .context("failed to compile verse.run.re (regex)")?;

    let mut text = String::new();
    let mut caps_iter = re.captures_iter(&contents).peekable();
    if caps_iter.peek().is_some() {
        for caps in re.captures_iter(&contents) {
            let sentence = caps.get(1).map_or("", |m| m.as_str());

            let sentence = sentence.trim_ascii_start();

            if !sentence.is_empty() {
                text.push_str(sentence);
                text.push('\n');
            }
        }
    } else {
        bail!(format!(
            "no verses found or failed to parse the page: {}",
            url
        ));
    }

    let mut child = Command::new("less")
        .args(["-R", "-i", "--silent"])
        .stdin(Stdio::piped())
        .spawn()
        .context("failed to run 'less'")?;

    // `stdin` を `take()` で完全に外に切り出す
    let mut stdin = child.stdin.take().context("failed to open stdin")?;

    // less への書き込み (途中で q で閉じられた場合の BrokenPipe エラーを許容する)
    if let Err(e) = write!(stdin, "{}[{}]\n{}", book.cyan(), chapter.green(), text)
        && e.kind() != std::io::ErrorKind::BrokenPipe
    {
        let _ = child.kill();
        bail!("error occurred while writing to less: {}", e);
    }

    // 書き込みが完了したため、明示的に `stdin` を閉じて less に通知する
    drop(stdin);

    let _ = child.wait().context("'less' failed")?;

    Ok(())
}
