# \JobsApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**jobs_get**](JobsApi.md#jobs_get) | **GET** /jobs | List claimable jobs for the authenticated agent
[**jobs_job_id_claim_post**](JobsApi.md#jobs_job_id_claim_post) | **POST** /jobs/{job_id}/claim | Claim a job (atomic lease)
[**jobs_job_id_fail_post**](JobsApi.md#jobs_job_id_fail_post) | **POST** /jobs/{job_id}/fail | Mark job as failed
[**jobs_job_id_heartbeat_post**](JobsApi.md#jobs_job_id_heartbeat_post) | **POST** /jobs/{job_id}/heartbeat | Extend job lease
[**jobs_job_id_submit_post**](JobsApi.md#jobs_job_id_submit_post) | **POST** /jobs/{job_id}/submit | Submit job result patch



## jobs_get

> Vec<models::Job> jobs_get(accept_language)
List claimable jobs for the authenticated agent

Returns jobs with status `pending` and compatible with the agent capabilities. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**Vec<models::Job>**](Job.md)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## jobs_job_id_claim_post

> models::Job jobs_job_id_claim_post(job_id, x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, accept_language)
Claim a job (atomic lease)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** |  | [required] |
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::Job**](Job.md)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## jobs_job_id_fail_post

> jobs_job_id_fail_post(job_id, idempotency_key, x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, jobs_job_id_fail_post_request, accept_language)
Mark job as failed

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**jobs_job_id_fail_post_request** | [**JobsJobIdFailPostRequest**](JobsJobIdFailPostRequest.md) |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

 (empty response body)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## jobs_job_id_heartbeat_post

> models::JobsJobIdHeartbeatPost200Response jobs_job_id_heartbeat_post(job_id, x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, jobs_job_id_heartbeat_post_request, accept_language)
Extend job lease

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** |  | [required] |
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**jobs_job_id_heartbeat_post_request** | [**JobsJobIdHeartbeatPostRequest**](JobsJobIdHeartbeatPostRequest.md) |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::JobsJobIdHeartbeatPost200Response**](_jobs__job_id__heartbeat_post_200_response.md)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## jobs_job_id_submit_post

> jobs_job_id_submit_post(job_id, idempotency_key, x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, job_submit_request, accept_language)
Submit job result patch

Submits one job result patch. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**job_submit_request** | [**JobSubmitRequest**](JobSubmitRequest.md) |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

 (empty response body)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

