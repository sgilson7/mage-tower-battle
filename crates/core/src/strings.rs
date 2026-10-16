//! Every word a player reads, from `data/strings.json`: a flat map from a
//! key to a string, with `{placeholders}` the core fills. A missing key
//! shows as `[key]`, which the tests look for.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct Strings {
    map: BTreeMap<String, String>,
}

impl Strings {
    pub fn parse(json: &str) -> Result<Strings, String> {
        let raw: BTreeMap<String, serde_json::Value> =
            serde_json::from_str(json).map_err(|e| format!("The strings file could not be read: {e}"))?;
        let mut map = BTreeMap::new();
        for (k, v) in raw {
            if k.starts_with('_') {
                continue;
            }
            let s = v
                .as_str()
                .ok_or_else(|| format!("The string \"{k}\" is not text."))?;
            if s.trim().is_empty() {
                return Err(format!("The strings file has an empty \"{k}\"."));
            }
            map.insert(k, s.to_string());
        }
        Ok(Strings { map })
    }

    pub fn has(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn t(&self, key: &str) -> String {
        self.map.get(key).cloned().unwrap_or_else(|| format!("[{key}]"))
    }

    /// A string with its placeholders filled: `("n", "3")` fills `{n}`.
    pub fn f(&self, key: &str, fill: &[(&str, String)]) -> String {
        let mut s = self.t(key);
        for (name, value) in fill {
            s = s.replace(&format!("{{{name}}}"), value);
        }
        s
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }
}

/// Whole numbers from comma-separated text the page sent, dropping any
/// fraction: "123.5,40" is [123, 40]. The page sends pointer positions as
/// the browser reports them and never rounds them itself.
pub fn ints(value: &str) -> Vec<i32> {
    value
        .split(',')
        .filter_map(|s| s.trim().split('.').next().and_then(|w| w.parse().ok()))
        .collect()
}

/// Minutes and seconds from ticks at 60 a second: 3:05.
pub fn clock(ticks: i32) -> String {
    let s = (ticks.max(0) + crate::geom::TPS - 1) / crate::geom::TPS;
    format!("{}:{:02}", s / 60, s % 60)
}

/// A whole number with thousands marked: 10,000.
pub fn number(n: i32) -> String {
    let neg = n < 0;
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if neg {
        format!("−{out}")
    } else {
        out
    }
}

/// A percentage as CSS text: 37.5 → "37.5%".
pub fn css_pct(thousandths: i32) -> String {
    format!("{}.{}%", thousandths / 10, (thousandths % 10).abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_fill_and_a_missing_key_shows_itself() {
        let s = Strings::parse(r#"{"a": "{n} books left", "_note": 3}"#).unwrap();
        assert_eq!(s.f("a", &[("n", "12".into())]), "12 books left");
        assert_eq!(s.t("b"), "[b]");
        assert!(Strings::parse(r#"{"a": " "}"#).is_err());
    }

    #[test]
    fn clocks_numbers_and_percentages_read_as_people_write_them() {
        assert_eq!(clock(60 * 185), "3:05");
        assert_eq!(clock(1), "0:01");
        assert_eq!(clock(0), "0:00");
        assert_eq!(number(10000), "10,000");
        assert_eq!(number(-1234567), "−1,234,567");
        assert_eq!(number(999), "999");
        assert_eq!(css_pct(375), "37.5%");
        assert_eq!(ints("123.5,40,-3.9,x"), vec![123, 40, -3]);
    }
}
