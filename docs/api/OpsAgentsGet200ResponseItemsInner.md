# OpsAgentsGet200ResponseItemsInner

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**agent_id** | **uuid::Uuid** |  | 
**client_id** | **String** |  | 
**agent_name** | **String** |  | 
**agent_version** | **String** |  | 
**os_name** | Option<**OsName**> |  (enum: linux, macos, windows) | [optional]
**os_version** | Option<**String**> |  | [optional]
**arch** | Option<**String**> |  | [optional]
**status** | **Status** |  (enum: online_idle, online_busy, stale) | 
**identity_conflict** | **bool** |  | 
**last_seen_at** | **String** |  | 
**last_register_at** | **String** |  | 
**last_heartbeat_at** | Option<**String**> |  | [optional]
**effective_capabilities** | **Vec<String>** |  | 
**capability_warnings** | **Vec<String>** |  | 
**current_job** | Option<[**models::OpsAgentsGet200ResponseItemsInnerCurrentJob**](OpsAgentsGet200ResponseItemsInnerCurrentJob.md)> |  | [optional]
**last_successful_job** | Option<[**models::OpsAgentsGet200ResponseItemsInnerLastSuccessfulJob**](OpsAgentsGet200ResponseItemsInnerLastSuccessfulJob.md)> |  | [optional]
**last_failed_job** | Option<[**models::OpsAgentsGet200ResponseItemsInnerLastFailedJob**](OpsAgentsGet200ResponseItemsInnerLastFailedJob.md)> |  | [optional]
**debug** | [**models::OpsAgentsGet200ResponseItemsInnerDebug**](OpsAgentsGet200ResponseItemsInnerDebug.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


