
#![cfg(target_os = "macos")]

use std::env;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct Config {
    pub title: String,
    pub start_minimized: bool,
    pub icon_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            title: "Quest Passer".to_string(),
            start_minimized: false,
            icon_url: None,
        }
    }
}

pub fn parse_args() -> Config {
    // Basic argument parsing, similar to Windows but potentially simpler for now
    let args: Vec<String> = env::args().collect();
    let mut config = Config::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--title" => {
                if i + 1 < args.len() {
                    config.title = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    config
}

pub fn run() {
    println!("Starting MacOS Runner Stub...");
    let config = parse_args();
    println!("Title: {}", config.title);

    // On Mac, we might just sleep to simulate the process running.
    // Creating a window without a full framework (like Cocoa) in raw Rust is complex.
    // Ideally, we'd use something like `winit` here in the future.
    // For "Quest Completion", just the process existing is often enough.
    
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
