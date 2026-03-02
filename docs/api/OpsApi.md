# \OpsApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ops_ingest_diagnostics_get**](OpsApi.md#ops_ingest_diagnostics_get) | **GET** /ops/ingest/diagnostics | Get ingest diagnostics counters and latest unmatched sidecars
[**ops_ingest_requeue_post**](OpsApi.md#ops_ingest_requeue_post) | **POST** /ops/ingest/requeue | Requeue ingest processing for a specific target
[**ops_ingest_unmatched_get**](OpsApi.md#ops_ingest_unmatched_get) | **GET** /ops/ingest/unmatched | List unmatched ingest sidecars
[**ops_jobs_queue_get**](OpsApi.md#ops_jobs_queue_get) | **GET** /ops/jobs/queue | Get jobs queue diagnostics
[**ops_locks_get**](OpsApi.md#ops_locks_get) | **GET** /ops/locks | List active operation locks
[**ops_locks_recover_post**](OpsApi.md#ops_locks_recover_post) | **POST** /ops/locks/recover | Recover stale operation locks
[**ops_readiness_get**](OpsApi.md#ops_readiness_get) | **GET** /ops/readiness | Get operational readiness checks



## ops_ingest_diagnostics_get

> models::OpsIngestDiagnosticsGet200Response ops_ingest_diagnostics_get()
Get ingest diagnostics counters and latest unmatched sidecars

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::OpsIngestDiagnosticsGet200Response**](_ops_ingest_diagnostics_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_ingest_requeue_post

> models::OpsIngestRequeuePost202Response ops_ingest_requeue_post(ops_ingest_requeue_post_request)
Requeue ingest processing for a specific target

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ops_ingest_requeue_post_request** | [**OpsIngestRequeuePostRequest**](OpsIngestRequeuePostRequest.md) |  | [required] |

### Return type

[**models::OpsIngestRequeuePost202Response**](_ops_ingest_requeue_post_202_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_ingest_unmatched_get

> models::OpsIngestUnmatchedGet200Response ops_ingest_unmatched_get(reason, since, limit)
List unmatched ingest sidecars

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**reason** | Option<**String**> |  |  |
**since** | Option<**String**> | ISO-8601 UTC lower bound for `detected_at` (invalid values return `400 VALIDATION_FAILED`). |  |
**limit** | Option<**i32**> |  |  |[default to 50]

### Return type

[**models::OpsIngestUnmatchedGet200Response**](_ops_ingest_unmatched_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_jobs_queue_get

> models::OpsJobsQueueGet200Response ops_jobs_queue_get()
Get jobs queue diagnostics

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::OpsJobsQueueGet200Response**](_ops_jobs_queue_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_locks_get

> models::OpsLocksGet200Response ops_locks_get(asset_uuid, lock_type, limit, offset)
List active operation locks

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**asset_uuid** | Option<**String**> |  |  |
**lock_type** | Option<**String**> |  |  |
**limit** | Option<**i32**> | Maximum number of items to return. |  |[default to 50]
**offset** | Option<**i32**> | Zero-based pagination offset. |  |[default to 0]

### Return type

[**models::OpsLocksGet200Response**](_ops_locks_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_locks_recover_post

> models::OpsLocksRecoverPost200Response ops_locks_recover_post(ops_locks_recover_post_request)
Recover stale operation locks

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ops_locks_recover_post_request** | Option<[**OpsLocksRecoverPostRequest**](OpsLocksRecoverPostRequest.md)> |  |  |

### Return type

[**models::OpsLocksRecoverPost200Response**](_ops_locks_recover_post_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_readiness_get

> models::OpsReadinessGet200Response ops_readiness_get()
Get operational readiness checks

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::OpsReadinessGet200Response**](_ops_readiness_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

