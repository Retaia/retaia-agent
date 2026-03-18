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

[UserBearerAuth](../README.md#UserBearerAuth), [TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

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

[UserBearerAuth](../README.md#UserBearerAuth), [TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_upload_complete_post

> assets_uuid_derived_upload_complete_post(uuid, if_match, idempotency_key, x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, assets_uuid_derived_upload_complete_post_request)
Complete derived upload

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**assets_uuid_derived_upload_complete_post_request** | [**AssetsUuidDerivedUploadCompletePostRequest**](AssetsUuidDerivedUploadCompletePostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_upload_init_post

> assets_uuid_derived_upload_init_post(uuid, if_match, idempotency_key, x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, assets_uuid_derived_upload_init_post_request)
Initialize derived upload

Initializes upload for one derived file. Normative media profile constraints: - `proxy_video`: `video/mp4` (H.264/AVC, browser-compatible), source framerate preserved. - `proxy_audio`: `audio/mp4` (AAC-LC) or `audio/mpeg`. - `proxy_photo` / `thumb`: `image/jpeg` or `image/webp`. - `waveform`: `application/json` (preferred) or `application/octet-stream`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**assets_uuid_derived_upload_init_post_request** | [**AssetsUuidDerivedUploadInitPostRequest**](AssetsUuidDerivedUploadInitPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_upload_part_post

> assets_uuid_derived_upload_part_post(uuid, if_match, x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, assets_uuid_derived_upload_part_post_request)
Upload one part

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** |  | [required] |
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**assets_uuid_derived_upload_part_post_request** | [**AssetsUuidDerivedUploadPartPostRequest**](AssetsUuidDerivedUploadPartPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

