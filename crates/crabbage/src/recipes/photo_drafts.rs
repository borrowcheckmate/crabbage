//! Recipes read from photos, kept in memory until the user reviews them. A draft belongs to the
//! user who uploaded the photos and is found by a random token.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use super::types::ImportPreview;

const LIFETIME: Duration = Duration::from_hours(1);
const MAX_ENTRIES: usize = 100;
const TOKEN_BYTES: usize = 16;

type Entry = (i64, ImportPreview, Instant);

/// The photo imports waiting for review.
#[derive(Default)]
pub(crate) struct PhotoDrafts {
    entries: Mutex<HashMap<String, Entry>>,
}

impl PhotoDrafts {
    /// Stores `preview` for `user` at time `now` and returns its token. Expired entries are dropped
    /// first, and the oldest ones when the store is full.
    ///
    /// # Errors
    ///
    /// Returns an error if the system random source fails.
    pub(crate) fn insert(
        &self,
        user: i64,
        preview: ImportPreview,
        now: Instant,
    ) -> Result<String, getrandom::Error> {
        let token = token()?;
        self.locked(|entries| Self::add(entries, token.clone(), (user, preview, now)));
        Ok(token)
    }

    fn locked<R>(&self, f: impl FnOnce(&mut HashMap<String, Entry>) -> R) -> R {
        f(&mut self.entries.lock().unwrap_or_else(PoisonError::into_inner))
    }

    fn add(entries: &mut HashMap<String, Entry>, token: String, entry: Entry) {
        let now = entry.2;
        entries.retain(|_, (_, _, made)| now.saturating_duration_since(*made) < LIFETIME);
        while entries.len() >= MAX_ENTRIES {
            let Some(oldest) = entries
                .iter()
                .min_by_key(|(_, (_, _, made))| *made)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            entries.remove(&oldest);
        }
        entries.insert(token, entry);
    }

    /// The preview under `token` when it belongs to `user` and is younger than an hour at `now`.
    pub(crate) fn get(&self, token: &str, user: i64, now: Instant) -> Option<ImportPreview> {
        self.locked(|entries| {
            let (owner, preview, made) = entries.get(token)?;
            (*owner == user && now.saturating_duration_since(*made) < LIFETIME)
                .then(|| preview.clone())
        })
    }
}

fn token() -> Result<String, getrandom::Error> {
    let mut bytes = [0_u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes)?;
    Ok(bytes
        .iter()
        .flat_map(|&byte| [byte >> 4, byte & 0x0F])
        .filter_map(|nibble| char::from_digit(u32::from(nibble), 16))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preview(title: &str) -> ImportPreview {
        let mut preview = ImportPreview::default();
        preview.draft.title = title.to_owned();
        preview
    }

    #[test]
    fn a_draft_is_found_by_its_token_for_its_user_only() {
        let drafts = PhotoDrafts::default();
        let now = Instant::now();
        let token = drafts.insert(1, preview("Soup"), now).expect("token");
        assert_eq!(token.len(), 32);
        assert_eq!(drafts.get(&token, 1, now), Some(preview("Soup")));
        assert_eq!(drafts.get(&token, 2, now), None);
        assert_eq!(drafts.get("unknown", 1, now), None);
    }

    #[test]
    fn a_draft_expires_after_an_hour_and_is_pruned_on_insert() {
        let drafts = PhotoDrafts::default();
        let now = Instant::now();
        let old = drafts.insert(1, preview("Old"), now).expect("token");
        let later = now + LIFETIME;
        assert_eq!(drafts.get(&old, 1, later), None);
        assert!(drafts.get(&old, 1, now + Duration::from_mins(59)).is_some());
        drafts.insert(1, preview("New"), later).expect("token");
        let count = drafts.entries.lock().expect("lock").len();
        assert_eq!(count, 1);
    }

    #[test]
    fn at_most_one_hundred_drafts_are_kept_and_the_oldest_go_first() {
        let drafts = PhotoDrafts::default();
        let now = Instant::now();
        let first = drafts.insert(1, preview("First"), now).expect("token");
        let mut last = String::new();
        for offset in 1..=100_u64 {
            let at = now + Duration::from_secs(offset);
            last = drafts.insert(1, preview("Next"), at).expect("token");
        }
        let at = now + Duration::from_secs(100);
        assert_eq!(drafts.entries.lock().expect("lock").len(), MAX_ENTRIES);
        assert_eq!(drafts.get(&first, 1, at), None);
        assert!(drafts.get(&last, 1, at).is_some());
    }
}
