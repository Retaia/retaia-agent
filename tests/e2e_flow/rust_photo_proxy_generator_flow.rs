use image::{DynamicImage, GenericImageView};
use retaia_agent::infrastructure::rust_photo_proxy_generator::{
    load_source_image, scale_f32_to_u8, scale_u16_to_u8, to_rgb8_from_float, to_rgb8_from_integer,
    validate_photo_request, write_photo_proxy,
};
use retaia_agent::{
    AudioProxyFormat, AudioProxyRequest, PhotoProxyFormat, PhotoProxyRequest, ProxyGenerationError,
    ProxyGenerator, RawPhotoDecoder, RustPhotoProxyGenerator, VideoProxyRequest,
};

struct E2eRawDecoder;

impl RawPhotoDecoder for E2eRawDecoder {
    fn decode_photo(&self, _input_path: &str) -> Result<DynamicImage, String> {
        Ok(DynamicImage::new_rgb8(240, 180))
    }
}

struct E2eFailingRawDecoder;

impl RawPhotoDecoder for E2eFailingRawDecoder {
    fn decode_photo(&self, _input_path: &str) -> Result<DynamicImage, String> {
        Err("e2e decoder failure".to_string())
    }
}

#[test]
fn e2e_rust_photo_proxy_generator_flow_supports_jpeg_and_webp_outputs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let input = temp.path().join("asset.jpg");
    let out_jpeg = temp.path().join("proxy.jpg");
    let out_webp = temp.path().join("proxy.webp");

    DynamicImage::new_rgb8(1024, 768)
        .save(&input)
        .expect("save input");

    let generator = RustPhotoProxyGenerator::default();

    generator
        .generate_photo_proxy(&PhotoProxyRequest {
            input_path: input.display().to_string(),
            output_path: out_jpeg.display().to_string(),
            format: PhotoProxyFormat::Jpeg,
            max_width: 512,
            max_height: 384,
        })
        .expect("jpeg proxy generation");

    generator
        .generate_photo_proxy(&PhotoProxyRequest {
            input_path: input.display().to_string(),
            output_path: out_webp.display().to_string(),
            format: PhotoProxyFormat::Webp,
            max_width: 256,
            max_height: 192,
        })
        .expect("webp proxy generation");

    let jpeg = image::open(&out_jpeg).expect("open jpeg");
    let webp = image::open(&out_webp).expect("open webp");
    assert_eq!(jpeg.dimensions(), (512, 384));
    assert_eq!(webp.dimensions(), (256, 192));
}

#[test]
fn e2e_rust_photo_proxy_generator_flow_covers_validation_and_decoder_fallback_paths() {
    let invalid = validate_photo_request(&PhotoProxyRequest {
        input_path: String::new(),
        output_path: "/tmp/out.jpg".to_string(),
        format: PhotoProxyFormat::Jpeg,
        max_width: 200,
        max_height: 100,
    });
    assert!(matches!(
        invalid,
        Err(ProxyGenerationError::InvalidRequest(_))
    ));

    let temp = tempfile::tempdir().expect("tempdir");
    let non_image = temp.path().join("source.nef");
    std::fs::write(&non_image, b"not-image").expect("write non-image");

    let fallback = load_source_image(&E2eRawDecoder, &non_image.display().to_string())
        .expect("fallback should produce image");
    assert_eq!(fallback.dimensions(), (240, 180));

    let failing = load_source_image(&E2eFailingRawDecoder, &non_image.display().to_string());
    assert!(matches!(failing, Err(ProxyGenerationError::Process(_))));
}

#[test]
fn e2e_rust_photo_proxy_generator_flow_covers_output_write_and_conversion_helpers() {
    let temp = tempfile::tempdir().expect("tempdir");
    let nested = temp.path().join("proxy/deep/out.webp");
    write_photo_proxy(
        &DynamicImage::new_rgb8(50, 30),
        &nested.display().to_string(),
        PhotoProxyFormat::Webp,
    )
    .expect("write nested webp");
    assert!(nested.exists());

    let rgb = to_rgb8_from_integer(&[5, 10, 20, 25, 30, 35], 3, 2, 1).expect("rgb integer");
    let gray = to_rgb8_from_integer(&[20, 40], 1, 2, 1).expect("gray integer");
    let float_rgb = to_rgb8_from_float(&[0.2, 0.3, 0.4], 3, 1, 1).expect("float rgb");
    let float_gray = to_rgb8_from_float(&[0.5], 1, 1, 1).expect("float gray");
    assert_eq!(rgb.len(), 6);
    assert_eq!(gray.len(), 6);
    assert_eq!(float_rgb.len(), 3);
    assert_eq!(float_gray.len(), 3);
    assert!(to_rgb8_from_integer(&[1], 3, 1, 1).is_err());
    assert!(to_rgb8_from_float(&[0.1], 2, 1, 1).is_err());
    assert_eq!(scale_u16_to_u8(500, 100.0), 255);
    assert_eq!(scale_f32_to_u8(1.2), 255);
}

#[test]
fn e2e_rust_photo_proxy_generator_flow_rejects_audio_video_paths_on_photo_generator() {
    let generator = RustPhotoProxyGenerator::new(E2eRawDecoder);
    let video = generator.generate_video_proxy(&VideoProxyRequest {
        input_path: "/tmp/a.mov".to_string(),
        output_path: "/tmp/a.mp4".to_string(),
        max_width: 640,
        max_height: 360,
        video_bitrate_kbps: 1200,
        audio_bitrate_kbps: 96,
    });
    let audio = generator.generate_audio_proxy(&AudioProxyRequest {
        input_path: "/tmp/a.wav".to_string(),
        output_path: "/tmp/a.mp3".to_string(),
        format: AudioProxyFormat::Mpeg,
        audio_bitrate_kbps: 160,
        sample_rate_hz: 44100,
    });
    assert!(matches!(
        video,
        Err(ProxyGenerationError::InvalidRequest(_))
    ));
    assert!(matches!(
        audio,
        Err(ProxyGenerationError::InvalidRequest(_))
    ));
}
