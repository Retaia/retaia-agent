use retaia_agent::{DerivedKind, DerivedUploadInit, validate_derived_upload_init};

#[test]
fn tdd_derived_kind_content_type_matrix_matches_v1_constraints() {
    assert!(DerivedKind::ProxyVideo.allows_content_type("video/mp4"));
    assert!(!DerivedKind::ProxyVideo.allows_content_type("video/webm"));

    assert!(DerivedKind::ProxyAudio.allows_content_type("audio/mp4"));
    assert!(DerivedKind::ProxyAudio.allows_content_type("audio/mpeg"));
    assert!(!DerivedKind::ProxyAudio.allows_content_type("audio/wav"));

    assert!(DerivedKind::ProxyPhoto.allows_content_type("image/jpeg"));
    assert!(DerivedKind::ProxyPhoto.allows_content_type("image/webp"));
    assert!(DerivedKind::Thumb.allows_content_type("image/jpeg"));
    assert!(!DerivedKind::Thumb.allows_content_type("image/png"));

    assert!(DerivedKind::Waveform.allows_content_type("application/json"));
    assert!(DerivedKind::Waveform.allows_content_type("application/octet-stream"));
}

#[test]
fn tdd_validate_derived_upload_init_rejects_invalid_content_type_or_zero_size() {
    let invalid_mime = DerivedUploadInit {
        asset_uuid: "asset-1".to_string(),
        kind: DerivedKind::ProxyVideo,
        content_type: "video/webm".to_string(),
        size_bytes: 1024,
        sha256: None,
        idempotency_key: "idem-1".to_string(),
    };
    let mime_err = validate_derived_upload_init(&invalid_mime).expect_err("mime must fail");
    assert!(
        mime_err
            .to_string()
            .contains("invalid derived content type")
    );

    let invalid_size = DerivedUploadInit {
        asset_uuid: "asset-1".to_string(),
        kind: DerivedKind::ProxyVideo,
        content_type: "video/mp4".to_string(),
        size_bytes: 0,
        sha256: None,
        idempotency_key: "idem-2".to_string(),
    };
    let size_err = validate_derived_upload_init(&invalid_size).expect_err("size must fail");
    assert!(size_err.to_string().contains("invalid derived upload size"));
}

#[test]
fn tdd_validate_derived_upload_init_accepts_v1_valid_proxy_video_payload() {
    let request = DerivedUploadInit {
        asset_uuid: "asset-1".to_string(),
        kind: DerivedKind::ProxyVideo,
        content_type: "video/mp4".to_string(),
        size_bytes: 32_000,
        sha256: Some("abc".to_string()),
        idempotency_key: "idem-3".to_string(),
    };

    validate_derived_upload_init(&request).expect("valid payload");
}
