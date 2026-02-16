# Job

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**job_id** | **String** |  | 
**job_type** | **JobType** |  (enum: extract_facts, generate_proxy, generate_thumbnails, generate_audio_waveform) | 
**status** | **Status** |  (enum: pending, claimed, completed, failed) | 
**asset_uuid** | **String** |  | 
**required_capabilities** | **Vec<String>** |  | 
**claimed_by** | Option<**String**> |  | [optional]
**lock_token** | Option<**String**> |  | [optional]
**locked_until** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


