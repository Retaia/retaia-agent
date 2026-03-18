# \AssetsApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assets_get**](AssetsApi.md#assets_get) | **GET** /assets | List assets
[**assets_uuid_get**](AssetsApi.md#assets_uuid_get) | **GET** /assets/{uuid} | Get one asset detail
[**assets_uuid_patch**](AssetsApi.md#assets_uuid_patch) | **PATCH** /assets/{uuid} | Update one asset (metadata and lifecycle transitions)
[**assets_uuid_reprocess_post**](AssetsApi.md#assets_uuid_reprocess_post) | **POST** /assets/{uuid}/reprocess | Trigger explicit reprocess



## assets_get

> models::AssetsGet200Response assets_get(state, media_type, tags, has_proxy, tags_mode, q, location_country, location_city, geo_bbox, sort, captured_at_from, captured_at_to, limit, cursor)
List assets

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**state** | Option<[**AssetState**](AssetState.md)> |  |  |
**media_type** | Option<**String**> |  |  |
**tags** | Option<**String**> |  |  |
**has_proxy** | Option<**bool**> |  |  |
**tags_mode** | Option<**String**> |  |  |
**q** | Option<**String**> | Full-text query over filename and notes (v1 baseline). |  |
**location_country** | Option<**String**> | Country-level location filter (uses secure derived search index). |  |
**location_city** | Option<**String**> | City-level location filter (uses secure derived search index). |  |
**geo_bbox** | Option<**String**> | Bounding box filter `min_lon,min_lat,max_lon,max_lat` (uses secure derived spatial index). |  |
**sort** | Option<**String**> |  |  |
**captured_at_from** | Option<**String**> | Include assets with `captured_at` greater than or equal to this timestamp (UTC ISO-8601). |  |
**captured_at_to** | Option<**String**> | Include assets with `captured_at` lower than or equal to this timestamp (UTC ISO-8601). |  |
**limit** | Option<**i32**> |  |  |
**cursor** | Option<**String**> |  |  |

### Return type

[**models::AssetsGet200Response**](_assets_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_get

> assets_uuid_get(uuid)
Get one asset detail

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_patch

> assets_uuid_patch(uuid, if_match, assets_uuid_patch_request)
Update one asset (metadata and lifecycle transitions)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** |  | [required] |
**assets_uuid_patch_request** | [**AssetsUuidPatchRequest**](AssetsUuidPatchRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_reprocess_post

> assets_uuid_reprocess_post(uuid, if_match, idempotency_key)
Trigger explicit reprocess

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

