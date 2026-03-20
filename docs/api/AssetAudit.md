# AssetAudit

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**path_history** | Option<**Vec<String>**> | Chronological path history in ascending order, using canonical relative paths only. | [optional]
**revision_history** | Option<[**Vec<models::AssetAuditRevisionHistoryInner>**](AssetAuditRevisionHistoryInner.md)> | Append-only revision timeline in ascending `revision` order. The entry marked `is_current=true` matches the current `summary.revision_etag`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


