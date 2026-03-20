# \PurgeApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assets_purge_post**](PurgeApi.md#assets_purge_post) | **POST** /assets/purge | Purge multiple rejected assets
[**assets_uuid_purge_post**](PurgeApi.md#assets_uuid_purge_post) | **POST** /assets/{uuid}/purge | Purge one rejected asset
[**assets_uuid_purge_preview_post**](PurgeApi.md#assets_uuid_purge_preview_post) | **POST** /assets/{uuid}/purge/preview | Preview purge impact for one asset



## assets_purge_post

> models::AssetsPurgePost200Response assets_purge_post(idempotency_key, assets_purge_post_request, accept_language)
Purge multiple rejected assets

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**idempotency_key** | **String** |  | [required] |
**assets_purge_post_request** | [**AssetsPurgePostRequest**](AssetsPurgePostRequest.md) |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::AssetsPurgePost200Response**](_assets_purge_post_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_purge_post

> assets_uuid_purge_post(uuid, if_match, idempotency_key, assets_uuid_purge_post_request, accept_language)
Purge one rejected asset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** | Strong quoted HTTP entity-tag previously read from the asset `ETag` response header. | [required] |
**idempotency_key** | **String** |  | [required] |
**assets_uuid_purge_post_request** | [**AssetsUuidPurgePostRequest**](AssetsUuidPurgePostRequest.md) |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_purge_preview_post

> assets_uuid_purge_preview_post(uuid, accept_language)
Preview purge impact for one asset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

