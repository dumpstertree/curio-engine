use egui_wgpu::wgpu::TextureFormat;
use image::{ImageBuffer, Rgba};
use std::io::Cursor;

pub fn encode_frame(
    pixels: &[u8],
    width: u32,
    height: u32,
    format: TextureFormat,
) -> Option<Vec<u8>> {
    let rgba: Vec<u8> = match format {
        TextureFormat::Bgra8UnormSrgb | TextureFormat::Bgra8Unorm => pixels
            .chunks(4)
            .flat_map(|p| [p[2], p[1], p[0], p[3]])
            .collect(),

        _ => pixels.to_vec(),
    };

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba)?;

    let mut buffer = Cursor::new(Vec::new());

    image.write_to(&mut buffer, image::ImageFormat::Png).ok()?;

    Some(buffer.into_inner())
}

pub fn base64_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};

    STANDARD.encode(data)
}
