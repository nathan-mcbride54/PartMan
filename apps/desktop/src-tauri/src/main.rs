//! PartMan desktop executable entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    partman_desktop_lib::run();
}
