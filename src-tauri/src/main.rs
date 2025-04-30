// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use web_app_runner_lib;
fn main() {
    web_app_runner_lib::run()
}
