use std::sync::Mutex;

use image::{DynamicImage, GenericImageView, ImageFormat};
use retaia_agent::{
    PhotoProxyFormat, PhotoProxyRequest, ProxyGenerationError, ProxyGenerator, RawPhotoDecoder,
    RustPhotoProxyGenerator,
};

struct StubRawDecoder {
    calls: Mutex<usize>,
    image: DynamicImage,
}

impl StubRawDecoder {
    fn new(image: DynamicImage) -> Self {
        Self {
            calls: Mutex::new(0),
            image,
        }
    }

    fn calls(&self) -> usize {
        *self.calls.lock().expect("calls")
    }
}

impl RawPhotoDecoder for StubRawDecoder {
    fn decode_photo(&self, _input_path: &str) -> Result<DynamicImage, String> {
        *self.calls.lock().expect("calls") += 1;
        Ok(self.image.clone())
    }
}

#[test]
fn tdd_rust_photo_proxy_generator_emits_jpeg_with_max_dimensions_applied() {
    let temp = tempfile::tempdir().expect("tempdir");
    let input = temp.path().join("source.png");
    let output = temp.path().join("proxy.jpg");

    let image = DynamicImage::new_rgb8(1600, 900);
    image.save(&input).expect("save input png");

    let generator = RustPhotoProxyGenerator::default();
    generator
        .generate_photo_proxy(&PhotoProxyRequest {
            input_path: input.display().to_string(),
            output_path: output.display().to_string(),
            format: PhotoProxyFormat::Jpeg,
            max_width: 640,
            max_height: 360,
        })
        .expect("jpeg proxy generation should succeed");

    let produced = image::open(&output).expect("output image should open");
    assert!(matches!(
        ImageFormat::from_path(&output).expect("format from path"),
        ImageFormat::Jpeg
    ));
    assert_eq!(produced.dimensions(), (640, 360));
}

#[test]
fn tdd_rust_photo_proxy_generator_uses_raw_fallback_when_image_open_fails() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing_input = temp.path().join("source.cr2");
    let output = temp.path().join("proxy.webp");
    let decoder = StubRawDecoder::new(DynamicImage::new_rgb8(100, 50));
    let generator = RustPhotoProxyGenerator::new(decoder);

    generator
        .generate_photo_proxy(&PhotoProxyRequest {
            input_path: missing_input.display().to_string(),
            output_path: output.display().to_string(),
            format: PhotoProxyFormat::Webp,
            max_width: 80,
            max_height: 40,
        })
        .expect("raw fallback should generate output");

    assert_eq!(generator.raw_decoder().calls(), 1);
    let produced = image::open(&output).expect("output image should open");
    assert_eq!(produced.dimensions(), (80, 40));
}

#[test]
fn tdd_rust_photo_proxy_generator_rejects_zero_dimensions() {
    let generator = RustPhotoProxyGenerator::default();

    let error = generator
        .generate_photo_proxy(&PhotoProxyRequest {
            input_path: "/tmp/source.png".to_string(),
            output_path: "/tmp/proxy.jpg".to_string(),
            format: PhotoProxyFormat::Jpeg,
            max_width: 0,
            max_height: 360,
        })
        .expect_err("invalid dimensions should fail");

    assert!(matches!(error, ProxyGenerationError::InvalidRequest(_)));
}

#[test]
fn tdd_rust_photo_proxy_generator_rejects_empty_output_path() {
    let generator = RustPhotoProxyGenerator::default();

    let error = generator
        .generate_photo_proxy(&PhotoProxyRequest {
            input_path: "/tmp/source.png".to_string(),
            output_path: String::new(),
            format: PhotoProxyFormat::Jpeg,
            max_width: 640,
            max_height: 360,
        })
        .expect_err("empty output path should fail");

    assert!(matches!(error, ProxyGenerationError::InvalidRequest(_)));
}

#[test]
fn tdd_rust_photo_proxy_generator_rejects_zero_max_height() {
    let generator = RustPhotoProxyGenerator::default();

    let error = generator
        .generate_photo_proxy(&PhotoProxyRequest {
            input_path: "/tmp/source.png".to_string(),
            output_path: "/tmp/proxy.jpg".to_string(),
            format: PhotoProxyFormat::Jpeg,
            max_width: 640,
            max_height: 0,
        })
        .expect_err("zero max_height should fail");

    assert!(matches!(error, ProxyGenerationError::InvalidRequest(_)));
}

#[test]
fn tdd_rust_photo_proxy_generator_missing_input_path_returns_process_error() {
    let temp = tempfile::tempdir().expect("tempdir");
    let input = temp.path().join("missing-source.jpg");
    let output = temp.path().join("proxy.jpg");

    let generator = RustPhotoProxyGenerator::default();
    let error = generator
        .generate_photo_proxy(&PhotoProxyRequest {
            input_path: input.display().to_string(),
            output_path: output.display().to_string(),
            format: PhotoProxyFormat::Jpeg,
            max_width: 320,
            max_height: 200,
        })
        .expect_err("missing input should fail");

    assert!(matches!(error, ProxyGenerationError::Process(_)));
}

#[test]
fn tdd_rust_photo_proxy_generator_mismatched_raw_extension_with_text_content_fails_deterministically()
 {
    let temp = tempfile::tempdir().expect("tempdir");
    let input = temp.path().join("fake.cr2");
    let output = temp.path().join("proxy.webp");
    std::fs::write(&input, b"this is not a raw image").expect("write fake raw");

    let generator = RustPhotoProxyGenerator::default();
    let error = generator
        .generate_photo_proxy(&PhotoProxyRequest {
            input_path: input.display().to_string(),
            output_path: output.display().to_string(),
            format: PhotoProxyFormat::Webp,
            max_width: 320,
            max_height: 200,
        })
        .expect_err("fake raw should fail");

    match error {
        ProxyGenerationError::Process(message) => {
            assert!(message.contains("unable to decode photo source"))
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn tdd_rust_photo_proxy_generator_mixed_batch_reports_success_and_failure_counts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let jpg = temp.path().join("source.jpg");
    let png = temp.path().join("source.png");
    let tiff = temp.path().join("source.tiff");
    let webp = temp.path().join("source.webp");
    let fake_raw = temp.path().join("fake.cr2");
    let empty_file = temp.path().join("empty.jpg");

    DynamicImage::new_rgb8(640, 360)
        .save(&jpg)
        .expect("save jpg source");
    DynamicImage::new_rgb8(640, 360)
        .save(&png)
        .expect("save png source");
    DynamicImage::new_rgb8(640, 360)
        .save(&tiff)
        .expect("save tiff source");
    DynamicImage::new_rgb8(640, 360)
        .save(&webp)
        .expect("save webp source");
    std::fs::write(&fake_raw, b"not-a-real-raw").expect("write fake raw");
    std::fs::write(&empty_file, b"").expect("write empty file");

    let entries = vec![jpg, png, tiff, webp, fake_raw, empty_file];
    let generator = RustPhotoProxyGenerator::default();
    let mut success = 0usize;
    let mut failed = 0usize;

    for (index, input) in entries.iter().enumerate() {
        let output = temp.path().join(format!("proxy-{index}.webp"));
        let result = generator.generate_photo_proxy(&PhotoProxyRequest {
            input_path: input.display().to_string(),
            output_path: output.display().to_string(),
            format: PhotoProxyFormat::Webp,
            max_width: 320,
            max_height: 180,
        });
        match result {
            Ok(()) => success += 1,
            Err(ProxyGenerationError::Process(_)) => failed += 1,
            Err(other) => panic!("unexpected error variant in mixed batch: {other:?}"),
        }
    }

    assert_eq!(success, 4);
    assert_eq!(failed, 2);
}
