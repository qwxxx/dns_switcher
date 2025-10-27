use anyhow::anyhow;
use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
use image::{ImageFormat, load_from_memory_with_format};
use std::{fs, sync::Arc};

use crate::types::DnsConfig;

pub fn hide_app_from_dock() {
    unsafe {
        let app = NSApp();
        app.setActivationPolicy_(
            NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
        );
    }
}
pub fn load_custom_dns_from_config() -> Option<DnsConfig> {
    if let Ok(content) = fs::read_to_string(
        std::env::current_exe()
            .ok()?
            .parent()?
            .parent()?
            .join("Resources")
            .join("dns_config.yaml"),
    ) {
        match serde_yaml::from_str(&content) {
            Ok(config) => return Some(config),
            Err(e) => eprintln!("Failed to parse config file: {}", e),
        }
    } else {
        eprintln!(
            "Failed to read config file in directory {}",
            std::env::current_dir().unwrap().display()
        );
    }

    None
}

pub fn load_icon_from_assets(
    asset_source: &Arc<dyn gpui::AssetSource>,
    path: &str,
) -> anyhow::Result<tray_icon::Icon> {
    let data = asset_source
        .load(path)?
        .ok_or(anyhow!("Failed to load icon"))?;
    let img = load_from_memory_with_format(&data, ImageFormat::Png)?;
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();

    if width > 256 || height > 256 {
        return Err(anyhow!("Icon too large"));
    }

    tray_icon::Icon::from_rgba(rgba.into_raw(), width, height)
        .map_err(|e| anyhow!("Failed to create icon: {}", e))
}
