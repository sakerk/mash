#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod checksum_file;
mod cli;
mod diff;
mod gui;
mod hashing;
mod manifest_reader;
mod mhl;
mod util;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        // Re-attach to the parent console so CLI output is visible
        // (windows_subsystem = "windows" detaches the console in release builds)
        #[cfg(target_os = "windows")]
        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn AttachConsole(dw_process_id: u32) -> i32;
            }
            const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;
            AttachConsole(ATTACH_PARENT_PROCESS);
        }

        cli::run();
    } else {
        app::run_gui();
    }
}
