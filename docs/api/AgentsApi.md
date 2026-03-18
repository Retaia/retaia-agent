# \AgentsApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**agents_register_post**](AgentsApi.md#agents_register_post) | **POST** /agents/register | Register a processing agent



## agents_register_post

> models::AgentsRegisterPost200Response agents_register_post(x_retaia_agent_id, x_retaia_open_pgp_fingerprint, x_retaia_signature, x_retaia_signature_timestamp, x_retaia_signature_nonce, agents_register_post_request)
Register a processing agent

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_retaia_agent_id** | **uuid::Uuid** |  | [required] |
**x_retaia_open_pgp_fingerprint** | **String** |  | [required] |
**x_retaia_signature** | **String** |  | [required] |
**x_retaia_signature_timestamp** | **String** |  | [required] |
**x_retaia_signature_nonce** | **String** |  | [required] |
**agents_register_post_request** | [**AgentsRegisterPostRequest**](AgentsRegisterPostRequest.md) |  | [required] |

### Return type

[**models::AgentsRegisterPost200Response**](_agents_register_post_200_response.md)

### Authorization

[TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

