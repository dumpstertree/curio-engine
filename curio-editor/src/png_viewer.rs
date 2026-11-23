//! Real PNG preview — decodes the image and hands it to egui's own texture
//! manager (`ctx.load_texture`). Much simpler than the GLB case: no custom
//! wgpu pipeline needed, egui already has a perfectly good 2D image path.

use eframe::egui;

pub struct PngPreview {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub texture: egui::TextureHandle,
}

impl PngPreview {
    /// Decodes `bytes` (PNG or JPEG — anything the `image` crate's enabled
    /// formats cover) and uploads it as an egui texture. `ctx` is needed
    /// because `load_texture` goes through egui's texture manager, not a
    /// raw wgpu call — unlike the GLB preview, this never touches
    /// `render_state` directly.
    pub fn load(ctx: &egui::Context, path: String, bytes: &[u8]) -> Result<Self, String> {
        let image = image::load_from_memory(bytes).map_err(|e| format!("Failed to decode image: {e}"))?;
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();

        let color_image = egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], rgba.as_raw());
        let texture = ctx.load_texture(format!("asset_png:{path}"), color_image, egui::TextureOptions::LINEAR);

        Ok(Self { path, width, height, texture })
    }
}
