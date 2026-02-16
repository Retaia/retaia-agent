# \BatchesApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**batches_moves_batch_id_get**](BatchesApi.md#batches_moves_batch_id_get) | **GET** /batches/moves/{batch_id} | Get move batch status and report
[**batches_moves_post**](BatchesApi.md#batches_moves_post) | **POST** /batches/moves | Execute or dry-run move batch
[**batches_moves_preview_post**](BatchesApi.md#batches_moves_preview_post) | **POST** /batches/moves/preview | Preview move batch



## batches_moves_batch_id_get

> std::collections::HashMap<String, serde_json::Value> batches_moves_batch_id_get(batch_id)
Get move batch status and report

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_id** | **String** |  | [required] |

### Return type

[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## batches_moves_post

> batches_moves_post(idempotency_key, batches_moves_post_request)
Execute or dry-run move batch

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**idempotency_key** | **String** |  | [required] |
**batches_moves_post_request** | [**BatchesMovesPostRequest**](BatchesMovesPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## batches_moves_preview_post

> batches_moves_preview_post(batches_moves_preview_post_request)
Preview move batch

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batches_moves_preview_post_request** | [**BatchesMovesPreviewPostRequest**](BatchesMovesPreviewPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

