use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;
use zeroize::Zeroizing;

pub const LOWERCASE: &str = "abcdefghijkmnopqrstuvwxyz";
pub const UPPERCASE: &str = "ABCDEFGHJKLMNPQRSTUVWXYZ";
pub const NUMBERS: &str = "23456789";
pub const SYMBOLS: &str = "!@#$%^&*";
pub const DEFAULT_FORMAT: &str = "{word:capitalize}-{word}-{word}-{number:3}";
const WORDS: &str = include_str!("english.txt");
pub const ENTROPY_ERROR: &str = "Could not obtain secure randomness. Generate again.";

// Kept crate/module private: normal callers cannot choose deterministic randomness.
pub(super) trait Choice {
    fn choose(&mut self, upper: usize) -> Result<usize, String>;
}
pub(super) struct OsChoice;
impl Choice for OsChoice {
    fn choose(&mut self, upper: usize) -> Result<usize, String> {
        bounded(upper, |bytes| getrandom::getrandom(bytes).map_err(|_| ()))
    }
}
fn bounded(
    upper: usize,
    mut fill: impl FnMut(&mut [u8]) -> Result<(), ()>,
) -> Result<usize, String> {
    if upper == 0 || upper > u32::MAX as usize {
        return Err(ENTROPY_ERROR.into());
    }
    let upper = upper as u64;
    let limit = (1_u64 << 32) / upper * upper;
    // Bound work on a faulty source; fail closed, never use rejected bytes.
    for _ in 0..128 {
        let mut bytes = [0_u8; 4];
        fill(&mut bytes).map_err(|_| ENTROPY_ERROR.to_owned())?;
        let value = u32::from_le_bytes(bytes) as u64;
        if value < limit {
            return Ok((value % upper) as usize);
        }
    }
    Err(ENTROPY_ERROR.into())
}
fn pick(chars: &str, rng: &mut impl Choice) -> Result<char, String> {
    let i = rng.choose(chars.len())?;
    chars
        .as_bytes()
        .get(i)
        .map(|b| *b as char)
        .ok_or_else(|| ENTROPY_ERROR.into())
}
pub fn js_trim(s: &str) -> &str {
    s.trim_matches(|c| {
        matches!(c,
        '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}' |
        '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' |
        '\u{205f}' | '\u{3000}' | '\u{feff}')
    })
}
pub fn parse_length(s: &str) -> Result<usize, String> {
    let s = js_trim(s);
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Enter a whole number from 5 to 64.".into());
    }
    // JS Number accepts arbitrarily many leading zeros (and rounds large values).
    let n = s.parse::<f64>().unwrap_or(f64::INFINITY);
    if !(5.0..=64.0).contains(&n) {
        return Err("Password length must be between 5 and 64.".into());
    }
    Ok(n as usize)
}
pub(super) fn random_with(
    length: usize,
    numbers: bool,
    symbols: bool,
    rng: &mut impl Choice,
) -> Result<Zeroizing<String>, String> {
    if !(5..=64).contains(&length) {
        return Err("Password length must be between 5 and 64.".into());
    }
    let mut groups = vec![LOWERCASE, UPPERCASE];
    if numbers {
        groups.push(NUMBERS);
    }
    if symbols {
        groups.push(SYMBOLS);
    }
    let all = groups.join("");
    let mut out = Zeroizing::new(Vec::<u8>::with_capacity(length));
    for group in groups {
        out.push(pick(group, rng)? as u8);
    }
    while out.len() < length {
        out.push(pick(&all, rng)? as u8);
    }
    for i in (1..out.len()).rev() {
        let j = rng.choose(i + 1)?;
        if j > i {
            return Err(ENTROPY_ERROR.into());
        }
        out.swap(i, j);
    }
    Ok(Zeroizing::new(
        String::from_utf8(out.to_vec()).map_err(|_| ENTROPY_ERROR.to_owned())?,
    ))
}
fn token_length(s: Option<&str>) -> Result<usize, String> {
    let Some(s) = s else { return Ok(1) };
    // The regex permits only ASCII JS \w: Number's radix prefixes, decimal
    // leading zero and exponent syntax are intentionally retained.
    let radix = if s.starts_with("0x") || s.starts_with("0X") {
        Some(16)
    } else if s.starts_with("0b") || s.starts_with("0B") {
        Some(2)
    } else if s.starts_with("0o") || s.starts_with("0O") {
        Some(8)
    } else {
        None
    };
    let n = if let Some(radix) = radix {
        let digits = &s[2..];
        if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
            f64::NAN
        } else {
            digits.chars().fold(0.0, |acc, c| {
                acc * f64::from(radix) + f64::from(c.to_digit(radix).unwrap_or(0))
            })
        }
    } else {
        s.parse::<f64>().unwrap_or(f64::NAN)
    };
    if !n.is_finite() || n.fract() != 0.0 || !(1.0..=64.0).contains(&n) {
        return Err("Token lengths must be whole numbers from 1 to 64.".into());
    }
    Ok(n as usize)
}
pub(super) fn memorable_with(
    format: &str,
    rng: &mut impl Choice,
) -> Result<Zeroizing<String>, String> {
    if format.is_empty() || format.encode_utf16().count() > 256 {
        return Err("Provide a format up to 256 characters.".into());
    }
    static TOKENS: OnceLock<Regex> = OnceLock::new();
    let tokens = TOKENS.get_or_init(|| {
        Regex::new(r"\{(word|number|symbol|random)(?::([A-Za-z0-9_]+))?\}")
            .expect("constant token regex")
    });
    let words: Vec<_> = WORDS.lines().collect();
    let mut used = HashSet::new();
    let mut result = Zeroizing::new(String::new());
    let mut last = 0;
    let mut draw_budget = 32768; // Bounded duplicate-word work; entropy failure is closed.
    for captures in tokens.captures_iter(format) {
        let token = captures.get(0).expect("capture 0");
        result.push_str(&format[last..token.start()]);
        last = token.end();
        let modifier = captures.get(2).map(|m| m.as_str());
        if &captures[1] == "word" {
            let word = loop {
                if draw_budget == 0 {
                    return Err(ENTROPY_ERROR.into());
                }
                draw_budget -= 1;
                let index = rng.choose(words.len())?;
                let word = *words.get(index).ok_or(ENTROPY_ERROR)?;
                if !used.contains(word) || used.len() == words.len() {
                    break word;
                }
            };
            used.insert(word);
            match modifier {
                None => {
                    if rng.choose(2)? == 0 {
                        result.push_str(word);
                    } else {
                        result.push_str(&word.to_ascii_uppercase());
                    }
                }
                Some("uppercase") => result.push_str(&word.to_ascii_uppercase()),
                Some("lowercase") => result.push_str(word),
                Some("capitalize") => {
                    result.push((word.as_bytes()[0] as char).to_ascii_uppercase());
                    result.push_str(&word[1..]);
                }
                Some(m) => return Err(format!("Unsupported word modifier: {m}")),
            }
        } else {
            let len = token_length(modifier)?;
            let all = format!("{LOWERCASE}{UPPERCASE}{NUMBERS}{SYMBOLS}");
            let chars = match &captures[1] {
                "number" => NUMBERS,
                "symbol" => SYMBOLS,
                _ => &all,
            };
            for _ in 0..len {
                result.push(pick(chars, rng)?);
            }
        }
    }
    result.push_str(&format[last..]);
    if !(5..=128).contains(&result.encode_utf16().count()) {
        return Err("The generated password must contain 5 to 128 characters.".into());
    }
    Ok(result)
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
