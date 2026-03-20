# \DecisionsApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assets_uuid_reopen_post**](DecisionsApi.md#assets_uuid_reopen_post) | **POST** /assets/{uuid}/reopen | Reopen archived or rejected asset to decision pending



## assets_uuid_reopen_post

> assets_uuid_reopen_post(uuid, if_match, accept_language)
Reopen archived or rejected asset to decision pending

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uuid** | **String** |  | [required] |
**if_match** | **String** | Strong quoted HTTP entity-tag previously read from the asset `ETag` response header. | [required] |
**accept_language** | Option<**String**> | Optional locale preference for localized human-readable messages. Business payload semantics remain locale-independent. |  |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

