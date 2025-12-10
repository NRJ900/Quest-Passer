use tray_icon::TrayIcon;
use tray_icon::TrayIconBuilder;
use tray_icon::Icon;
use tray_icon::menu::{Menu};

pub fn create_tray_icon(tray_menu: Menu, title: &str) -> TrayIcon {

    // Load icon from included bytes to avoid resource dependency issues
    let icon_data = include_bytes!("../icon.ico");
    let image = image::load_from_memory(icon_data).expect("Failed to load embedded icon");
    let rgba = image.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let icon = Icon::from_rgba(rgba.into_raw(), width, height).expect("Failed to create tray icon from bytes");

    TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(title)
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon")
}
