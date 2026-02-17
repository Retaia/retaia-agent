use retaia_agent::{DerivedKind, DerivedUploadInit, validate_derived_upload_init};

#[test]
fn bdd_given_derived_upload_init_request_when_kind_proxy_audio_then_only_mp4_or_mpeg_are_accepted()
{
    let accepted_mp4 = DerivedUploadInit {
        asset_uuid: "asset-a".to_string(),
        kind: DerivedKind::ProxyAudio,
        content_type: "audio/mp4".to_string(),
        size_bytes: 2048,
        sha256: None,
        idempotency_key: "idem-a1".to_string(),
    };
    validate_derived_upload_init(&accepted_mp4).expect("audio/mp4 should pass");

    let accepted_mpeg = DerivedUploadInit {
        content_type: "audio/mpeg".to_string(),
        idempotency_key: "idem-a2".to_string(),
        ..accepted_mp4.clone()
    };
    validate_derived_upload_init(&accepted_mpeg).expect("audio/mpeg should pass");

    let rejected_wav = DerivedUploadInit {
        content_type: "audio/wav".to_string(),
        idempotency_key: "idem-a3".to_string(),
        ..accepted_mp4
    };
    assert!(validate_derived_upload_init(&rejected_wav).is_err());
}

#[test]
fn bdd_given_derived_upload_init_request_when_kind_waveform_then_json_or_octet_stream_are_accepted()
{
    let json_waveform = DerivedUploadInit {
        asset_uuid: "asset-w".to_string(),
        kind: DerivedKind::Waveform,
        content_type: "application/json".to_string(),
        size_bytes: 1024,
        sha256: None,
        idempotency_key: "idem-w1".to_string(),
    };
    validate_derived_upload_init(&json_waveform).expect("json waveform should pass");

    let binary_waveform = DerivedUploadInit {
        content_type: "application/octet-stream".to_string(),
        idempotency_key: "idem-w2".to_string(),
        ..json_waveform
    };
    validate_derived_upload_init(&binary_waveform).expect("octet-stream waveform should pass");
}
