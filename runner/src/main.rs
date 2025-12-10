
mod tray;

#[cfg(target_os = "windows")]
mod win32;
#[cfg(target_os = "windows")]
use win32 as os;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as os;

fn main() {
    os::run();
}
