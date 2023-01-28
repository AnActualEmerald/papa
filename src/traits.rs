use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use thermite::model::{InstalledMod, Mod};
use thermite::prelude::ThermiteError;
use tracing::debug;

use crate::model::ModName;

const SCORE_THRESHOLD: i32 = 75;

pub trait Index<T> {
    fn get_item(&self, name: &ModName) -> Option<&T>;
    fn search(&self, term: &str) -> Vec<&T>;
}

impl Index<Mod> for Vec<Mod> {
    fn get_item(&self, name: &ModName) -> Option<&Mod> {
        self.iter().find(|v| {
            v.name.to_lowercase() == name.name.to_lowercase()
                && v.author.to_lowercase() == name.author.to_lowercase()
        })
    }

    fn search(&self, term: &str) -> Vec<&Mod> {
        if term.len() == 0 {
            return self.iter().collect();
        }
        let matcher = SkimMatcherV2::default();
        let mut res = vec![];
        for v in self.iter() {
            let author = matcher.fuzzy_indices(&v.author, term);
            let name = matcher.fuzzy_indices(&v.name, term);
            let desc = matcher.fuzzy_indices(&v.get_latest().unwrap().desc, term);

            if let Some((score, _)) = author {
                debug!("author matched with score '{score}'");
                if score >= SCORE_THRESHOLD as i64 {
                    res.push((score, v));
                }
            } else if let Some((score, _)) = name {
                debug!("name matched with score '{score}'");
                if score >= SCORE_THRESHOLD as i64 {
                    res.push((score, v));
                }
            } else if let Some((score, _)) = desc {
                debug!("desc matched with score '{score}'");
                if score >= SCORE_THRESHOLD as i64 {
                    res.push((score, v));
                }
            }
        }

        res.sort_by(|l, r| l.0.cmp(&r.0));
        res.iter().map(|v| v.1).rev().collect()
    }
}

impl Index<InstalledMod> for Vec<Result<InstalledMod, ThermiteError>> {
    fn get_item(&self, name: &ModName) -> Option<&InstalledMod> {
        self.iter()
            .filter_map(|v| v.as_ref().ok())
            .find(|v| v.mod_json.name.to_lowercase() == name.name.to_lowercase())
    }

    fn search(&self, term: &str) -> Vec<&InstalledMod> {
        if term.len() == 0 {
            return self.iter().filter_map(|v| v.as_ref().ok()).collect();
        }
        let matcher = SkimMatcherV2::default();
        let mut res = vec![];
        for v in self.iter().filter_map(|v| v.as_ref().ok()) {
            let author = matcher.fuzzy_indices(&v.author, term);
            let name = matcher.fuzzy_indices(&v.manifest.name, term);

            if let Some((score, _)) = author {
                debug!("author matched with score '{score}'");
                if score >= SCORE_THRESHOLD as i64 {
                    res.push((score, v));
                }
            } else if let Some((score, _)) = name {
                debug!("name matched with score '{score}'");
                if score >= SCORE_THRESHOLD as i64 {
                    res.push((score, v));
                }
            }
        }

        res.sort_by(|l, r| l.0.cmp(&r.0));
        res.iter().map(|v| v.1).rev().collect()
    }
}
