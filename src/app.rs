// fs-browser/src/app.rs — iced-based browser application.
//
// Uses fs-gui-engine-iced (and thus fs-render) as the GUI layer.
// Imports fs-web-engine for WebView/WebEngine abstractions.
// No Dioxus dependency.

use fs_gui_engine_iced::iced::{
    self,
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Alignment, Element, Length, Task,
};
use fs_web_engine::{
    stub::StubWebEngine,
    view::{WebView, WebViewConfig},
    WebEngine,
};

use crate::bookmarks::BookmarkManager;
use crate::model::{Bookmark, BrowserTab, DownloadEntry, HistoryEntry};
use crate::search_engine::BrowserConfig;

// ── BrowserPanel ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BrowserPanel {
    #[default]
    Browse,
    Bookmarks,
    History,
    Downloads,
}

// ── Message ───────────────────────────────────────────────────────────────────

/// Browser application messages.
#[derive(Debug, Clone)]
pub enum Message {
    AddressChanged(String),
    Navigate,
    NavigateTo(String),
    Reload,
    TabSelected(u32),
    TabClosed(u32),
    NewTab,
    PanelToggled(BrowserPanel),
    BookmarkCurrent,
    BookmarkRemoved(i64),
    HistoryCleared,
    Noop,
}

// ── BrowserApp ────────────────────────────────────────────────────────────────

/// `FreeSynergy` Browser application state (iced-based).
pub struct BrowserApp {
    tabs: Vec<BrowserTab>,
    active_tab: u32,
    next_tab_id: u32,
    address_input: String,
    panel: BrowserPanel,
    bookmarks: Vec<Bookmark>,
    history: Vec<HistoryEntry>,
    downloads: Vec<DownloadEntry>,
    config: BrowserConfig,
    status_msg: Option<String>,
    // Web engine — stub by default; replace with ServoWebEngine when G2.6 is done.
    engine: Box<dyn WebEngine>,
    web_view: Box<dyn WebView>,
}

impl std::fmt::Debug for BrowserApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserApp")
            .field("tabs", &self.tabs)
            .field("active_tab", &self.active_tab)
            .field("next_tab_id", &self.next_tab_id)
            .field("address_input", &self.address_input)
            .field("panel", &self.panel)
            .field("bookmarks", &self.bookmarks)
            .field("history", &self.history)
            .field("downloads", &self.downloads)
            .field("config", &self.config)
            .field("status_msg", &self.status_msg)
            .finish_non_exhaustive()
    }
}

impl Default for BrowserApp {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserApp {
    /// Create a new browser with one empty tab.
    pub fn new() -> Self {
        let config = BrowserConfig::load();
        let engine: Box<dyn WebEngine> = Box::new(StubWebEngine::new());
        let web_view = engine.create_view(WebViewConfig::new("tab-1"));
        Self {
            tabs: vec![BrowserTab::new(1)],
            active_tab: 1,
            next_tab_id: 2,
            address_input: String::new(),
            panel: BrowserPanel::Browse,
            bookmarks: Vec::new(),
            history: Vec::new(),
            downloads: Vec::new(),
            config,
            status_msg: None,
            engine,
            web_view,
        }
    }

    // ── Update ────────────────────────────────────────────────────────────────

    /// Handle a browser message and return the next task.
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::AddressChanged(s) => {
                self.address_input = s;
            }
            Message::Navigate => {
                let url = normalize_url(&self.address_input, &self.config);
                self.navigate_to(url);
            }
            Message::NavigateTo(url) => {
                self.navigate_to(url.clone());
                self.address_input = url;
            }
            Message::Reload => {
                let url = self.current_url().to_string();
                if !url.is_empty() {
                    self.engine.reload(self.web_view.view_id());
                    self.navigate_to(url);
                }
            }
            Message::TabSelected(id) => {
                self.active_tab = id;
                let url = self
                    .tabs
                    .iter()
                    .find(|t| t.id == id)
                    .map(|t| t.url.clone())
                    .unwrap_or_default();
                self.address_input = url;
            }
            Message::TabClosed(id) => {
                self.close_tab(id);
            }
            Message::NewTab => {
                let id = self.next_tab_id;
                self.next_tab_id += 1;
                self.tabs.push(BrowserTab::new(id));
                self.active_tab = id;
                self.address_input = String::new();
                // Create a new WebView for the tab.
                let view_id = format!("tab-{id}");
                self.web_view = self.engine.create_view(WebViewConfig::new(view_id));
            }
            Message::PanelToggled(p) => {
                self.panel = if self.panel == p {
                    BrowserPanel::Browse
                } else {
                    p
                };
            }
            Message::BookmarkCurrent => {
                let url = self.current_url().to_string();
                if !url.is_empty() {
                    if let Some(bm) = BookmarkManager::add(&url, &url) {
                        self.bookmarks.push(bm);
                        self.status_msg = Some("Bookmark added".to_string());
                    }
                }
            }
            Message::BookmarkRemoved(id) => {
                BookmarkManager::remove(&mut self.bookmarks, id);
            }
            Message::HistoryCleared => {
                crate::history::clear(&mut self.history);
            }
            Message::Noop => {}
        }
        Task::none()
    }

    // ── View ──────────────────────────────────────────────────────────────────

    /// Render the browser UI.
    pub fn view(&self) -> Element<'_, Message> {
        let show_panel = self.panel != BrowserPanel::Browse;

        column![
            Self::view_titlebar(),
            self.view_toolbar(),
            self.view_tabbar(),
            self.view_status(),
            row![
                self.view_viewport(show_panel),
                if show_panel {
                    self.view_panel()
                } else {
                    Space::with_width(0).into()
                },
            ]
            .height(Length::Fill),
        ]
        .into()
    }

    fn view_titlebar() -> Element<'static, Message> {
        container(text("FreeSynergy Browser").size(15))
            .width(Length::Fill)
            .padding(8)
            .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        let nav_btn = |label: &'static str, msg: Message| {
            button(text(label).size(16)).on_press(msg).padding([4, 8])
        };

        row![
            nav_btn("‹", Message::Noop),
            nav_btn("›", Message::Noop),
            nav_btn("↺", Message::Reload),
            text_input("Enter URL or search…", &self.address_input)
                .on_input(Message::AddressChanged)
                .on_submit(Message::Navigate)
                .padding(6)
                .width(Length::Fill),
            nav_btn("☆", Message::BookmarkCurrent),
            button(text("🔖").size(14))
                .on_press(Message::PanelToggled(BrowserPanel::Bookmarks))
                .padding([4, 8]),
            button(text("⏱").size(14))
                .on_press(Message::PanelToggled(BrowserPanel::History))
                .padding([4, 8]),
            button(text("⬇").size(14))
                .on_press(Message::PanelToggled(BrowserPanel::Downloads))
                .padding([4, 8]),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .padding(6)
        .into()
    }

    fn view_tabbar(&self) -> Element<'_, Message> {
        let tabs: Vec<Element<Message>> = self
            .tabs
            .iter()
            .map(|tab| {
                let is_active = tab.id == self.active_tab;
                let tab_id = tab.id;
                let label = tab.title.chars().take(20).collect::<String>();
                row![
                    button(text(label).size(12))
                        .on_press(Message::TabSelected(tab_id))
                        .padding([4, 10])
                        .style(if is_active {
                            iced::widget::button::primary
                        } else {
                            iced::widget::button::secondary
                        }),
                    button(text("✕").size(10))
                        .on_press(Message::TabClosed(tab_id))
                        .padding([4, 6]),
                ]
                .spacing(0)
                .into()
            })
            .collect();

        let mut tab_row = row(tabs).spacing(2);
        tab_row = tab_row.push(
            button(text("+").size(16))
                .on_press(Message::NewTab)
                .padding([3, 10]),
        );

        container(tab_row.padding(4)).width(Length::Fill).into()
    }

    fn view_status(&self) -> Element<'_, Message> {
        if let Some(msg) = &self.status_msg {
            container(text(msg.as_str()).size(11))
                .width(Length::Fill)
                .padding([2, 16])
                .into()
        } else {
            Space::with_height(0).into()
        }
    }

    fn view_viewport(&self, _show_panel: bool) -> Element<'_, Message> {
        let url = self.current_url();
        let content: Element<Message> = if url.is_empty() {
            column![
                text("🌐").size(48),
                text("Enter a URL or search term above").size(14),
            ]
            .spacing(12)
            .align_x(Alignment::Center)
            .into()
        } else {
            // Placeholder — real rendering via fs-web-engine-servo (G2.6).
            column![
                text(format!("Loading: {url}")).size(13),
                text("(Web engine not yet connected — G2.6)").size(11),
            ]
            .spacing(8)
            .padding(16)
            .into()
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(8)
            .into()
    }

    fn view_panel(&self) -> Element<'_, Message> {
        let content: Element<Message> = match self.panel {
            BrowserPanel::Bookmarks => self.view_bookmarks_panel(),
            BrowserPanel::History => self.view_history_panel(),
            BrowserPanel::Downloads => self.view_downloads_panel(),
            BrowserPanel::Browse => Space::with_width(0).into(),
        };
        container(scrollable(content))
            .width(280)
            .height(Length::Fill)
            .into()
    }

    fn view_bookmarks_panel(&self) -> Element<'_, Message> {
        let items: Vec<Element<Message>> = self
            .bookmarks
            .iter()
            .map(|bm| {
                let url = bm.url.clone();
                let id = bm.id;
                row![
                    button(text(bm.title.chars().take(30).collect::<String>()).size(12))
                        .on_press(Message::NavigateTo(url))
                        .style(iced::widget::button::text)
                        .width(Length::Fill),
                    button(text("✕").size(10))
                        .on_press(Message::BookmarkRemoved(id))
                        .padding([2, 6]),
                ]
                .spacing(4)
                .into()
            })
            .collect();

        let empty = self.bookmarks.is_empty();
        let mut col = column![text("Bookmarks").size(14)].spacing(4).padding(12);
        if empty {
            col = col.push(text("No bookmarks yet").size(12));
        } else {
            for item in items {
                col = col.push(item);
            }
        }
        col.into()
    }

    fn view_history_panel(&self) -> Element<'_, Message> {
        let recent = crate::history::recent(&self.history, 50);
        let items: Vec<Element<Message>> = recent
            .into_iter()
            .map(|entry| {
                let url = entry.url.clone();
                button(text(entry.title.chars().take(30).collect::<String>()).size(12))
                    .on_press(Message::NavigateTo(url))
                    .style(iced::widget::button::text)
                    .width(Length::Fill)
                    .into()
            })
            .collect();

        let mut col = column![row![
            text("History").size(14).width(Length::Fill),
            button(text("Clear").size(11))
                .on_press(Message::HistoryCleared)
                .padding([2, 8]),
        ],]
        .spacing(4)
        .padding(12);

        if items.is_empty() {
            col = col.push(text("No history yet").size(12));
        } else {
            for item in items {
                col = col.push(item);
            }
        }
        col.into()
    }

    fn view_downloads_panel(&self) -> Element<'_, Message> {
        let items: Vec<Element<Message>> = self
            .downloads
            .iter()
            .map(|dl| {
                row![
                    text(dl.filename.chars().take(24).collect::<String>())
                        .size(12)
                        .width(Length::Fill),
                    text(dl.status.label()).size(11),
                ]
                .spacing(8)
                .into()
            })
            .collect();

        let mut col = column![text("Downloads").size(14)].spacing(4).padding(12);
        if items.is_empty() {
            col = col.push(text("No downloads").size(12));
        } else {
            for item in items {
                col = col.push(item);
            }
        }
        col.into()
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn current_url(&self) -> &str {
        self.tabs
            .iter()
            .find(|t| t.id == self.active_tab)
            .map_or("", |t| t.url.as_str())
    }

    fn navigate_to(&mut self, url: String) {
        self.address_input.clone_from(&url);
        self.web_view.navigate(&url);
        self.history.push(BookmarkManager::record_visit(&url, &url));
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == self.active_tab) {
            tab.navigate(url);
        }
    }

    fn close_tab(&mut self, id: u32) {
        if self.tabs.len() <= 1 {
            if let Some(t) = self.tabs.first_mut() {
                t.reset();
                self.web_view = self.engine.create_view(WebViewConfig::new("tab-1"));
            }
            return;
        }
        let idx = self.tabs.iter().position(|t| t.id == id).unwrap_or(0);
        self.tabs.remove(idx);
        if self.active_tab == id {
            let new_idx = idx.saturating_sub(1).min(self.tabs.len() - 1);
            if let Some(t) = self.tabs.get(new_idx) {
                self.active_tab = t.id;
                self.address_input = t.url.clone();
            }
        }
    }
}

// ── URL helpers ───────────────────────────────────────────────────────────────

/// Normalize address-bar input to a URL.
fn normalize_url(input: &str, config: &BrowserConfig) -> String {
    let t = input.trim();
    if t.is_empty() {
        return String::new();
    }
    if t.starts_with("http://") || t.starts_with("https://") || t.starts_with("file://") {
        t.to_string()
    } else if t.contains('.') && !t.contains(' ') {
        format!("https://{t}")
    } else {
        config.build_search_url(t)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_app_has_one_tab() {
        let app = BrowserApp::new();
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab, 1);
    }

    #[test]
    fn navigate_updates_tab_url() {
        let mut app = BrowserApp::new();
        app.address_input = "https://freesynergy.net".into();
        let _ = app.update(Message::Navigate);
        assert_eq!(app.current_url(), "https://freesynergy.net");
    }

    #[test]
    fn new_tab_increments_count() {
        let mut app = BrowserApp::new();
        let _ = app.update(Message::NewTab);
        assert_eq!(app.tabs.len(), 2);
    }

    #[test]
    fn close_last_tab_resets_instead_of_removing() {
        let mut app = BrowserApp::new();
        let _ = app.update(Message::Navigate);
        let _ = app.update(Message::TabClosed(1));
        assert_eq!(app.tabs.len(), 1);
        assert!(app.tabs[0].url.is_empty());
    }

    #[test]
    fn close_tab_switches_to_adjacent() {
        let mut app = BrowserApp::new();
        let _ = app.update(Message::NewTab);
        assert_eq!(app.tabs.len(), 2);
        let second_id = app.tabs[1].id;
        let _ = app.update(Message::TabClosed(second_id));
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab, 1);
    }

    #[test]
    fn bookmark_added_to_list() {
        let mut app = BrowserApp::new();
        let _ = app.update(Message::NavigateTo("https://example.com".into()));
        let _ = app.update(Message::BookmarkCurrent);
        assert_eq!(app.bookmarks.len(), 1);
    }

    #[test]
    fn history_recorded_on_navigate() {
        let mut app = BrowserApp::new();
        let _ = app.update(Message::NavigateTo("https://example.com".into()));
        assert_eq!(app.history.len(), 1);
    }

    #[test]
    fn history_cleared() {
        let mut app = BrowserApp::new();
        let _ = app.update(Message::NavigateTo("https://example.com".into()));
        let _ = app.update(Message::HistoryCleared);
        assert!(app.history.is_empty());
    }

    #[test]
    fn normalize_url_prepends_https_for_domain() {
        let cfg = BrowserConfig::default();
        assert_eq!(normalize_url("example.com", &cfg), "https://example.com");
    }

    #[test]
    fn normalize_url_passthrough_for_full_url() {
        let cfg = BrowserConfig::default();
        assert_eq!(
            normalize_url("https://example.com/path", &cfg),
            "https://example.com/path"
        );
    }

    #[test]
    fn normalize_url_builds_search_url_for_terms() {
        let cfg = BrowserConfig::default();
        let url = normalize_url("freeSynergy platform", &cfg);
        assert!(url.contains("freeSynergy"), "url={url}");
    }
}
