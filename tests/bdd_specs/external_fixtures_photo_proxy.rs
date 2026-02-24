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
