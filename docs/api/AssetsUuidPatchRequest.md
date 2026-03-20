# AssetsUuidPatchRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tags** | Option<**Vec<String>**> |  | [optional]
**notes** | Option<**String**> |  | [optional]
**fields** | Option<[**std::collections::HashMap<String, models::AssetFieldValue>**](AssetFieldValue.md)> | Shared complementary metadata map. Values stay JSON-simple; domains requiring dedicated semantics, storage or security rules must not be hidden implicitly here.  | [optional]
**processing_profile** | Option<**ProcessingProfile**> |  (enum: video_standard, audio_undefined, audio_music, audio_voice, photo_standard) | [optional]
**state** | Option<**State**> |  (enum: DECISION_PENDING, DECIDED_KEEP, DECIDED_REJECT, ARCHIVED, REJECTED) | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


