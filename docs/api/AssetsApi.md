# \AssetsApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assets_get**](AssetsApi.md#assets_get) | **GET** /assets | List assets
[**assets_uuid_get**](AssetsApi.md#assets_uuid_get) | **GET** /assets/{uuid} | Get one asset detail
[**assets_uuid_patch**](AssetsApi.md#assets_uuid_patch) | **PATCH** /assets/{uuid} | Update one asset (metadata and lifecycle transitions)
[**assets_uuid_reprocess_post**](AssetsApi.md#assets_uuid_reprocess_post) | **POST** /assets/{uuid}/reprocess | Trigger explicit reprocess



## assets_get

> models::AssetsGet200Response assets_get(state, media_type, tags, has_preview, tags_mode, q, location_country, location_city, geo_bbox, sort, captured_at_from, captured_at_to, limit, cursor, accept_language)
List assets

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**state** | Option<[**Vec<models::AssetState>**](Models__AssetState.md)> | Comma-separated asset states. Ordering is not significant; duplicates are ignored. |  |
**media_type** | Option<**String**> |  |  |
**tags** | Option<[**Vec<String>**](String.md)> | Comma-separated human tags. Ordering is not significant; duplicates are ignored. |  |
**has_preview** | Option<**bool**> |  |  |
**tags_mode** | Option<**String**> |  |  |
**q** | Option<**String**> | Full-text query over filename and notes (v1 baseline). |  |
**location_country** | Option<**String**> | Country-level location filter (uses secure derived search index). |  |
**location_city** | Option<**String**> | City-level location filter (uses secure derived search index). |  |
**geo_bbox** | Option<**String**> | Bounding box filter `min_lon,min_lat,max_lon,max_lat` with lon in `[-180,180]`, lat in `[-90,90]`, strict `min < max`, and no antimeridian crossing in v1. |  |
**sort** | Option<**String**> | Primary sort key. Ties are stabilized by `uuid` ascending. |  |[default to -created_at]
**captured_at_from** | Option<**String**> | Include assets with `captured_at` greater than or equal to this timestamp (UTC ISO-8601). |  |
**captured_at_to** | Option<**String**> | Include assets with `captured_at` lower than or equal to this timestamp (UTC ISO-8601). |  |
**limit** | Option<**i32**> |  |  |
**cursor** | Option<**String**> | Opaque server-issued cursor bound to the exact `(filters, sort, limit)` tuple of the previous page. |  |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::AssetsGet200Response**](_assets_get_200_response.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_get

> models::AssetDetail assets_uuid_get(uuid, accept_language)
Get one asset detail

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

[**models::AssetDetail**](AssetDetail.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_patch

> assets_uuid_patch(uuid, if_match, assets_uuid_patch_request, accept_language)
Update one asset (metadata and lifecycle transitions)

Partial human mutation. Only the provided fields are updated; omitted fields stay unchanged. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** | Strong quoted HTTP entity-tag previously read from the asset `ETag` response header. | [required] |
**assets_uuid_patch_request** | [**AssetsUuidPatchRequest**](AssetsUuidPatchRequest.md) |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_reprocess_post

> assets_uuid_reprocess_post(uuid, if_match, idempotency_key, accept_language)
Trigger explicit reprocess

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** | Strong quoted HTTP entity-tag previously read from the asset `ETag` response header. | [required] |
**idempotency_key** | **String** |  | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

