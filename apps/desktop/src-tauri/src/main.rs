// Prevent additional console window from showing on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    contextflow_desktop_lib::run();
}
