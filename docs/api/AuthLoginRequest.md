# AuthLoginRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**email** | **String** |  | 
**password** | **String** |  | 
**client_id** | Option<**String**> | Optional client identifier. If omitted, server derives one from runtime client identity. | [optional]
**client_kind** | Option<[**models::ClientKind**](ClientKind.md)> |  | [optional]
**otp_code** | Option<**String**> | 6-digit TOTP code when 2FA is enabled on account. | [optional]
**recovery_code** | Option<**String**> | One-shot backup code when TOTP app is unavailable. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


