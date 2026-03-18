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
**revision_etag** | **String** | Strong opaque asset revision tag to be reused in `If-Match` for the next mutation. Changes on any accepted human-visible business mutation and stays stable for purely technical noise with no review/operator impact. | 
**captured_at** | Option<**String**> |  | [optional]
**duration** | Option<**f64**> |  | [optional]
**tags** | Option<**Vec<String>**> |  | [optional]
**has_proxy** | Option<**bool**> |  | [optional]
**thumb_url** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


