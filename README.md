# BIKRAM

This library is created as a first step towards learning rust from scratch. I've learned somewhat C, so I thought I should do something in Rust as well. So this is some of the first things I've built. So here it is, `bikram`.

`bikram` is a rust library that handles nepali date, and is primarily built to accomplish one thing: Render Nepali Dates. The backend calculation is based on AD (Gregorian), and only display dates are in Nepali Bikram Sambat (and hence the name).

## Dependency:

- The only dependency I've used so far is `chrono`. I needed someway to at least have easy access to AD dates.

# Project Itself

## Directory Structure

```txt
src
├── main.rs
├── lib.rs
├── calendar
│   ├── confidence.rs
│   ├── data.rs
│   ├── date.rs
│   ├── error.rs
│   ├── mod.rs
│   ├── month.rs
│   ├── parse.rs
│   ├── range.rs
│   └── table.rs
├── cli
│   ├── mod.rs
│   └── render.rs
└── task
    ├── item.rs
    ├── mod.rs
    ├── recurrence.rs
    └── when.rs
```

### Modules:

- Calendar: the main backend
- CLI: just a simple cli interface to talk with calendar backend
- task: not integrated with main perform or any calculation, just tested for now.

The rule I'm following is that dependencies only go one way: `cli -> task -> calendar`. `calendar` knows nothing about tasks, and `task` knows nothing about the cli. That way when a GUI comes, it's just one more thing sitting next to `cli`.

## How it works (tl;dr)

BS has no formula. Month lengths change every year, so everything is looked up from a table (`calendar/data.rs`) covering BS 2000 to 2090.

Every BS date gets turned into a "day index", meaning how many days since 1 Baisakh 2000. From there AD is just `anchor date + day index`, and `chrono` does the Gregorian part. Going back is the same thing in reverse. I have a test that converts every single day in the table to AD and back, so if the table has a broken row, that test should scream.

Some choices I made along the way:

- **AD is the truth, BS is for display.** Dates would be stored as AD and only shown as BS.
- **Nepal time.** A BS day changes at Nepal midnight (UTC+05:45), not UTC midnight. So converting an instant to BS goes through that offset.
- **Month names are a fixed table.** `kar`, `kartik` and `kattik` all mean Kartik. No prefix guessing, so there's no "is `as` Asar or Ashwin?" problem. `as` is simply rejected.
- **Data confidence.** Not every year in the table is equally trusted. Up to 2080 is confirmed by multiple sources, up to 2083 is officially published, and after that it's provisional (extrapolated, might be wrong). The cli prints a small note when you touch those years.

## Currently completed:

- BS <-> AD conversion, both directions, for BS 2000 to 2090
- Weekday for any date
- Date arithmetic: add days, months, years (a day that doesn't exist in the target month gets clamped, so 32 Jestha + 1 month = 31 Asar)
- Ranges: month, week (start day is a parameter), year, with iteration
- Parsing: `2083-07-25`, `25 Kartik 2083`, `25 kar`, `today`, `tomorrow`, `+3d`, `-1w`, `+1m`, `+1y`
- Month names with short aliases, 3 letter ones are the standard
- Basic cli (see below)
- Tasks: `Item`, `When` (whole day or exact moment) and `Recurrence` (daily, weekly, monthly, yearly in either AD or BS). Tested, but not connected to anything yet.

## Using the cli

```sh
cargo run -- cal kar 2083       # month grid, today highlighted
cargo run -- to-ad 25 kar       # BS -> AD
cargo run -- to-bs 2026-11-09   # AD -> BS
cargo run -- months             # month list and accepted spellings
cargo run -- help               # get list of commands
cargo run                       # no args = today
```

Or install it and use it anywhere:

```sh
cargo install --path .
bikram cal
```

## Using it as a library

```rust
use bikram::BsDate;

let bs = BsDate::new(2081, 1, 1).unwrap();
let ad = bs.to_ad(); // a chrono::NaiveDate, 2024-04-13
let back = BsDate::from_ad(ad).unwrap();
assert_eq!(bs, back);
```

## Testing

```sh
cargo test
```

## Known limitations:

- Only BS 2000 to 2090 (AD 1943 to 2034). Anything outside gives an `OutOfRange` error.
- Years after 2083 are provisional, so those dates may shift if the official data differs.
- No holidays, tithi or festivals. Just the calendar.
- Tasks have no storage yet, everything lives in memory.

## Future Scope:

- Storage for tasks (probably SQLite or plain files)
- `add` / `list` commands in the cli, and switching to `clap` once there are enough commands
- Complete tauri and a basic app
- Export to iCalendar (AD recurrence rules already map to RFC 5545, BS ones will need a custom property)

## Acknowledgement:

- [Patro Repo by Yogesh Ojha](https://github.com/yogeshojha/patro/)
- And of course rust doc for syntax/api explanation.

