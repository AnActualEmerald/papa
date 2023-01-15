use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use thermite::model::Mod;
use tracing::debug;

use crate::model::ModName;

const SCORE_THRESHOLD: i32 = 75;

pub trait RemoteIndex {
    fn get_mod(&self, name: &ModName) -> Option<&Mod>;
    fn search(&self, term: &str) -> Vec<&Mod>;
}

impl RemoteIndex for Vec<Mod> {
    fn get_mod(&self, name: &ModName) -> Option<&Mod> {
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
