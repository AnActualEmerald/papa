use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use thermite::model::{InstalledMod, Mod};
use tracing::debug;

use crate::model::ModName;

const SCORE_THRESHOLD: i64 = 75;

pub trait Answer {
    fn is_no(&self) -> bool;
    fn is_yes(&self) -> bool;
}

pub trait Indexed<T> {
    fn get_item(&self, name: &ModName) -> Option<&T>;
    fn search(&self, term: &str) -> Vec<&T>;
}

impl Indexed<Mod> for Vec<Mod> {
    fn get_item(&self, name: &ModName) -> Option<&Mod> {
        self.iter().find(|v| {
            v.name.to_lowercase() == name.name.to_lowercase()
                && v.author.to_lowercase() == name.author.to_lowercase()
        })
    }

    fn search(&self, term: &str) -> Vec<&Mod> {
        if term.is_empty() {
            return self.iter().collect();
        }
        let matcher = SkimMatcherV2::default();
        let mut res = vec![];
        for v in self {
            let author = matcher.fuzzy_indices(&v.author, term);
            let name = matcher.fuzzy_indices(&v.name, term);
            let desc = matcher.fuzzy_indices(&v.get_latest().unwrap().desc, term);

            if let Some((score, _)) = author {
                debug!("author matched with score '{score}'");
                if score >= SCORE_THRESHOLD {
                    res.push((score, v));
                }
            } else if let Some((score, _)) = name {
                debug!("name matched with score '{score}'");
                if score >= SCORE_THRESHOLD {
                    res.push((score, v));
                }
            } else if let Some((score, _)) = desc {
                debug!("desc matched with score '{score}'");
                if score >= SCORE_THRESHOLD {
                    res.push((score, v));
                }
            }
        }

        res.sort_by(|l, r| l.0.cmp(&r.0));
        res.iter().map(|v| v.1).rev().collect()
    }
}

impl Indexed<InstalledMod> for Vec<InstalledMod> {
    fn get_item(&self, name: &ModName) -> Option<&InstalledMod> {
        self.iter()
            .find(|v| v.mod_json.name.to_lowercase() == name.name.to_lowercase())
    }

    fn search(&self, term: &str) -> Vec<&InstalledMod> {
        if term.is_empty() {
            return self.iter().collect();
        }
        let matcher = SkimMatcherV2::default();
        let mut res = vec![];
        for v in self {
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

impl Answer for String {
    fn is_no(&self) -> bool {
        self.to_lowercase().trim().starts_with('n')
    }

    fn is_yes(&self) -> bool {
        self.to_lowercase().trim().starts_with('y')
    }
}
