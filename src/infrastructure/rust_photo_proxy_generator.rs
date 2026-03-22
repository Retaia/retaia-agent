use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use exif::{DateTime as ExifDateTime, In, Rational, Reader as ExifReader, SRational, Tag, Value};
use image::imageops::FilterType;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};

use crate::application::derived_processing_gateway::FactsPatchPayload;
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

    fn extract_media_facts(
        &self,
        input_path: &str,
    ) -> Result<FactsPatchPayload, ProxyGenerationError> {
        let source = load_source_image(&self.raw_decoder, input_path)?;
        let format = image::ImageFormat::from_path(input_path)
            .ok()
            .map(|format| {
                format
                    .extensions_str()
                    .first()
                    .copied()
                    .unwrap_or("image")
                    .to_string()
            })
            .or_else(|| {
                Path::new(input_path)
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(|value| value.to_ascii_lowercase())
            });
        let mut facts = FactsPatchPayload {
            duration_ms: None,
            media_format: format,
            video_codec: None,
            audio_codec: None,
            width: i32::try_from(source.width()).ok(),
            height: i32::try_from(source.height()).ok(),
            fps: None,
            ..FactsPatchPayload::default()
        };
        if let Some(exif_facts) = extract_exif_facts(input_path) {
            merge_photo_facts(&mut facts, exif_facts);
        }
        Ok(facts)
    }
}

fn merge_photo_facts(target: &mut FactsPatchPayload, extra: FactsPatchPayload) {
    target.captured_at = extra.captured_at.or_else(|| target.captured_at.take());
    target.exposure_time_s = extra.exposure_time_s.or(target.exposure_time_s);
    target.aperture_f_number = extra.aperture_f_number.or(target.aperture_f_number);
    target.iso = extra.iso.or(target.iso);
    target.focal_length_mm = extra.focal_length_mm.or(target.focal_length_mm);
    target.camera_make = extra.camera_make.or_else(|| target.camera_make.take());
    target.camera_model = extra.camera_model.or_else(|| target.camera_model.take());
    target.lens_model = extra.lens_model.or_else(|| target.lens_model.take());
    target.orientation = extra.orientation.or(target.orientation);
    target.gps_latitude = extra.gps_latitude.or(target.gps_latitude);
    target.gps_longitude = extra.gps_longitude.or(target.gps_longitude);
    target.gps_altitude_m = extra.gps_altitude_m.or(target.gps_altitude_m);
}

#[doc(hidden)]
pub fn extract_exif_facts(input_path: &str) -> Option<FactsPatchPayload> {
    let file = File::open(input_path).ok()?;
    let mut reader = std::io::BufReader::new(file);
    let exif = ExifReader::new().read_from_container(&mut reader).ok()?;

    Some(FactsPatchPayload {
        captured_at: exif_captured_at(&exif),
        exposure_time_s: exif_rational_field(&exif, Tag::ExposureTime),
        aperture_f_number: exif_rational_field(&exif, Tag::FNumber),
        iso: exif_uint_field(&exif, Tag::PhotographicSensitivity),
        focal_length_mm: exif_rational_field(&exif, Tag::FocalLength),
        camera_make: exif_ascii_field(&exif, Tag::Make),
        camera_model: exif_ascii_field(&exif, Tag::Model),
        lens_model: exif_ascii_field(&exif, Tag::LensModel),
        orientation: exif_uint_field(&exif, Tag::Orientation),
        gps_latitude: exif_gps_coordinate(&exif, Tag::GPSLatitude, Tag::GPSLatitudeRef),
        gps_longitude: exif_gps_coordinate(&exif, Tag::GPSLongitude, Tag::GPSLongitudeRef),
        gps_altitude_m: exif_gps_altitude(&exif),
        ..FactsPatchPayload::default()
    })
}

fn exif_captured_at(exif: &exif::Exif) -> Option<String> {
    parse_exif_datetime_field(
        exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)?,
        exif.get_field(Tag::OffsetTimeOriginal, In::PRIMARY),
    )
    .or_else(|| exif_gps_timestamp(exif))
}

fn exif_ascii_field(exif: &exif::Exif, tag: Tag) -> Option<String> {
    let field = exif.get_field(tag, In::PRIMARY)?;
    match &field.value {
        Value::Ascii(values) => values.first().and_then(|value| {
            let trimmed = String::from_utf8_lossy(value).trim().to_string();
            (!trimmed.is_empty()).then_some(trimmed)
        }),
        _ => None,
    }
}

fn exif_uint_field(exif: &exif::Exif, tag: Tag) -> Option<i32> {
    exif.get_field(tag, In::PRIMARY)?
        .value
        .get_uint(0)
        .and_then(|value| i32::try_from(value).ok())
}

fn exif_rational_field(exif: &exif::Exif, tag: Tag) -> Option<f64> {
    let field = exif.get_field(tag, In::PRIMARY)?;
    rational_value_to_f64(&field.value)
}

fn exif_gps_coordinate(exif: &exif::Exif, tag: Tag, ref_tag: Tag) -> Option<f64> {
    let field = exif.get_field(tag, In::PRIMARY)?;
    let reference = exif_ascii_field(exif, ref_tag)?;
    gps_coordinate_to_decimal(&field.value, &reference)
}

fn exif_gps_altitude(exif: &exif::Exif) -> Option<f64> {
    let altitude = exif
        .get_field(Tag::GPSAltitude, In::PRIMARY)
        .and_then(|field| rational_value_to_f64(&field.value))?;
    let altitude_ref = exif
        .get_field(Tag::GPSAltitudeRef, In::PRIMARY)
        .and_then(|field| field.value.get_uint(0))
        .unwrap_or(0);
    Some(if altitude_ref == 1 {
        -altitude
    } else {
        altitude
    })
}

#[doc(hidden)]
pub fn parse_exif_datetime_field(
    datetime_field: &exif::Field,
    offset_field: Option<&exif::Field>,
) -> Option<String> {
    let mut datetime = exif_datetime_from_field(datetime_field)?;
    if let Some(offset_field) = offset_field {
        if let Some(offset_ascii) = exif_ascii_bytes(offset_field) {
            let _ = datetime.parse_offset(offset_ascii);
        }
    }
    exif_datetime_to_utc_rfc3339(&datetime)
}

fn exif_datetime_from_field(field: &exif::Field) -> Option<ExifDateTime> {
    ExifDateTime::from_ascii(exif_ascii_bytes(field)?).ok()
}

fn exif_ascii_bytes<'a>(field: &'a exif::Field) -> Option<&'a [u8]> {
    match &field.value {
        Value::Ascii(values) => values.first().map(Vec::as_slice),
        _ => None,
    }
}

#[doc(hidden)]
pub fn exif_datetime_to_utc_rfc3339(datetime: &ExifDateTime) -> Option<String> {
    let date = NaiveDate::from_ymd_opt(
        i32::from(datetime.year),
        u32::from(datetime.month),
        u32::from(datetime.day),
    )?;
    let time = NaiveTime::from_hms_opt(
        u32::from(datetime.hour),
        u32::from(datetime.minute),
        u32::from(datetime.second),
    )?;
    let naive = NaiveDateTime::new(date, time);
    let offset = FixedOffset::east_opt(i32::from(datetime.offset?) * 60)?;
    let local = offset.from_local_datetime(&naive).single()?;
    Some(
        local
            .with_timezone(&Utc)
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    )
}

fn exif_gps_timestamp(exif: &exif::Exif) -> Option<String> {
    let date = exif_ascii_field(exif, Tag::GPSDateStamp)?;
    let time_field = exif.get_field(Tag::GPSTimeStamp, In::PRIMARY)?;
    gps_timestamp_to_utc_rfc3339(&date, &time_field.value)
}

#[doc(hidden)]
pub fn gps_timestamp_to_utc_rfc3339(date: &str, time_value: &Value) -> Option<String> {
    let [year, month, day] = parse_gps_date(date)?;
    let components = match time_value {
        Value::Rational(values) if values.len() >= 3 => values,
        _ => return None,
    };
    let hour = rational_to_f64(&components[0])?;
    let minute = rational_to_f64(&components[1])?;
    let second = rational_to_f64(&components[2])?;
    let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32)?;
    let time = NaiveTime::from_hms_opt(
        hour.trunc() as u32,
        minute.trunc() as u32,
        second.trunc() as u32,
    )?;
    Some(
        Utc.from_utc_datetime(&NaiveDateTime::new(date, time))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    )
}

fn parse_gps_date(date: &str) -> Option<[i32; 3]> {
    let mut parts = date.split(':');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<i32>().ok()?;
    let day = parts.next()?.parse::<i32>().ok()?;
    Some([year, month, day])
}

#[doc(hidden)]
pub fn gps_coordinate_to_decimal(value: &Value, reference: &str) -> Option<f64> {
    let values = match value {
        Value::Rational(values) if values.len() >= 3 => values,
        _ => return None,
    };
    let degrees = rational_to_f64(&values[0])?;
    let minutes = rational_to_f64(&values[1])?;
    let seconds = rational_to_f64(&values[2])?;
    let sign = match reference.trim().to_ascii_uppercase().as_str() {
        "S" | "W" => -1.0,
        "N" | "E" => 1.0,
        _ => return None,
    };
    Some(sign * (degrees + minutes / 60.0 + seconds / 3600.0))
}

fn rational_value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Rational(values) => values.first().and_then(rational_to_f64),
        Value::SRational(values) => values.first().and_then(signed_rational_to_f64),
        _ => None,
    }
}

fn rational_to_f64(value: &Rational) -> Option<f64> {
    (value.denom != 0).then_some(value.num as f64 / value.denom as f64)
}

fn signed_rational_to_f64(value: &SRational) -> Option<f64> {
    (value.denom != 0).then_some(value.num as f64 / value.denom as f64)
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
