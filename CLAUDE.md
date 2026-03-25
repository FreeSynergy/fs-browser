# CLAUDE.md – fs-browser

## What is this?

FreeSynergy Browser — embedded web browser with tabs, bookmarks, history, and S3 downloads.
Runs standalone or embedded inside the FreeSynergy Desktop shell (`fs-gui-workspace`).

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md
- After every feature: commit directly

## Architecture

Follows the Provider Pattern (OOP, Dioxus):
- `BrowserApp` is the root component
- `BrowserConfig` / `SearchEngineRegistry` handle configuration
- `BookmarkManager` owns all bookmark/history CRUD
- `BrowserUrlRequest` context allows external callers (e.g. Conductor) to open URLs

## Dependencies

- **fs-libs** (`../fs-libs/`) — `fs-components`, `fs-i18n`
- **fs-desktop** (`../fs-desktop/vendor/dioxus-desktop`) — patched Dioxus desktop

## CSS Variables Prefix

Always `--fs-` (e.g., `--fs-color-primary`, `--fs-font-family`).

## Config path

`~/.config/fsn/browser.toml`
