//! What is kept between visits (a change to Builder's D17, SECOND-ORDER
//! R3.6): trial bests, achievements and the last settings. The page keeps
//! one text in browser storage through `web/store.js` and decides nothing;
//! the core reads it, and an unreadable save is treated as a new one.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::game::Settings;

pub const VERSION: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Save {
    pub version: i32,
    /// trial id → best result in its measure's units
    pub bests: BTreeMap<String, i32>,
    pub achievements: Vec<String>,
    pub settings: Option<Settings>,
    pub muted: bool,
}

impl Save {
    /// Reads a stored save. Empty text is a first visit; text that does not
    /// parse, or a different version, starts again and says so.
    pub fn read(text: &str) -> (Save, bool) {
        if text.trim().is_empty() {
            return (
                Save {
                    version: VERSION,
                    ..Default::default()
                },
                false,
            );
        }
        match serde_json::from_str::<Save>(text) {
            Ok(s) if s.version == VERSION => (s, false),
            _ => (
                Save {
                    version: VERSION,
                    ..Default::default()
                },
                true,
            ),
        }
    }

    pub fn text(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Records an achievement; true if it is new.
    pub fn earn(&mut self, id: &str) -> bool {
        if self.achievements.iter().any(|a| a == id) {
            return false;
        }
        self.achievements.push(id.to_string());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_save_round_trips_and_an_unreadable_one_starts_again() {
        let mut s = Save {
            version: VERSION,
            ..Default::default()
        };
        s.bests.insert("errand_run".into(), 92);
        assert!(s.earn("first_spell"));
        assert!(!s.earn("first_spell"), "earned once");
        let (back, broken) = Save::read(&s.text());
        assert_eq!(back, s);
        assert!(!broken);
        let (fresh, broken) = Save::read("{not json");
        assert!(broken);
        assert!(fresh.bests.is_empty());
        let (first, broken) = Save::read("");
        assert!(!broken && first.version == VERSION);
        let (old, broken) = Save::read(
            r#"{"version": 0, "bests": {}, "achievements": [], "settings": null, "muted": false}"#,
        );
        assert!(broken && old.achievements.is_empty());
    }
}
