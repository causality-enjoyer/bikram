use std::fmt;
use std::str::FromStr;

use super::error::ParseError;

/// A BS month. Drisciminants are the monthh numbers (Baisakh = 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BsMonth {
    Baisakh = 1,
    Jestha,
    Asar,
    Shrawan,
    Bhadra,
    Ashoj,
    Kartik,
    Mangsir,
    Poush,
    Magh,
    Falgun,
    Chaitra,
}

use BsMonth::*;

impl BsMonth {
    pub const ALL: [BsMonth; 12] = [
        Baisakh, Jestha, Asar, Shrawan, Bhadra, Ashoj, Kartik, Mangsir, Poush, Magh, Falgun,
        Chaitra,
    ];

    pub fn number(self) -> u8 {
        self as u8
    }

    pub fn from_number(n: u8) -> Option<BsMonth> {
        n.checked_sub(1)
            .and_then(|i| Self::ALL.get(i as usize))
            .copied()
    }

    /// Canonical full name
    pub fn name(self) -> &'static str {
        match self {
            Baisakh => "Baisakh",
            Jestha => "Jestha",
            Asar => "Asar",
            Shrawan => "Shrawan",
            Bhadra => "Bhadra",
            Ashoj => "Ashoj",
            Kartik => "Kartik",
            Mangsir => "Mangsir",
            Poush => "Poush",
            Magh => "Magh",
            Falgun => "Falgun",
            Chaitra => "Chaitra",
        }
    }

    /// The standard 3-letter form.
    /// Used for display and always accepted as input
    pub fn abbr(self) -> &'static str {
        match self {
            Baisakh => "Bai",
            Jestha => "Jes",
            Asar => "Asa",
            Shrawan => "Shr",
            Bhadra => "Bha",
            Ashoj => "Ash",
            Kartik => "Kar",
            Mangsir => "Man",
            Poush => "Pou",
            Magh => "Mag",
            Falgun => "Fal",
            Chaitra => "Cha",
        }
    }

    /// Next month, wrapping Chaitra -> Baisakh
    pub fn next(self) -> BsMonth {
        Self::ALL[self.number() as usize % 12]
    }

    /// Previous month, wrapping Baisakh -> Chaitra
    pub fn prev(self) -> BsMonth {
        Self::ALL[(self.number() as usize + 10) % 12]
    }

    /// Every accepted spelling (lowercase), canonical name first
    pub fn aliases(self) -> impl Iterator<Item = &'static str> {
        ALIASES
            .iter()
            .filter(move |(_, m)| *m == self)
            .map(|(a, _)| *a)
    }
}

/// Exact, **case-insensitive** spellings. No prefix matching: the table is the standard,
/// so there is never an "ambiguous abbreviation" case to explain.
/// `tests::aliases_are_unambiguous` guards against one spelling meaning two months.
#[rustfmt::skip]
const ALIASES: &[(&str, BsMonth)] = &[
    ("baisakh", Baisakh), ("baisakha", Baisakh), ("bai", Baisakh), ("ba", Baisakh),
    ("jestha", Jestha), ("jeth", Jestha), ("jes", Jestha), ("jth", Jestha), ("je", Jestha),
    ("asar", Asar), ("asadh", Asar), ("asa", Asar), ("asr", Asar), ("asd", Asar),
    ("shrawan", Shrawan), ("saun", Shrawan), ("shr", Shrawan), ("sh", Shrawan),
    ("bhadra", Bhadra), ("bhadau", Bhadra), ("bha", Bhadra), ("bh", Bhadra),
    ("ashoj", Ashoj), ("ashwin", Ashoj), ("ash", Ashoj), ("asn", Ashoj), ("asj", Ashoj),
    ("kartik", Kartik), ("kattik", Kartik), ("kar", Kartik), ("ka", Kartik),
    ("mangsir", Mangsir), ("mansir", Mangsir), ("man", Mangsir), ("mn", Mangsir),
    ("poush", Poush), ("push", Poush), ("pou", Poush), ("pu", Poush),
    ("magh", Magh), ("maghi", Magh), ("mag", Magh), ("ma", Magh),
    ("falgun", Falgun), ("fagun", Falgun), ("fal", Falgun), ("fa", Falgun),
    ("chaitra", Chaitra), ("chait", Chaitra), ("cha", Chaitra), ("ch", Chaitra),
];

impl FromStr for BsMonth {
    type Err = ParseError;

    /// Accepts a number `1..=12`, a canonical name, a 3-letter form, or an alias
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let unknown = || ParseError::UnknownMonth(s.to_string());
        if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) {
            let n: u8 = s.parse().map_err(|_| unknown())?;
            return BsMonth::from_number(n).ok_or_else(unknown);
        }
        ALIASES
            .iter()
            .find(|(a, _)| a.eq_ignore_ascii_case(s))
            .map(|(_, m)| *m)
            .ok_or_else(unknown)
    }
}

impl fmt::Display for BsMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_are_unambiguous() {
        for (i, (a, _)) in ALIASES.iter().enumerate() {
            for (b, _) in &ALIASES[i + 1..] {
                assert!(!a.eq_ignore_ascii_case(b), "duplicate alias {a}");
            }
        }
    }

    #[test]
    fn names_and_abbrs_round_trip() {
        for m in BsMonth::ALL {
            assert_eq!(m.name().parse::<BsMonth>().unwrap(), m);
            assert_eq!(m.abbr().parse::<BsMonth>().unwrap(), m);
            assert_eq!(m.abbr().len(), 3);
            assert_eq!(BsMonth::from_number(m.number()), Some(m));
            assert_eq!(m.aliases().next().unwrap(), m.name().to_ascii_lowercase());
        }
    }

    #[test]
    fn user_aliases() {
        assert_eq!("KAR".parse::<BsMonth>().unwrap(), Kartik);
        assert_eq!("ashwin".parse::<BsMonth>().unwrap(), Ashoj);
        assert_eq!("asadh".parse::<BsMonth>().unwrap(), Asar);
        assert_eq!("7".parse::<BsMonth>().unwrap(), Kartik);
        assert_eq!("ma".parse::<BsMonth>().unwrap(), Magh);
        assert_eq!("man".parse::<BsMonth>().unwrap(), Mangsir);
        assert!("as".parse::<BsMonth>().is_err()); // deliberately not an alias
        assert!("13".parse::<BsMonth>().is_err());
        assert!("0".parse::<BsMonth>().is_err());
    }

    #[test]
    fn wraps() {
        assert_eq!(Chaitra.next(), Baisakh);
        assert_eq!(Baisakh.prev(), Chaitra);
        assert_eq!(Kartik.next(), Mangsir);
        assert_eq!(Kartik.prev(), Ashoj);
    }
}
