//! A small calendar CLI. Thin by design: parse arguments, call the library,
//! print. Anything reusable belongs in `calendar` or `task`.

mod render;

use std::error::Error;
use std::io::IsTerminal;

use crate::calendar::BsDate;
use crate::cli::render::render_month;
use crate::{BsMonth, Confidence};
use chrono::NaiveDate;
use render::weekday_name;

type CliResult = Result<(), Box<dyn Error>>;

const USAGE: &str = "\
bikram - Bikram Sambat Calendar

USAGE:
    bikram                      show today (same as `today`)
    bikram today                today in BS and AD
    bikram cal [MONTH] [YEAR]   month grid; MONTH is 1-12, a name or an alias
    bikram to-ad <BS Date>      e.g., 2083-07-25 | 25 kar 2083 | 25 kar | +3d
    bikram to-bs <AD Date>      e.g., 2026-11-09 | today
    bikram months               list months and accepted spellings
    bikram help                 print this message
";

pub fn run<I: IntoIterator<Item = String>>(args: I) -> CliResult {
    let args: Vec<String> = args.into_iter().collect();
    let (cmd, rest) = match args.split_first() {
        Some((c, r)) => (c.as_str(), r),
        None => ("today", &[][..]),
    };

    match cmd {
        "today" => cmd_today(),
        "cal" => cmd_cal(rest),
        "to-ad" => cmd_to_ad(rest),
        "to-bs" => cmd_to_bs(rest),
        "months" => cmd_months(),
        "help" | "-h" | "--help" => {
            print!("{USAGE}");
            Ok(())
        }
        other => Err(format!("unknown command {other:?}\n\n{USAGE}").into()),
    }
}

fn describe(bs: BsDate) -> String {
    format!(
        "{}, {} {} {} BS ({})",
        weekday_name(bs.weekday()),
        bs.day(),
        bs.month_name(),
        bs.year(),
        bs
    )
}

fn warn_if_unverified(bs: BsDate) {
    match bs.confidence() {
        Confidence::Attested => {}
        Confidence::Published => {
            eprint!(
                "note: Bs {} month lengths are published but not cross-checked",
                bs.year()
            )
        }
        Confidence::Provisional => {
            eprintln!(
                "note: BS {} month lengths are provisional and may change",
                bs.year()
            )
        }
    }
}

fn cmd_today() -> CliResult {
    let bs = BsDate::today()?;
    println!("{}", describe(bs));
    println!("{} AD", bs.to_ad());
    warn_if_unverified(bs);
    Ok(())
}

fn cmd_cal(args: &[String]) -> CliResult {
    let today = BsDate::today()?;
    let (year, month) = match args {
        [] => (today.year(), today.month_name()),
        [m] => (today.year(), m.parse::<BsMonth>()?),
        [m, y] => (y.parse::<i32>()?, m.parse::<BsMonth>()?),
        _ => return Err("usage: bikram cal [Month] [Year]".into()),
    };
    let color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    println!(
        "{}",
        render_month(year, month, Some(today), chrono::Weekday::Sun, color)?
    );
    warn_if_unverified(BsDate::new(year, month.number(), 1)?);
    Ok(())
}

fn cmd_to_ad(args: &[String]) -> CliResult {
    if args.is_empty() {
        return Err("usage: bikram to-ad <BS DATE>".into());
    }
    let bs = BsDate::parse_relative_to(&args.join(" "), BsDate::today()?)?;
    println!("{}", describe(bs));
    println!("{} AD", bs.to_ad());
    warn_if_unverified(bs);
    Ok(())
}

fn cmd_to_bs(args: &[String]) -> CliResult {
    let input = args.join(" ");
    let ad = if input.trim().eq_ignore_ascii_case("today") {
        BsDate::today()?.to_ad()
    } else {
        NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d")
            .map_err(|_| "usage: bikram to-bs <YYYY-MM-DD | today>")?
    };
    let bs = BsDate::from_ad(ad)?;
    println!("{ad} AD");
    println!("{}", describe(bs));
    warn_if_unverified(bs);
    Ok(())
}

fn cmd_months() -> CliResult {
    for m in BsMonth::ALL {
        let aliases: Vec<_> = m.aliases().skip(1).collect();
        println!(
            "{:>2} {:<8} {} also: {}",
            m.number(),
            m.name(),
            m.abbr(),
            aliases.join(", ")
        );
    }
    Ok(())
}
