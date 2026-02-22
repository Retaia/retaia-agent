use image::{DynamicImage, GenericImageView};
use retaia_agent::{PhotoProxyFormat, PhotoProxyRequest, ProxyGenerator, RustPhotoProxyGenerator};

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
