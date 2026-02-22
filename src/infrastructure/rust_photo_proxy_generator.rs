use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use image::imageops::FilterType;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};

use crate::application::proxy_generator::{
    AudioProxyRequest, PhotoProxyFormat, PhotoProxyRequest, ProxyGenerationError, ProxyGenerator,
    VideoProxyRequest,
};

pub trait RawPhotoDecoder {
    fn decode_photo(&self, input_path: &str) -> Result<DynamicImage, String>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RawloaderPhotoDecoder;

impl RawPhotoDecoder for RawloaderPhotoDecoder {
    fn decode_photo(&self, input_path: &str) -> Result<DynamicImage, String> {
        let decoded = rawloader::decode_file(input_path).map_err(|error| error.to_string())?;
        let width = u32::try_from(decoded.width).map_err(|_| "raw width overflow".to_string())?;
        let height =
            u32::try_from(decoded.height).map_err(|_| "raw height overflow".to_string())?;
        let rgb = match decoded.data {
            rawloader::RawImageData::Integer(values) => {
                to_rgb8_from_integer(&values, decoded.cpp, width, height)?
            }
            rawloader::RawImageData::Float(values) => {
                to_rgb8_from_float(&values, decoded.cpp, width, height)?
            }
        };
        let image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width, height, rgb)
            .ok_or_else(|| "unable to create RGB image from raw decode".to_string())?;
        Ok(DynamicImage::ImageRgb8(image))
    }
}

#[derive(Debug, Clone)]
pub struct RustPhotoProxyGenerator<D: RawPhotoDecoder = RawloaderPhotoDecoder> {
    raw_decoder: D,
}

impl Default for RustPhotoProxyGenerator<RawloaderPhotoDecoder> {
    fn default() -> Self {
        Self::new(RawloaderPhotoDecoder)
    }
}

impl<D: RawPhotoDecoder> RustPhotoProxyGenerator<D> {
    pub fn new(raw_decoder: D) -> Self {
        Self { raw_decoder }
    }

    pub fn raw_decoder(&self) -> &D {
        &self.raw_decoder
    }
}

impl<D: RawPhotoDecoder> ProxyGenerator for RustPhotoProxyGenerator<D> {
    fn generate_video_proxy(
        &self,
        _request: &VideoProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        Err(ProxyGenerationError::InvalidRequest(
            "video proxy generation is handled by FfmpegProxyGenerator".to_string(),
        ))
    }

    fn generate_audio_proxy(
        &self,
        _request: &AudioProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        Err(ProxyGenerationError::InvalidRequest(
            "audio proxy generation is handled by FfmpegProxyGenerator".to_string(),
        ))
    }

    fn generate_photo_proxy(
        &self,
        request: &PhotoProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        validate_photo_request(request)?;
        let source = load_source_image(&self.raw_decoder, &request.input_path)?;
        let resized = source.resize(
            u32::from(request.max_width),
            u32::from(request.max_height),
            FilterType::Lanczos3,
        );
        write_photo_proxy(&resized, &request.output_path, request.format)
    }
}

#[doc(hidden)]
pub fn validate_photo_request(request: &PhotoProxyRequest) -> Result<(), ProxyGenerationError> {
    if request.input_path.trim().is_empty() {
        return Err(ProxyGenerationError::InvalidRequest(
            "photo input path is required".to_string(),
        ));
    }
    if request.output_path.trim().is_empty() {
        return Err(ProxyGenerationError::InvalidRequest(
            "photo output path is required".to_string(),
        ));
    }
    if request.max_width == 0 || request.max_height == 0 {
        return Err(ProxyGenerationError::InvalidRequest(
            "photo max dimensions must be > 0".to_string(),
        ));
    }
    Ok(())
}

#[doc(hidden)]
pub fn load_source_image<D: RawPhotoDecoder>(
    raw_decoder: &D,
    input_path: &str,
) -> Result<DynamicImage, ProxyGenerationError> {
    match image::open(input_path) {
        Ok(image) => Ok(image),
        Err(image_error) => raw_decoder
            .decode_photo(input_path)
            .map_err(|raw_error| {
                ProxyGenerationError::Process(format!(
                    "unable to decode photo source with image crate ({image_error}) or raw fallback ({raw_error})"
                ))
            }),
    }
}

#[doc(hidden)]
pub fn write_photo_proxy(
    image: &DynamicImage,
    output_path: &str,
    format: PhotoProxyFormat,
) -> Result<(), ProxyGenerationError> {
    let output = Path::new(output_path);
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
        }
    }

    let file =
        File::create(output).map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
    let mut writer = BufWriter::new(file);
    let image_format = match format {
        PhotoProxyFormat::Jpeg => ImageFormat::Jpeg,
        PhotoProxyFormat::Webp => ImageFormat::WebP,
    };
    image
        .write_to(&mut writer, image_format)
        .map_err(|error| ProxyGenerationError::Process(error.to_string()))
}

#[doc(hidden)]
pub fn to_rgb8_from_integer(
    data: &[u16],
    cpp: usize,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    if cpp == 0 {
        return Err("raw decoder returned cpp=0".to_string());
    }
    let pixel_count = usize::try_from(width)
        .ok()
        .and_then(|w| usize::try_from(height).ok().map(|h| w.saturating_mul(h)))
        .ok_or_else(|| "raw dimensions overflow".to_string())?;
    if data.len() < pixel_count.saturating_mul(cpp) {
        return Err("raw decoder returned less data than expected".to_string());
    }

    let max_value = data.iter().copied().max().unwrap_or(1).max(1) as f32;
    let mut out = Vec::with_capacity(pixel_count.saturating_mul(3));
    for pixel in 0..pixel_count {
        let base = pixel.saturating_mul(cpp);
        if cpp >= 3 {
            out.push(scale_u16_to_u8(data[base], max_value));
            out.push(scale_u16_to_u8(data[base + 1], max_value));
            out.push(scale_u16_to_u8(data[base + 2], max_value));
        } else {
            let value = scale_u16_to_u8(data[base], max_value);
            out.extend_from_slice(&[value, value, value]);
        }
    }
    Ok(out)
}

#[doc(hidden)]
pub fn to_rgb8_from_float(
    data: &[f32],
    cpp: usize,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    if cpp == 0 {
        return Err("raw decoder returned cpp=0".to_string());
    }
    let pixel_count = usize::try_from(width)
        .ok()
        .and_then(|w| usize::try_from(height).ok().map(|h| w.saturating_mul(h)))
        .ok_or_else(|| "raw dimensions overflow".to_string())?;
    if data.len() < pixel_count.saturating_mul(cpp) {
        return Err("raw decoder returned less data than expected".to_string());
    }

    let mut out = Vec::with_capacity(pixel_count.saturating_mul(3));
    for pixel in 0..pixel_count {
        let base = pixel.saturating_mul(cpp);
        if cpp >= 3 {
            out.push(scale_f32_to_u8(data[base]));
            out.push(scale_f32_to_u8(data[base + 1]));
            out.push(scale_f32_to_u8(data[base + 2]));
        } else {
            let value = scale_f32_to_u8(data[base]);
            out.extend_from_slice(&[value, value, value]);
        }
    }
    Ok(out)
}

#[doc(hidden)]
pub fn scale_u16_to_u8(value: u16, max_value: f32) -> u8 {
    ((value as f32 / max_value) * 255.0).clamp(0.0, 255.0) as u8
}

#[doc(hidden)]
pub fn scale_f32_to_u8(value: f32) -> u8 {
    (value * 255.0).clamp(0.0, 255.0) as u8
}
