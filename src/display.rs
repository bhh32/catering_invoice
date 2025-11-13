use std::fs;
use std::path::Path;

/// Display size information
#[derive(Debug, Clone, Copy)]
pub struct DisplaySize {
    pub width: f32,
    pub height: f32,
}

impl DisplaySize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Convert to cosmic::iced::Size
    pub fn to_size(self) -> cosmic::iced::Size {
        cosmic::iced::Size::new(self.width / 2.0, self.height / 2.0)
    }

    /// Check if the display size is reasonable (not too small)
    pub fn is_valid(self) -> bool {
        self.width >= 1080.0 && self.height >= 720.0
    }
}

/// Parse virtual_size format: "1920,1080"
fn parse_virtual_size(contents: &str) -> Option<DisplaySize> {
    let parts: Vec<&str> = contents.trim().split(",").collect();

    if parts.len() == 2 {
        if let (Ok(width), Ok(height)) = (
            parts[0].trim().parse::<f32>(),
            parts[1].trim().parse::<f32>(),
        ) {
            return Some(DisplaySize::new(width, height));
        }
    }

    None
}

/// Try to get screen size from a specific framebuffer device
fn try_framebuffer(fb: u8) -> Option<DisplaySize> {
    let fb_path = format!("/sys/class/graphics/fb{fb}");

    if !Path::new(&fb_path).exists() {
        return None;
    }

    // Try virtual_size file first (format: "width,height")
    let virt_size_path = format!("{fb_path}/virtual_size");
    if let Ok(contents) = fs::read_to_string(&virt_size_path) {
        if let Some(size) = parse_virtual_size(&contents) {
            if size.is_valid() {
                return Some(size);
            }
        }
    }

    // Fallback: try separate width/height files
    let width_path = format!("{fb_path}/virtual_x");
    let height_path = format!("{fb_path}/virtual_y");

    if let (Ok(width_str), Ok(height_str)) = (
        fs::read_to_string(&width_path),
        fs::read_to_string(&height_path),
    ) {
        if let (Ok(width), Ok(height)) = (
            width_str.trim().parse::<f32>(),
            height_str.trim().parse::<f32>(),
        ) {
            let size = DisplaySize::new(width, height);
            if size.is_valid() {
                return Some(size);
            }
        }
    }

    None
}

/// Get the primary display size from system info
pub fn get_primary_display_size() -> Option<DisplaySize> {
    None
}
