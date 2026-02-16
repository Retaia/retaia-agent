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

> Vec<models::Job> jobs_get()
List claimable jobs for the authenticated agent

Returns jobs with status `pending` and compatible with the agent capabilities. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Job>**](Job.md)

### Authorization

[OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## jobs_job_id_claim_post

> models::Job jobs_job_id_claim_post(job_id)
Claim a job (atomic lease)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** |  | [required] |

### Return type

[**models::Job**](Job.md)

### Authorization

[OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## jobs_job_id_fail_post

> jobs_job_id_fail_post(job_id, idempotency_key, jobs_job_id_fail_post_request)
Mark job as failed

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |
**jobs_job_id_fail_post_request** | [**JobsJobIdFailPostRequest**](JobsJobIdFailPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## jobs_job_id_heartbeat_post

> models::JobsJobIdHeartbeatPost200Response jobs_job_id_heartbeat_post(job_id, jobs_job_id_heartbeat_post_request)
Extend job lease

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** |  | [required] |
**jobs_job_id_heartbeat_post_request** | [**JobsJobIdHeartbeatPostRequest**](JobsJobIdHeartbeatPostRequest.md) |  | [required] |

### Return type

[**models::JobsJobIdHeartbeatPost200Response**](_jobs__job_id__heartbeat_post_200_response.md)

### Authorization

[OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## jobs_job_id_submit_post

> jobs_job_id_submit_post(job_id, idempotency_key, job_submit_request)
Submit job result patch

Submits one job result patch. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |
**job_submit_request** | [**JobSubmitRequest**](JobSubmitRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

