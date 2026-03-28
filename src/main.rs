#![deny(clippy::all, clippy::pedantic, warnings)]
// FreeSynergy Browser — iced-based standalone launcher.

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "iced-gui")]
fn main() -> fs_gui_engine_iced::iced::Result {
    use fs_browser::app::{BrowserApp, Message};
    fs_gui_engine_iced::IcedEngine::run::<BrowserApp, Message, _, _>(
        "FreeSynergy Browser",
        BrowserApp::update,
        BrowserApp::view,
    )
}

#[cfg(not(feature = "iced-gui"))]
fn main() {
    eprintln!("No GUI feature enabled. Build with --features iced-gui");
}
