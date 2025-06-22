use eframe::egui::IconData;
use tray_icon::Icon;

/// Represents an image used for tray and egui icons.
pub struct IconImage {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

impl IconImage {
    /// Converts the icon image into a tray icon.
    pub fn to_tray_icon(&self) -> Icon {
        // Create tray icon; cloning rgba is necessary as Icon consumes it.
        Icon::from_rgba(self.rgba.clone(), self.width, self.height)
            .expect("Failed to create tray icon")
    }

    /// Converts the icon image into an egui icon.
    pub fn to_egui_icon(&self) -> IconData {
        IconData {
            rgba: self.rgba.clone(),
            width: self.width,
            height: self.height,
        }
    }
}

pub fn load_icon_from_memory(data: &[u8]) -> Result<IconImage, image::ImageError> {
    let image = image::load_from_memory(data)?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Ok(IconImage { rgba, width, height })
}
