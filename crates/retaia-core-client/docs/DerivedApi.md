# \DerivedApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assets_uuid_derived_get**](DerivedApi.md#assets_uuid_derived_get) | **GET** /assets/{uuid}/derived | List available derived files for one asset
[**assets_uuid_derived_kind_get**](DerivedApi.md#assets_uuid_derived_kind_get) | **GET** /assets/{uuid}/derived/{kind} | Fetch one derived file by kind
[**assets_uuid_derived_upload_complete_post**](DerivedApi.md#assets_uuid_derived_upload_complete_post) | **POST** /assets/{uuid}/derived/upload/complete | Complete derived upload
[**assets_uuid_derived_upload_init_post**](DerivedApi.md#assets_uuid_derived_upload_init_post) | **POST** /assets/{uuid}/derived/upload/init | Initialize derived upload
[**assets_uuid_derived_upload_part_post**](DerivedApi.md#assets_uuid_derived_upload_part_post) | **POST** /assets/{uuid}/derived/upload/part | Upload one part



## assets_uuid_derived_get

> std::collections::HashMap<String, serde_json::Value> assets_uuid_derived_get(uuid)
List available derived files for one asset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |

### Return type

[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_kind_get

> assets_uuid_derived_kind_get(uuid, kind)
Fetch one derived file by kind

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**kind** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_upload_complete_post

> assets_uuid_derived_upload_complete_post(uuid, idempotency_key, assets_uuid_derived_upload_complete_post_request)
Complete derived upload

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |
**assets_uuid_derived_upload_complete_post_request** | [**AssetsUuidDerivedUploadCompletePostRequest**](AssetsUuidDerivedUploadCompletePostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_upload_init_post

> assets_uuid_derived_upload_init_post(uuid, idempotency_key, assets_uuid_derived_upload_init_post_request)
Initialize derived upload

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |
**assets_uuid_derived_upload_init_post_request** | [**AssetsUuidDerivedUploadInitPostRequest**](AssetsUuidDerivedUploadInitPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_upload_part_post

> assets_uuid_derived_upload_part_post(uuid, assets_uuid_derived_upload_part_post_request)
Upload one part

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**assets_uuid_derived_upload_part_post_request** | [**AssetsUuidDerivedUploadPartPostRequest**](AssetsUuidDerivedUploadPartPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

