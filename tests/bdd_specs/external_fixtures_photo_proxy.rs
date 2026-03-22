use image::GenericImageView;
use retaia_agent::{PhotoProxyFormat, PhotoProxyRequest, ProxyGenerator, RustPhotoProxyGenerator};

use crate::external_fixtures::load_manifest_entries;

#[test]
fn bdd_given_external_fixture_manifest_when_loading_then_entries_have_existing_files() {
    let entries = load_manifest_entries();
    assert!(
        !entries.is_empty(),
        "external fixture manifest must not be empty"
    );

    for entry in entries {
        assert!(
            entry.absolute_path().exists(),
            "manifest references missing file: {}",
            entry.relative_path
        );
    }
}

#[test]
fn bdd_given_supported_raw_external_fixtures_when_generating_photo_proxy_then_outputs_are_created()
{
    let entries: Vec<_> = load_manifest_entries()
        .into_iter()
        .filter(|entry| entry.kind == "raw_photo" && entry.expected == "supported")
        .collect();
    assert!(
        !entries.is_empty(),
        "expected at least one supported raw fixture"
    );

    let temp = tempfile::tempdir().expect("tempdir");
    let generator = RustPhotoProxyGenerator::default();

    for (index, entry) in entries.iter().enumerate() {
        let output = temp.path().join(format!("raw-proxy-{index}.webp"));
        generator
            .generate_photo_proxy(&PhotoProxyRequest {
                input_path: entry.absolute_path().display().to_string(),
                output_path: output.display().to_string(),
                format: PhotoProxyFormat::Webp,
                max_width: 480,
                max_height: 320,
            })
            .unwrap_or_else(|error| {
                panic!(
                    "raw fixture should generate proxy: {} ({error:?})",
                    entry.relative_path
                )
            });

        let generated = image::open(&output).expect("generated proxy should be readable");
        let (width, height) = generated.dimensions();
        assert!(width > 0 && width <= 480, "unexpected width for {entry:?}");
        assert!(
            height > 0 && height <= 320,
            "unexpected height for {entry:?}"
        );
    }
}

#[test]
fn bdd_given_supported_raw_external_fixtures_when_extracting_facts_then_dimensions_are_present() {
    let entries: Vec<_> = load_manifest_entries()
        .into_iter()
        .filter(|entry| entry.kind == "raw_photo" && entry.expected == "supported")
        .collect();
    assert!(
        !entries.is_empty(),
        "expected at least one supported raw fixture"
    );

    let generator = RustPhotoProxyGenerator::default();

    for entry in entries {
        let facts = generator
            .extract_media_facts(&entry.absolute_path().display().to_string())
            .unwrap_or_else(|error| {
                panic!(
                    "raw fixture should expose facts: {} ({error:?})",
                    entry.relative_path
                )
            });
        assert!(
            facts.duration_ms.is_none(),
            "raw photo must not expose duration"
        );
        assert!(facts.width.unwrap_or_default() > 0);
        assert!(facts.height.unwrap_or_default() > 0);
        assert!(facts.media_format.as_deref().is_some());
    }
}

#[test]
fn bdd_given_real_canon_raw_fixture_when_extracting_facts_then_stable_exif_values_are_exposed() {
    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "raw/canon/sample1.cr2")
        .expect("expected canon raw fixture");
    let generator = RustPhotoProxyGenerator::default();

    let facts = generator
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .expect("raw fixture should expose facts");

    assert_eq!(facts.media_format.as_deref(), Some("cr2"));
    assert_eq!(facts.width, Some(8896));
    assert_eq!(facts.height, Some(5920));
    assert_eq!(facts.camera_make.as_deref(), Some("Canon"));
    assert_eq!(facts.camera_model.as_deref(), Some("Canon EOS 5DS"));
    assert_eq!(facts.lens_model.as_deref(), Some("EF24-105mm f/4L IS USM"));
    assert_eq!(facts.orientation, Some(1));
    assert_eq!(facts.iso, Some(125));
    assert_eq!(facts.focal_length_mm, Some(60.0));
    assert_eq!(facts.aperture_f_number, Some(7.1));
    assert_eq!(facts.exposure_time_s, Some(1.0 / 60.0));
    assert_eq!(facts.captured_at, None);
    assert_eq!(facts.gps_latitude, None);
    assert_eq!(facts.gps_longitude, None);
}

#[test]
fn bdd_given_real_nikon_nef_fixture_when_extracting_facts_then_vendor_metadata_is_exposed() {
    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "raw/nikon/sample1.nef")
        .expect("expected nikon nef fixture");
    let generator = RustPhotoProxyGenerator::default();

    let facts = generator
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .expect("raw fixture should expose facts");

    assert_eq!(facts.media_format.as_deref(), Some("nef"));
    assert_eq!(facts.width, Some(4288));
    assert_eq!(facts.height, Some(2844));
    assert_eq!(facts.camera_make.as_deref(), Some("NIKON CORPORATION"));
    assert_eq!(facts.camera_model.as_deref(), Some("NIKON D3"));
    assert_eq!(facts.lens_model, None);
    assert_eq!(facts.orientation, Some(1));
    assert_eq!(facts.iso, Some(400));
    assert_eq!(facts.focal_length_mm, Some(17.0));
    assert_eq!(facts.aperture_f_number, Some(16.0));
    assert_eq!(facts.exposure_time_s, Some(0.008));
    assert_eq!(facts.captured_at, None);
    assert_eq!(facts.gps_latitude, None);
    assert_eq!(facts.gps_longitude, None);
}

#[test]
fn bdd_given_real_nikon_nrw_fixture_when_extracting_facts_then_vendor_metadata_is_exposed() {
    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "raw/nikon/sample1.nrw")
        .expect("expected nikon nrw fixture");
    let generator = RustPhotoProxyGenerator::default();

    let facts = generator
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .expect("raw fixture should expose facts");

    assert_eq!(facts.media_format.as_deref(), Some("nrw"));
    assert_eq!(facts.width, Some(3664));
    assert_eq!(facts.height, Some(2744));
    assert_eq!(facts.camera_make.as_deref(), Some("NIKON"));
    assert_eq!(facts.camera_model.as_deref(), Some("COOLPIX P7100"));
    assert_eq!(facts.lens_model, None);
    assert_eq!(facts.orientation, Some(1));
    assert_eq!(facts.iso, Some(100));
    assert_eq!(facts.focal_length_mm, Some(13.0));
    assert_eq!(facts.aperture_f_number, Some(3.5));
    assert_eq!(facts.exposure_time_s, Some(0.008));
    assert_eq!(facts.captured_at, None);
    assert_eq!(facts.gps_latitude, None);
    assert_eq!(facts.gps_longitude, None);
}

#[test]
fn bdd_given_real_sony_arw_fixture_when_extracting_facts_then_vendor_metadata_is_exposed() {
    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "raw/sony/sample1.arw")
        .expect("expected sony arw fixture");
    let generator = RustPhotoProxyGenerator::default();

    let facts = generator
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .expect("raw fixture should expose facts");

    assert_eq!(facts.media_format.as_deref(), Some("arw"));
    assert_eq!(facts.width, Some(6048));
    assert_eq!(facts.height, Some(4024));
    assert_eq!(facts.camera_make.as_deref(), Some("SONY"));
    assert_eq!(facts.camera_model.as_deref(), Some("ILCE-7M3"));
    assert_eq!(facts.lens_model.as_deref(), Some("FE 28-70mm F3.5-5.6 OSS"));
    assert_eq!(facts.orientation, Some(1));
    assert_eq!(facts.iso, Some(100));
    assert_eq!(facts.focal_length_mm, Some(70.0));
    assert_eq!(facts.aperture_f_number, Some(5.6));
    assert_eq!(facts.exposure_time_s, Some(0.00125));
    assert_eq!(facts.captured_at.as_deref(), Some("2018-08-07T07:01:52Z"));
    assert_eq!(facts.gps_latitude, None);
    assert_eq!(facts.gps_longitude, None);
}

#[test]
fn bdd_given_second_real_sony_arw_fixture_when_extracting_facts_then_vendor_metadata_is_exposed() {
    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "raw/sony/sample2.arw")
        .expect("expected second sony arw fixture");
    let generator = RustPhotoProxyGenerator::default();

    let facts = generator
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .expect("raw fixture should expose facts");

    assert_eq!(facts.media_format.as_deref(), Some("arw"));
    assert_eq!(facts.width, Some(6048));
    assert_eq!(facts.height, Some(4024));
    assert_eq!(facts.camera_make.as_deref(), Some("SONY"));
    assert_eq!(facts.camera_model.as_deref(), Some("ILCE-7M3"));
    assert_eq!(facts.lens_model.as_deref(), Some("FE 28-70mm F3.5-5.6 OSS"));
    assert_eq!(facts.orientation, Some(1));
    assert_eq!(facts.iso, Some(1000));
    assert_eq!(facts.focal_length_mm, Some(41.0));
    assert_eq!(facts.aperture_f_number, Some(4.5));
    assert_eq!(facts.exposure_time_s, Some(0.02));
    assert_eq!(facts.captured_at.as_deref(), Some("2018-10-29T22:17:43Z"));
    assert_eq!(facts.gps_latitude, None);
    assert_eq!(facts.gps_longitude, None);
}
