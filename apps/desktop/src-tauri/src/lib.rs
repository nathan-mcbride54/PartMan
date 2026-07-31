//! Read-only native host for the PartMan desktop shell.
//!
//! This increment registers no commands, plugins, storage access, or
//! privileged behavior. The webview is a presentation boundary only.

/// Start the desktop application.
///
/// # Panics
///
/// Panics if the operating system cannot initialize or run the Tauri event
/// loop. No storage operation can have started because this shell exposes no
/// native commands.
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("the PartMan desktop event loop could not start");
}
