use image::{DynamicImage, GenericImageView};
use retaia_agent::infrastructure::rust_photo_proxy_generator::{
    load_source_image, scale_f32_to_u8, scale_u16_to_u8, to_rgb8_from_float, to_rgb8_from_integer,
    validate_photo_request, write_photo_proxy,
};
use retaia_agent::{
    AudioProxyRequest, PhotoProxyFormat, PhotoProxyRequest, ProxyGenerationError, ProxyGenerator,
    RawPhotoDecoder, RustPhotoProxyGenerator, VideoProxyRequest,
};

struct ScenarioRawDecoder;

impl RawPhotoDecoder for ScenarioRawDecoder {
    fn decode_photo(&self, _input_path: &str) -> Result<DynamicImage, String> {
        Ok(DynamicImage::new_rgb8(320, 200))
    }
}

struct FailingRawDecoder;

impl RawPhotoDecoder for FailingRawDecoder {
    fn decode_photo(&self, _input_path: &str) -> Result<DynamicImage, String> {
        Err("decoder failed".to_string())
    }
}

#[test]
fn bdd_given_photo_input_when_generating_webp_proxy_then_output_is_resized_and_readable() {
    let temp = tempfile::tempdir().expect("tempdir");
    let input = temp.path().join("asset.png");
    let output = temp.path().join("asset_proxy.webp");

    DynamicImage::new_rgb8(1200, 800)
        .save(&input)
        .expect("save input");

    let generator = RustPhotoProxyGenerator::default();
    generator
        .generate_photo_proxy(&PhotoProxyRequest {
            input_path: input.display().to_string(),
            output_path: output.display().to_string(),
            format: PhotoProxyFormat::Webp,
            max_width: 600,
            max_height: 400,
        })
        .expect("proxy");

    let produced = image::open(&output).expect("open output");
    assert_eq!(produced.dimensions(), (600, 400));
}

#[test]
fn bdd_given_empty_photo_paths_when_validating_then_request_is_rejected() {
    let err = validate_photo_request(&PhotoProxyRequest {
        input_path: String::new(),
        output_path: "/tmp/out.jpg".to_string(),
        format: PhotoProxyFormat::Jpeg,
        max_width: 320,
        max_height: 240,
    })
    .expect_err("empty input path should be rejected");
    assert!(matches!(err, ProxyGenerationError::InvalidRequest(_)));
}

#[test]
fn bdd_given_zero_photo_dimensions_when_validating_then_request_is_rejected() {
    let err = validate_photo_request(&PhotoProxyRequest {
        input_path: "/tmp/source.jpg".to_string(),
        output_path: "/tmp/out.jpg".to_string(),
        format: PhotoProxyFormat::Jpeg,
        max_width: 0,
        max_height: 240,
    })
    .expect_err("zero dimensions should be rejected");
    assert!(matches!(err, ProxyGenerationError::InvalidRequest(_)));
}

#[test]
fn bdd_given_non_image_source_when_loading_then_raw_decoder_fallback_is_used() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("source.nef");
    std::fs::write(&path, b"not-an-image").expect("write source");

    let image = load_source_image(&ScenarioRawDecoder, &path.display().to_string())
        .expect("raw fallback should return image");
    assert_eq!(image.dimensions(), (320, 200));
}

#[test]
fn bdd_given_non_image_source_and_failing_raw_decoder_when_loading_then_error_is_explicit() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("source.cr2");
    std::fs::write(&path, b"invalid").expect("write source");

    let err = load_source_image(&FailingRawDecoder, &path.display().to_string())
        .expect_err("failing fallback should return process error");
    assert!(matches!(err, ProxyGenerationError::Process(_)));
}

#[test]
fn bdd_given_nested_output_path_when_writing_proxy_then_parent_directories_are_created() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = temp.path().join("a/b/c/proxy.jpg");
    write_photo_proxy(
        &DynamicImage::new_rgb8(40, 30),
        &output.display().to_string(),
        PhotoProxyFormat::Jpeg,
    )
    .expect("write should create parent directories");
    assert!(output.exists());
}

#[test]
fn bdd_given_rgb_integer_raw_buffer_when_converting_then_three_channel_output_is_produced() {
    let bytes =
        to_rgb8_from_integer(&[10, 20, 30, 40, 50, 60], 3, 2, 1).expect("conversion should pass");
    assert_eq!(bytes.len(), 6);
}

#[test]
fn bdd_given_gray_integer_raw_buffer_when_converting_then_channels_are_duplicated() {
    let bytes = to_rgb8_from_integer(&[16, 32], 1, 2, 1).expect("grayscale conversion");
    assert_eq!(bytes.len(), 6);
    assert_eq!(bytes[0], bytes[1]);
    assert_eq!(bytes[1], bytes[2]);
}

#[test]
fn bdd_given_invalid_integer_raw_buffer_when_converting_then_error_is_returned() {
    assert!(to_rgb8_from_integer(&[1, 2], 0, 1, 1).is_err());
    assert!(to_rgb8_from_integer(&[1], 3, 1, 1).is_err());
}

#[test]
fn bdd_given_float_raw_buffer_when_converting_then_rgb_and_grayscale_paths_are_supported() {
    let rgb = to_rgb8_from_float(&[0.2, 0.3, 0.4], 3, 1, 1).expect("float rgb");
    let gray = to_rgb8_from_float(&[0.6], 1, 1, 1).expect("float gray");
    assert_eq!(rgb.len(), 3);
    assert_eq!(gray.len(), 3);
    assert!(to_rgb8_from_float(&[], 0, 1, 1).is_err());
    assert!(to_rgb8_from_float(&[0.1], 2, 1, 1).is_err());
}

#[test]
fn bdd_given_scaling_helpers_when_values_are_out_of_range_then_output_is_clamped() {
    assert_eq!(scale_u16_to_u8(500, 100.0), 255);
    assert_eq!(scale_f32_to_u8(1.5), 255);
}

#[test]
fn bdd_given_photo_generator_when_video_or_audio_requested_then_error_is_explicit() {
    let generator = RustPhotoProxyGenerator::new(ScenarioRawDecoder);
    let video = generator.generate_video_proxy(&VideoProxyRequest {
        input_path: "/tmp/in.mov".to_string(),
        output_path: "/tmp/out.mp4".to_string(),
        max_width: 640,
        max_height: 360,
        video_bitrate_kbps: 1000,
        audio_bitrate_kbps: 96,
    });
    let audio = generator.generate_audio_proxy(&AudioProxyRequest {
        input_path: "/tmp/in.wav".to_string(),
        output_path: "/tmp/out.mp3".to_string(),
        format: retaia_agent::AudioProxyFormat::Mpeg,
        audio_bitrate_kbps: 128,
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
