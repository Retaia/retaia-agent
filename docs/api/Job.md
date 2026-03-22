# Job

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**job_id** | **String** |  | 
**job_type** | **JobType** |  (enum: extract_facts, generate_preview, generate_thumbnails, generate_audio_waveform, transcribe_audio) | 
**status** | **Status** |  (enum: pending, claimed, completed, failed) | 
**asset_uuid** | **String** |  | 
**source** | [**models::AssetPaths**](AssetPaths.md) | Source locator (relative paths + storage id) for local media access. | 
**required_capabilities** | **Vec<String>** |  | 
**claimed_by** | Option<**uuid::Uuid**> |  | [optional]
**lock_token** | Option<**String**> | Opaque current lease token. | [optional]
**fencing_token** | Option<**i32**> | Monotone write-protection token bound to the current lease. Must be replayed unchanged on heartbeat/submit/fail. | [optional]
**locked_until** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


