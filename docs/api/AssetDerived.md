# AssetDerived

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**proxy_video_url** | Option<**String**> |  | [optional]
**proxy_audio_url** | Option<**String**> |  | [optional]
**proxy_photo_url** | Option<**String**> |  | [optional]
**waveform_url** | Option<**String**> | Required for any asset with an exploitable audio track once state progresses beyond READY. If `media_type=AUDIO` or facts reveal an exploitable audio track, this field MUST be present in all business states beyond READY. UI local fallback may exist for degraded playback UX but never replaces the required server/agent-derived waveform.  | [optional]
**thumbs** | Option<**Vec<String>**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


