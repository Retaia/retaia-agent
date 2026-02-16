# \PurgeApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assets_uuid_purge_post**](PurgeApi.md#assets_uuid_purge_post) | **POST** /assets/{uuid}/purge | Purge one rejected asset
[**assets_uuid_purge_preview_post**](PurgeApi.md#assets_uuid_purge_preview_post) | **POST** /assets/{uuid}/purge/preview | Preview purge impact for one asset



## assets_uuid_purge_post

> assets_uuid_purge_post(uuid, idempotency_key, assets_uuid_purge_post_request)
Purge one rejected asset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |
**assets_uuid_purge_post_request** | [**AssetsUuidPurgePostRequest**](AssetsUuidPurgePostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_purge_preview_post

> assets_uuid_purge_preview_post(uuid)
Preview purge impact for one asset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

