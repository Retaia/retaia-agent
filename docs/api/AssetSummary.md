# AssetSummary

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **String** |  | 
**name** | Option<**String**> |  | [optional]
**media_type** | **MediaType** |  (enum: VIDEO, PHOTO, AUDIO) | 
**state** | [**models::AssetState**](AssetState.md) |  | 
**created_at** | **String** |  | 
**updated_at** | **String** | Timestamp of the last accepted business mutation on this asset. Informational only; not a write precondition. | 
**revision_etag** | **String** | Strong quoted HTTP entity-tag to be reused byte-for-byte in `If-Match` for the next mutation. | 
**captured_at** | Option<**String**> |  | [optional]
**duration** | Option<**f64**> |  | [optional]
**tags** | Option<**Vec<String>**> |  | [optional]
**has_preview** | Option<**bool**> |  | [optional]
**thumb_url** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


