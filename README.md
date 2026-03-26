# fs-browser

Embedded web browser for the FreeSynergy desktop — tabs, bookmarks, history,
and S3 downloads. Runs standalone or embedded in the Desktop shell.

## Build

```sh
cargo build --release
cargo test
```

## Architecture

Follows the Provider Pattern (OOP, Dioxus):

- `BrowserApp` — root Dioxus component
- `BrowserConfig` / `SearchEngineRegistry` — configuration and search engines
- `BookmarkManager` — all bookmark and history CRUD
- `BrowserUrlRequest` — context signal for external callers to open URLs

## Config

`~/.config/fsn/browser.toml`
