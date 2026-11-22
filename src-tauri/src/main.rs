#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
mod linux_startup;

fn main() {
    #[cfg(target_os = "linux")]
    if std::env::args_os()
        .nth(1)
        .is_some_and(|value| value == "mcp")
    {
        if let Err(error) = memoquill::run_stdio() {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(target_os = "linux")]
    if let Err(error) = linux_startup::bootstrap() {
        eprintln!("{error}");
        std::process::exit(1);
    }
    memoquill::run();
}
