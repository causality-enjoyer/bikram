//! Accepted shapes (separators: space `-` `/` `.` `,`):
//!
//! | input                      | needs `today`? |
//! |----------------------------|----------------|
//! | `2083-07-25`, `2083/kar/25`| no             |
//! | `25 Kartik 2083`           | no             |
//! | `25 kar`, `kar 25`, `7-25` | yes (year)     |
//! | `today` `tomorrow` `yesterday` | yes        |
//! | `+3d` `-2w` `+1m` `+1y`    | yes            |

use super::date::BsDate;
use super::error::ParseError;
use super::month::BsMonth;

fn parse_relative(lower: &str, today: BsDate) -> Option<Result<BsDate, ParseError>> {
    match lower {
        "today" => return Some(Ok(today)),
        "tomorrow" => return Some(today.add_days(1).map_err(Into::into)),
        "yesterday" => return Some(today.add_days(-1).map_err(Into::into)),
        _ => {}
    }

    let (sign, rest) = match lower.as_bytes().first()? {
        b'+' => (1i64, &lower[1..]),
        b'-' => (-1i64, &lower[1..]),
        _ => return None,
    };

    let unit = rest.chars().last()?;
    let n: i64 = rest[..rest.len() - unit.len_utf8()].parse().ok()?;
    let n = sign * n;
    let result = match unit {
        'd' => today.add_days(n),
        'w' => today.add_days(n.checked_mul(7)?),
        'm' => today.add_months(i32::try_from(n).ok()?),
        'y' => today.add_years(i32::try_from(n).ok()?),
        _ => return None,
    };
    Some(result.map_err(Into::into))
}

fn is_number(t: &str) -> bool {
    !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit())
}

fn is_year(t: &str) -> bool {
    t.len() == 4 && is_number(t)
}

fn num(t: &str) -> Result<i32, ParseError> {
    t.parse::<i32>()
        .map_err(|_| ParseError::BadFormat(t.to_string()))
}

pub(super) fn parse(input: &str, today: Option<BsDate>) -> Result<BsDate, ParseError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ParseError::Empty);
    }

    let bad = || ParseError::BadFormat(input.trim().to_string());

    if let Some(today) = today
        && let Some(result) = parse_relative(&s.to_ascii_lowercase(), today)
    {
        return result;
    }

    let tokens: Vec<&str> = s
        .split(|c: char| c.is_whitespace() || matches!(c, '-' | '/' | '.' | ','))
        .filter(|t| !t.is_empty())
        .collect();

    let (year, month, day) = match tokens.as_slice() {
        [a, b, c] if is_year(a) => (num(a)?, b.parse::<BsMonth>()?, num(c)?),
        [a, b, c] if is_year(c) => (num(c)?, b.parse::<BsMonth>()?, num(a)?),
        [a, b] => {
            let today = today.ok_or_else(bad)?;
            if is_number(a) && !is_number(b) {
                (today.year(), b.parse::<BsMonth>()?, num(a)?) // "25 kar"
            } else {
                (today.year(), a.parse::<BsMonth>()?, num(b)?) // "kar 25", "7-25"
            }
        }
        _ => return Err(bad()),
    };

    let day = u8::try_from(day).map_err(|_| ParseError::Date(super::DateError::NoSuchDate))?;
    Ok(BsDate::new(year, month.number(), day)?)
}
