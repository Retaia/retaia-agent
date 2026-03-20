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

> models::AssetDerived assets_uuid_derived_get(uuid, accept_language)
List available derived files for one asset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::AssetDerived**](AssetDerived.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_kind_get

> std::path::PathBuf assets_uuid_derived_kind_get(uuid, kind, range, accept_language)
Fetch one derived file by kind

Direct Core byte delivery for the current derived file. No redirect response is part of the v1 contract.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**kind** | **String** |  | [required] |
**range** | Option<**String**> | Optional byte range request. Supported for audio/video preview delivery. |  |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, */*

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_upload_complete_post

> assets_uuid_derived_upload_complete_post(uuid, if_match, idempotency_key, x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, assets_uuid_derived_upload_complete_post_request, accept_language)
Complete derived upload

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** | Strong quoted HTTP entity-tag previously read from the asset `ETag` response header. | [required] |
**idempotency_key** | **String** |  | [required] |
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**assets_uuid_derived_upload_complete_post_request** | [**AssetsUuidDerivedUploadCompletePostRequest**](AssetsUuidDerivedUploadCompletePostRequest.md) |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

 (empty response body)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_upload_init_post

> assets_uuid_derived_upload_init_post(uuid, if_match, idempotency_key, x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, assets_uuid_derived_upload_init_post_request, accept_language)
Initialize derived upload

Initializes upload for one derived file. Supported kinds: `preview_video`, `preview_audio`, `preview_photo`, `thumb`, `waveform`. Media format details are defined in the Markdown specifications. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** | Strong quoted HTTP entity-tag previously read from the asset `ETag` response header. | [required] |
**idempotency_key** | **String** |  | [required] |
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**assets_uuid_derived_upload_init_post_request** | [**AssetsUuidDerivedUploadInitPostRequest**](AssetsUuidDerivedUploadInitPostRequest.md) |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

 (empty response body)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_derived_upload_part_post

> models::AssetsUuidDerivedUploadPartPost200Response assets_uuid_derived_upload_part_post(uuid, if_match, x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, upload_id, part_number, chunk, accept_language)
Upload one part

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** | Strong quoted HTTP entity-tag previously read from the asset `ETag` response header. | [required] |
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**upload_id** | **String** |  | [required] |
**part_number** | **i32** |  | [required] |
**chunk** | **std::path::PathBuf** |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::AssetsUuidDerivedUploadPartPost200Response**](_assets__uuid__derived_upload_part_post_200_response.md)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

