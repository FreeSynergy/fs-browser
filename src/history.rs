// Browser history helpers.

use crate::model::HistoryEntry;

/// Remove all history entries.
pub fn clear(history: &mut Vec<HistoryEntry>) {
    history.clear();
}

/// Return the last N entries (most recent first).
pub fn recent(history: &[HistoryEntry], limit: usize) -> Vec<&HistoryEntry> {
    history.iter().rev().take(limit).collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bookmarks::BookmarkManager;

    fn entry(url: &str) -> HistoryEntry {
        BookmarkManager::record_visit(url, url)
    }

    #[test]
    fn clear_empties_history() {
        let mut h = vec![entry("https://a.com"), entry("https://b.com")];
        clear(&mut h);
        assert!(h.is_empty());
    }

    #[test]
    fn recent_returns_most_recent_first() {
        let h = vec![entry("first"), entry("second"), entry("third")];
        let r = recent(&h, 10);
        assert_eq!(r[0].url, "third");
        assert_eq!(r[1].url, "second");
        assert_eq!(r[2].url, "first");
    }

    #[test]
    fn recent_respects_limit() {
        let h = vec![entry("a"), entry("b"), entry("c"), entry("d")];
        let r = recent(&h, 2);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].url, "d");
        assert_eq!(r[1].url, "c");
    }

    #[test]
    fn recent_on_empty_returns_empty() {
        let h: Vec<HistoryEntry> = vec![];
        assert!(recent(&h, 10).is_empty());
    }
}
