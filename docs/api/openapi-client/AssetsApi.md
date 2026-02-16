# \AssetsApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assets_get**](AssetsApi.md#assets_get) | **GET** /assets | List assets
[**assets_uuid_get**](AssetsApi.md#assets_uuid_get) | **GET** /assets/{uuid} | Get one asset detail
[**assets_uuid_patch**](AssetsApi.md#assets_uuid_patch) | **PATCH** /assets/{uuid} | Update human metadata on one asset
[**assets_uuid_reprocess_post**](AssetsApi.md#assets_uuid_reprocess_post) | **POST** /assets/{uuid}/reprocess | Trigger explicit reprocess



## assets_get

> models::AssetsGet200Response assets_get(state, media_type, tags, has_proxy, tags_mode, q, location_country, location_city, geo_bbox, sort, limit, cursor)
List assets

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**state** | Option<[**AssetState**](AssetState.md)> |  |  |
**media_type** | Option<**String**> |  |  |
**tags** | Option<**String**> |  |  |
**has_proxy** | Option<**bool**> |  |  |
**tags_mode** | Option<**String**> |  |  |
**q** | Option<**String**> | Full-text query over filename, notes and transcript_text (available in v1). |  |
**location_country** | Option<**String**> | Country-level location filter (uses secure derived search index). |  |
**location_city** | Option<**String**> | City-level location filter (uses secure derived search index). |  |
**geo_bbox** | Option<**String**> | Bounding box filter `min_lon,min_lat,max_lon,max_lat` (uses secure derived spatial index). |  |
**sort** | Option<**String**> |  |  |
**limit** | Option<**i32**> |  |  |
**cursor** | Option<**String**> |  |  |

### Return type

[**models::AssetsGet200Response**](_assets_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_get

> models::AssetDetail assets_uuid_get(uuid)
Get one asset detail

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |

### Return type

[**models::AssetDetail**](AssetDetail.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_patch

> assets_uuid_patch(uuid, assets_uuid_patch_request)
Update human metadata on one asset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**assets_uuid_patch_request** | [**AssetsUuidPatchRequest**](AssetsUuidPatchRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_reprocess_post

> assets_uuid_reprocess_post(uuid, idempotency_key)
Trigger explicit reprocess

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

