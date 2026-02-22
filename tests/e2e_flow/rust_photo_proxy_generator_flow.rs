use image::{DynamicImage, GenericImageView};
use retaia_agent::{PhotoProxyFormat, PhotoProxyRequest, ProxyGenerator, RustPhotoProxyGenerator};

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
