// Browser data model: tabs, bookmarks, history entries.

use serde::{Deserialize, Serialize};

/// A single browser tab.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowserTab {
    pub id: u32,
    pub title: String,
    pub url: String,
    /// `true` while the page is loading.
    pub loading: bool,
}

impl BrowserTab {
    #[must_use]
    pub fn new(id: u32) -> Self {
        Self {
            id,
            title: "New Tab".to_string(),
            url: String::new(),
            loading: false,
        }
    }

    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        self.url.clone_from(&url);
        self.title = url;
        self
    }

    /// Navigate to `url`, updating title from the URL string.
    pub fn navigate(&mut self, url: String) {
        self.title = url.chars().take(32).collect();
        self.url = url;
    }

    /// Reset to an empty new-tab state.
    pub fn reset(&mut self) {
        self.url = String::new();
        self.title = "New Tab".to_string();
    }
}

/// A bookmarked URL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub created_at: String,
}

/// A history entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub visited_at: String,
}

/// A download tracked by the browser.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadEntry {
    pub id: i64,
    pub filename: String,
    pub url: String,
    /// S3 destination path (e.g. `/shared/downloads/file.zip`).
    pub s3_path: String,
    pub status: DownloadStatus,
    pub started_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Saving,
    Done,
    Error(String),
}

impl DownloadStatus {
    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Self::Pending => "Pending",
            Self::Saving => "Saving…",
            Self::Done => "Done",
            Self::Error(_) => "Error",
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tab_is_empty() {
        let tab = BrowserTab::new(1);
        assert_eq!(tab.id, 1);
        assert!(tab.url.is_empty());
        assert_eq!(tab.title, "New Tab");
        assert!(!tab.loading);
    }

    #[test]
    fn with_url_sets_title_and_url() {
        let tab = BrowserTab::new(1).with_url("https://example.com");
        assert_eq!(tab.url, "https://example.com");
        assert_eq!(tab.title, "https://example.com");
    }

    #[test]
    fn navigate_truncates_title_to_32_chars() {
        let mut tab = BrowserTab::new(1);
        let long_url = "https://example.com/very/long/path/that/exceeds/32/chars";
        tab.navigate(long_url.to_string());
        assert_eq!(tab.url, long_url);
        assert!(tab.title.chars().count() <= 32);
    }

    #[test]
    fn reset_clears_tab() {
        let mut tab = BrowserTab::new(1).with_url("https://example.com");
        tab.reset();
        assert!(tab.url.is_empty());
        assert_eq!(tab.title, "New Tab");
    }

    #[test]
    fn download_status_labels() {
        assert_eq!(DownloadStatus::Pending.label(), "Pending");
        assert_eq!(DownloadStatus::Saving.label(), "Saving\u{2026}");
        assert_eq!(DownloadStatus::Done.label(), "Done");
        assert_eq!(DownloadStatus::Error("oops".into()).label(), "Error");
    }
}
