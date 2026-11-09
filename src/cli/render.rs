use std::fmt::Write;

use chrono::Weekday;

use crate::calendar::{BsDate, BsMonth, BsRange, DateError};

pub fn weekday_name(w: Weekday) -> &'static str {
    match w {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

/// A 20-column month grid, like `cal(1)`, with the AD span underneath
/// `today` (if inside the month) is highlighted when `color` is on.
pub fn render_month(
    year: i32,
    month: BsMonth,
    today: Option<BsDate>,
    week_start: Weekday,
    color: bool,
) -> Result<String, DateError> {
    let range = BsRange::month(year, month.number())?;
    let (ad_first, ad_last) = range.to_ad();
    let mut out = String::new();

    let title = format!("{} {} ({})", month.name(), year, month.abbr());
    let _ = writeln!(out, "{}", format!("{title:^20}").trim_end());

    let mut day = week_start;
    let mut header = Vec::with_capacity(7);

    for _ in 0..7 {
        header.push(&weekday_name(day)[..2]);
        day = day.succ();
    }
    let _ = writeln!(out, "{}", header.join(" "));

    let lead = range.start().weekday().days_since(week_start) as usize;
    out.push_str(&"   ".repeat(lead));

    for (i, d) in range.iter().enumerate() {
        if color && Some(d) == today {
            let _ = write!(out, "\x1b[7m{:>2}\x1b[0m ", d.day());
        } else {
            let _ = write!(out, "{:>2} ", d.day());
        }

        if (lead + i + 1).is_multiple_of(7) {
            out.truncate(out.trim_end().len());
            out.push('\n');
        }
    }

    out.truncate(out.trim_end().len());
    out.push('\n');

    let span = if ad_first.format("%Y").to_string() == ad_last.format("%Y").to_string() {
        format!(
            "{} - {}",
            ad_first.format("%b %-d"),
            ad_last.format("%b %-d, %Y")
        )
    } else {
        format!(
            "{} - {}",
            ad_first.format("%b %-d, %Y"),
            ad_last.format("%b %-d, %Y")
        )
    };
    let _ = writeln!(out, "\n{span:^20}");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baisakh_2081_grid() {
        // 1 Baisakh 2081 is a Saturday, so six cells of lead-in.
        let s = render_month(2081, BsMonth::Baisakh, None, Weekday::Sun, false).unwrap();
        let lines: Vec<_> = s.lines().collect();
        assert_eq!(lines[0].trim(), "Baisakh 2081 (Bai)");
        assert_eq!(lines[1], "Su Mo Tu We Th Fr Sa");
        assert_eq!(lines[2], "                   1");
        assert_eq!(lines[3], " 2  3  4  5  6  7  8");
        assert!(s.contains("Apr 13 - May 13, 2024"));
    }
}
