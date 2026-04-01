// config.rs — BrowserConfig and SearchEngineRegistry.

use serde::{Deserialize, Serialize};

// ── SearchEngine ──────────────────────────────────────────────────────────────

/// A search engine entry available in the browser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchEngine {
    /// Machine-readable identifier (e.g. `"duckduckgo"`).
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// URL template — `{query}` is replaced with the encoded search term.
    pub url_template: String,
}

// ── SearchEngineRegistry ──────────────────────────────────────────────────────

/// Built-in registry of available search engines.
pub struct SearchEngineRegistry;

impl SearchEngineRegistry {
    /// Returns the full list of built-in search engines.
    #[must_use]
    pub fn all() -> &'static [SearchEngine] {
        &ENGINES
    }

    /// Look up a search engine by its `id`.
    #[must_use]
    pub fn find(id: &str) -> Option<&'static SearchEngine> {
        ENGINES.iter().find(|e| e.id == id)
    }
}

static ENGINES: std::sync::LazyLock<Vec<SearchEngine>> = std::sync::LazyLock::new(|| {
    vec![
        SearchEngine {
            id: "duckduckgo".into(),
            name: "DuckDuckGo".into(),
            url_template: "https://duckduckgo.com/?q={query}".into(),
        },
        SearchEngine {
            id: "startpage".into(),
            name: "Startpage".into(),
            url_template: "https://www.startpage.com/search?q={query}".into(),
        },
        SearchEngine {
            id: "brave".into(),
            name: "Brave Search".into(),
            url_template: "https://search.brave.com/search?q={query}".into(),
        },
        SearchEngine {
            id: "ecosia".into(),
            name: "Ecosia".into(),
            url_template: "https://www.ecosia.org/search?q={query}".into(),
        },
    ]
});

// ── BrowserConfig ─────────────────────────────────────────────────────────────

/// Persistent browser configuration (search engine, homepage, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// ID of the active search engine (matches [`SearchEngine::id`]).
    pub search_engine: String,
    /// Optional default homepage URL.
    pub homepage: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            search_engine: "duckduckgo".into(),
            homepage: None,
        }
    }
}

impl BrowserConfig {
    /// Load config from the default location, falling back to defaults.
    #[must_use]
    pub fn load() -> Self {
        Self::default()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_search_engine_exists_in_registry() {
        let cfg = BrowserConfig::default();
        assert!(SearchEngineRegistry::find(&cfg.search_engine).is_some());
    }

    #[test]
    fn registry_is_non_empty() {
        assert!(!SearchEngineRegistry::all().is_empty());
    }

    #[test]
    fn find_unknown_returns_none() {
        assert!(SearchEngineRegistry::find("unknown").is_none());
    }
}
