# \DecisionsApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assets_uuid_decision_post**](DecisionsApi.md#assets_uuid_decision_post) | **POST** /assets/{uuid}/decision | Set or update a human decision
[**assets_uuid_reopen_post**](DecisionsApi.md#assets_uuid_reopen_post) | **POST** /assets/{uuid}/reopen | Reopen archived or rejected asset to decision pending
[**decisions_apply_post**](DecisionsApi.md#decisions_apply_post) | **POST** /decisions/apply | Apply bulk decisions from preview token
[**decisions_preview_post**](DecisionsApi.md#decisions_preview_post) | **POST** /decisions/preview | Preview bulk decisions



## assets_uuid_decision_post

> assets_uuid_decision_post(uuid, idempotency_key, assets_uuid_decision_post_request)
Set or update a human decision

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**idempotency_key** | **String** |  | [required] |
**assets_uuid_decision_post_request** | [**AssetsUuidDecisionPostRequest**](AssetsUuidDecisionPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assets_uuid_reopen_post

> assets_uuid_reopen_post(uuid)
Reopen archived or rejected asset to decision pending

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


## decisions_apply_post

> decisions_apply_post(idempotency_key, decisions_apply_post_request)
Apply bulk decisions from preview token

Available in v1.1+ when `features.decisions.bulk=true`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**idempotency_key** | **String** |  | [required] |
**decisions_apply_post_request** | [**DecisionsApplyPostRequest**](DecisionsApplyPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## decisions_preview_post

> decisions_preview_post(decisions_preview_post_request)
Preview bulk decisions

Available in v1.1+ when `features.decisions.bulk=true`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decisions_preview_post_request** | [**DecisionsPreviewPostRequest**](DecisionsPreviewPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

