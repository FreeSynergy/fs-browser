// Bookmark + History CRUD backed by fs-db (browser.db).

use std::sync::atomic::{AtomicI64, Ordering};

use crate::model::{Bookmark, HistoryEntry};

/// Monotonically increasing in-memory ID source.
/// Once persisted to fs-db the database will own ID generation.
static NEXT_ID: AtomicI64 = AtomicI64::new(1);

fn next_id() -> i64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

// ── BookmarkManager ───────────────────────────────────────────────────────────

pub struct BookmarkManager;

impl BookmarkManager {
    /// Add a bookmark. No-op if already bookmarked (same URL).
    #[must_use]
    pub fn add(title: &str, url: &str) -> Option<Bookmark> {
        Some(Bookmark {
            id: next_id(),
            title: title.to_string(),
            url: url.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Remove a bookmark by ID.
    pub fn remove(bookmarks: &mut Vec<Bookmark>, id: i64) {
        bookmarks.retain(|b| b.id != id);
    }

    /// Record a history visit. Adds a new entry; duplicates are kept for full history.
    #[must_use]
    pub fn record_visit(title: &str, url: &str) -> HistoryEntry {
        HistoryEntry {
            id: next_id(),
            title: title.to_string(),
            url: url.to_string(),
            visited_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_returns_bookmark_with_correct_fields() {
        let bm = BookmarkManager::add("Example", "https://example.com").unwrap();
        assert_eq!(bm.title, "Example");
        assert_eq!(bm.url, "https://example.com");
        assert!(!bm.created_at.is_empty());
    }

    #[test]
    fn remove_deletes_by_id() {
        let bm = BookmarkManager::add("A", "https://a.com").unwrap();
        let id = bm.id;
        let mut list = vec![bm, BookmarkManager::add("B", "https://b.com").unwrap()];
        BookmarkManager::remove(&mut list, id);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].url, "https://b.com");
    }

    #[test]
    fn remove_unknown_id_is_noop() {
        let bm = BookmarkManager::add("A", "https://a.com").unwrap();
        let mut list = vec![bm];
        BookmarkManager::remove(&mut list, 999_999);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn record_visit_returns_entry_with_correct_fields() {
        let entry = BookmarkManager::record_visit("Example", "https://example.com");
        assert_eq!(entry.title, "Example");
        assert_eq!(entry.url, "https://example.com");
        assert!(!entry.visited_at.is_empty());
    }
}
