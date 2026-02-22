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
