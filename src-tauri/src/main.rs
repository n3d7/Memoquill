#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
mod linux_startup;

fn main() {
    #[cfg(target_os = "linux")]
    if let Err(error) = linux_startup::bootstrap() {
        eprintln!("{error}");
        std::process::exit(1);
    }
    memoquill::run();
}
