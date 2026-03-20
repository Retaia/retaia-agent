# \OpsApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ops_agents_get**](OpsApi.md#ops_agents_get) | **GET** /ops/agents | List known agents with runtime status and debug information
[**ops_ingest_diagnostics_get**](OpsApi.md#ops_ingest_diagnostics_get) | **GET** /ops/ingest/diagnostics | Get ingest diagnostics counters and latest unmatched sidecars
[**ops_ingest_requeue_post**](OpsApi.md#ops_ingest_requeue_post) | **POST** /ops/ingest/requeue | Requeue ingest processing for a specific target
[**ops_ingest_unmatched_get**](OpsApi.md#ops_ingest_unmatched_get) | **GET** /ops/ingest/unmatched | List unmatched ingest sidecars
[**ops_jobs_queue_get**](OpsApi.md#ops_jobs_queue_get) | **GET** /ops/jobs/queue | Get jobs queue diagnostics
[**ops_locks_get**](OpsApi.md#ops_locks_get) | **GET** /ops/locks | List active operation locks
[**ops_locks_recover_post**](OpsApi.md#ops_locks_recover_post) | **POST** /ops/locks/recover | Recover stale operation locks
[**ops_readiness_get**](OpsApi.md#ops_readiness_get) | **GET** /ops/readiness | Get operational readiness checks



## ops_agents_get

> models::OpsAgentsGet200Response ops_agents_get(status, limit, offset, accept_language)
List known agents with runtime status and debug information

Requires `UserBearerAuth` and an authenticated admin actor, per AUTHZ matrix. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**status** | Option<**String**> |  |  |
**limit** | Option<**i32**> |  |  |[default to 50]
**offset** | Option<**i32**> |  |  |[default to 0]
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::OpsAgentsGet200Response**](_ops_agents_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_ingest_diagnostics_get

> models::OpsIngestDiagnosticsGet200Response ops_ingest_diagnostics_get(accept_language)
Get ingest diagnostics counters and latest unmatched sidecars

Requires `UserBearerAuth` and an authenticated admin actor, per AUTHZ matrix. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::OpsIngestDiagnosticsGet200Response**](_ops_ingest_diagnostics_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_ingest_requeue_post

> models::OpsIngestRequeuePost202Response ops_ingest_requeue_post(ops_ingest_requeue_post_request, accept_language)
Requeue ingest processing for a specific target

Requires `UserBearerAuth` and an authenticated admin actor, per AUTHZ matrix. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ops_ingest_requeue_post_request** | Option<[**OpsIngestRequeuePostRequest**](OpsIngestRequeuePostRequest.md)> |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::OpsIngestRequeuePost202Response**](_ops_ingest_requeue_post_202_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_ingest_unmatched_get

> models::OpsIngestUnmatchedGet200Response ops_ingest_unmatched_get(reason, since, limit, accept_language)
List unmatched ingest sidecars

Requires `UserBearerAuth` and an authenticated admin actor, per AUTHZ matrix. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**reason** | Option<**String**> |  |  |
**since** | Option<**String**> | ISO-8601 UTC lower bound for `detected_at` (invalid values return `400 VALIDATION_FAILED`). |  |
**limit** | Option<**i32**> |  |  |[default to 50]
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::OpsIngestUnmatchedGet200Response**](_ops_ingest_unmatched_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_jobs_queue_get

> models::OpsJobsQueueGet200Response ops_jobs_queue_get(accept_language)
Get jobs queue diagnostics

Requires `UserBearerAuth` and an authenticated admin actor, per AUTHZ matrix. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::OpsJobsQueueGet200Response**](_ops_jobs_queue_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_locks_get

> models::OpsLocksGet200Response ops_locks_get(asset_uuid, lock_type, limit, offset, accept_language)
List active operation locks

Requires `UserBearerAuth` and an authenticated admin actor, per AUTHZ matrix. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**asset_uuid** | Option<**String**> |  |  |
**lock_type** | Option<**String**> |  |  |
**limit** | Option<**i32**> | Maximum number of items to return. |  |[default to 50]
**offset** | Option<**i32**> | Zero-based pagination offset. |  |[default to 0]
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::OpsLocksGet200Response**](_ops_locks_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ops_locks_recover_post

> models::OpsLocksRecoverPost200Response ops_locks_recover_post(accept_language, ops_locks_recover_post_request)
Recover stale operation locks

Requires `UserBearerAuth` and an authenticated admin actor, per AUTHZ matrix. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |
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

> models::OpsReadinessGet200Response ops_readiness_get(accept_language)
Get operational readiness checks

Requires `UserBearerAuth` and an authenticated admin actor, per AUTHZ matrix. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::OpsReadinessGet200Response**](_ops_readiness_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

