# Auth2faSetupResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**method** | **Method** |  (enum: TOTP) | 
**issuer** | **String** |  | 
**account_name** | **String** | Login identifier shown in authenticator app. | 
**secret** | **String** | Base32 TOTP secret (show once during setup). | 
**otpauth_uri** | **String** | Provisioning URI for authenticator apps. | 
**qr_svg** | Option<**String**> | Optional inline SVG QR code payload. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


