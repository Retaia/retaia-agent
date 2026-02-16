# \AuthApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**app_features_get**](AuthApi.md#app_features_get) | **GET** /app/features | Get effective app feature switches
[**app_features_patch**](AuthApi.md#app_features_patch) | **PATCH** /app/features | Update effective app feature switches
[**app_policy_get**](AuthApi.md#app_policy_get) | **GET** /app/policy | Get runtime app policy
[**auth2fa_disable_post**](AuthApi.md#auth2fa_disable_post) | **POST** /auth/2fa/disable | Disable TOTP 2FA
[**auth2fa_enable_post**](AuthApi.md#auth2fa_enable_post) | **POST** /auth/2fa/enable | Confirm and enable TOTP 2FA
[**auth2fa_recovery_codes_regenerate_post**](AuthApi.md#auth2fa_recovery_codes_regenerate_post) | **POST** /auth/2fa/recovery-codes/regenerate | Regenerate backup recovery codes for current user
[**auth2fa_setup_post**](AuthApi.md#auth2fa_setup_post) | **POST** /auth/2fa/setup | Setup TOTP 2FA for current user
[**auth_clients_client_id_revoke_token_post**](AuthApi.md#auth_clients_client_id_revoke_token_post) | **POST** /auth/clients/{client_id}/revoke-token | Revoke one technical client token access
[**auth_clients_client_id_rotate_secret_post**](AuthApi.md#auth_clients_client_id_rotate_secret_post) | **POST** /auth/clients/{client_id}/rotate-secret | Rotate client secret key
[**auth_clients_device_cancel_post**](AuthApi.md#auth_clients_device_cancel_post) | **POST** /auth/clients/device/cancel | Cancel an in-progress device authorization flow
[**auth_clients_device_poll_post**](AuthApi.md#auth_clients_device_poll_post) | **POST** /auth/clients/device/poll | Poll device authorization status
[**auth_clients_device_start_post**](AuthApi.md#auth_clients_device_start_post) | **POST** /auth/clients/device/start | Start device authorization flow for technical client bootstrap
[**auth_clients_token_post**](AuthApi.md#auth_clients_token_post) | **POST** /auth/clients/token | Mint client bearer token from secret key
[**auth_login_post**](AuthApi.md#auth_login_post) | **POST** /auth/login | User login with email and password
[**auth_logout_post**](AuthApi.md#auth_logout_post) | **POST** /auth/logout | Logout current user token session
[**auth_lost_password_request_post**](AuthApi.md#auth_lost_password_request_post) | **POST** /auth/lost-password/request | Request lost password reset email
[**auth_lost_password_reset_post**](AuthApi.md#auth_lost_password_reset_post) | **POST** /auth/lost-password/reset | Reset password with token
[**auth_me_features_get**](AuthApi.md#auth_me_features_get) | **GET** /auth/me/features | Get current user feature preferences
[**auth_me_features_patch**](AuthApi.md#auth_me_features_patch) | **PATCH** /auth/me/features | Update current user feature preferences
[**auth_me_get**](AuthApi.md#auth_me_get) | **GET** /auth/me | Get current authenticated user
[**auth_verify_email_admin_confirm_post**](AuthApi.md#auth_verify_email_admin_confirm_post) | **POST** /auth/verify-email/admin-confirm | Admin confirms user email verification
[**auth_verify_email_confirm_post**](AuthApi.md#auth_verify_email_confirm_post) | **POST** /auth/verify-email/confirm | Confirm email verification token
[**auth_verify_email_request_post**](AuthApi.md#auth_verify_email_request_post) | **POST** /auth/verify-email/request | Request verification email



## app_features_get

> models::AppFeaturesResponse app_features_get()
Get effective app feature switches

Returns global app switches (`app_feature_enabled`). Also returns dependency/escalation metadata for deterministic client behavior. Effective global availability requires Core `feature_flags` AND `app_feature_enabled`. Normative gate: when `app_feature_enabled.features.ai=false`, `client_kind=MCP` is disabled at runtime. Admin-only endpoint. Runtime payload contract is stable: `app_feature_enabled`, `feature_governance`, `core_v1_global_features`. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::AppFeaturesResponse**](AppFeaturesResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## app_features_patch

> models::AppFeaturesResponse app_features_patch(app_features_update_request)
Update effective app feature switches

Updates effective app switches (`app_feature_enabled`). Admin-only operation. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**app_features_update_request** | [**AppFeaturesUpdateRequest**](AppFeaturesUpdateRequest.md) |  | [required] |

### Return type

[**models::AppFeaturesResponse**](AppFeaturesResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## app_policy_get

> models::AppPolicyResponse app_policy_get(client_feature_flags_contract_version)
Get runtime app policy

Returns runtime `server_policy` including `feature_flags`. This endpoint is the canonical runtime policy transport for UI_WEB, UI_MOBILE, AGENT, and MCP clients. Clients may optionally send their supported feature-flags contract version for compatibility negotiation. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_feature_flags_contract_version** | Option<**String**> | Optional client-advertised feature-flags contract version (SemVer). |  |

### Return type

[**models::AppPolicyResponse**](AppPolicyResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [OAuth2ClientCredentials](../README.md#OAuth2ClientCredentials)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth2fa_disable_post

> auth2fa_disable_post(auth2fa_otp_request)
Disable TOTP 2FA

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth2fa_otp_request** | [**Auth2faOtpRequest**](Auth2faOtpRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth2fa_enable_post

> models::Auth2faEnableResponse auth2fa_enable_post(auth2fa_otp_request)
Confirm and enable TOTP 2FA

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth2fa_otp_request** | [**Auth2faOtpRequest**](Auth2faOtpRequest.md) |  | [required] |

### Return type

[**models::Auth2faEnableResponse**](Auth2faEnableResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth2fa_recovery_codes_regenerate_post

> models::Auth2faRecoveryCodesResponse auth2fa_recovery_codes_regenerate_post()
Regenerate backup recovery codes for current user

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::Auth2faRecoveryCodesResponse**](Auth2faRecoveryCodesResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth2fa_setup_post

> models::Auth2faSetupResponse auth2fa_setup_post()
Setup TOTP 2FA for current user

Starts TOTP enrollment for external authenticator apps (Authy, Google Authenticator, 1Password, etc.). Returns provisioning material (`otpauth://` URI and secret) to be confirmed by `/auth/2fa/enable`. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::Auth2faSetupResponse**](Auth2faSetupResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_clients_client_id_revoke_token_post

> models::AuthRevokeClientTokenResponse auth_clients_client_id_revoke_token_post(client_id)
Revoke one technical client token access

Admin-only endpoint for base UI operations. Invalidates active bearer token(s) for the targeted technical client. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** |  | [required] |

### Return type

[**models::AuthRevokeClientTokenResponse**](AuthRevokeClientTokenResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_clients_client_id_rotate_secret_post

> models::AuthRotateClientSecretResponse auth_clients_client_id_rotate_secret_post(client_id)
Rotate client secret key

Admin-only operation. Rotates secret key for one client and invalidates active bearer token(s). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** |  | [required] |

### Return type

[**models::AuthRotateClientSecretResponse**](AuthRotateClientSecretResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_clients_device_cancel_post

> models::AuthDeviceCancelResponse auth_clients_device_cancel_post(auth_device_cancel_request)
Cancel an in-progress device authorization flow

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_device_cancel_request** | [**AuthDeviceCancelRequest**](AuthDeviceCancelRequest.md) |  | [required] |

### Return type

[**models::AuthDeviceCancelResponse**](AuthDeviceCancelResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_clients_device_poll_post

> models::AuthDevicePollResponse auth_clients_device_poll_post(auth_device_poll_request)
Poll device authorization status

Polls authorization status for a previously started device flow. On approval, returns one-shot `secret_key`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_device_poll_request** | [**AuthDevicePollRequest**](AuthDevicePollRequest.md) |  | [required] |

### Return type

[**models::AuthDevicePollResponse**](AuthDevicePollResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_clients_device_start_post

> models::AuthDeviceStartResponse auth_clients_device_start_post(auth_device_start_request)
Start device authorization flow for technical client bootstrap

Starts a browser-assisted authorization flow (GitHub-style) for `AGENT`/`MCP`. User validation (and optional 2FA) happens via `verification_uri`. Runtime gate: when `app_feature_enabled.features.ai=false`, `client_kind=MCP` MUST be rejected. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_device_start_request** | [**AuthDeviceStartRequest**](AuthDeviceStartRequest.md) |  | [required] |

### Return type

[**models::AuthDeviceStartResponse**](AuthDeviceStartResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_clients_token_post

> models::AuthClientTokenSuccess auth_clients_token_post(auth_client_token_request)
Mint client bearer token from secret key

Exchanges `(client_id, secret_key)` for a bearer token. Normative rule: one active token per technical client_id; minting a new token revokes the previous one. This endpoint is for technical non-interactive clients only. Runtime gate: when `app_feature_enabled.features.ai=false`, `client_kind=MCP` MUST be rejected. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_client_token_request** | [**AuthClientTokenRequest**](AuthClientTokenRequest.md) |  | [required] |

### Return type

[**models::AuthClientTokenSuccess**](AuthClientTokenSuccess.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_login_post

> models::AuthLoginSuccess auth_login_post(auth_login_request)
User login with email and password

Interactive login endpoint for supported human-operated clients (`UI_WEB`, `UI_MOBILE`, and `AGENT`). Supports optional TOTP 2FA code when enabled.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_login_request** | [**AuthLoginRequest**](AuthLoginRequest.md) |  | [required] |

### Return type

[**models::AuthLoginSuccess**](AuthLoginSuccess.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_logout_post

> auth_logout_post()
Logout current user token session

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_lost_password_request_post

> auth_lost_password_request_post(auth_email_request)
Request lost password reset email

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_email_request** | [**AuthEmailRequest**](AuthEmailRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_lost_password_reset_post

> auth_lost_password_reset_post(auth_lost_password_reset_request)
Reset password with token

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_lost_password_reset_request** | [**AuthLostPasswordResetRequest**](AuthLostPasswordResetRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_me_features_get

> models::UserFeaturesResponse auth_me_features_get()
Get current user feature preferences

Returns user-level feature preferences (`user_feature_enabled`) and effective availability. Effective availability is computed with AND semantics: `feature_flags` AND `app_feature_enabled` AND `user_feature_enabled` AND dependency constraints. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::UserFeaturesResponse**](UserFeaturesResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_me_features_patch

> models::UserFeaturesResponse auth_me_features_patch(user_features_update_request)
Update current user feature preferences

Updates user-level feature preferences (`user_feature_enabled`) for the current authenticated user. Core v1 global features are protected and cannot be disabled at user scope. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_features_update_request** | [**UserFeaturesUpdateRequest**](UserFeaturesUpdateRequest.md) |  | [required] |

### Return type

[**models::UserFeaturesResponse**](UserFeaturesResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_me_get

> models::AuthCurrentUser auth_me_get()
Get current authenticated user

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::AuthCurrentUser**](AuthCurrentUser.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_verify_email_admin_confirm_post

> auth_verify_email_admin_confirm_post(auth_email_request)
Admin confirms user email verification

Requires an authenticated admin actor, per AUTHZ matrix (FORBIDDEN_ACTOR or FORBIDDEN_SCOPE on authz failure).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_email_request** | [**AuthEmailRequest**](AuthEmailRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_verify_email_confirm_post

> auth_verify_email_confirm_post(auth_token_request)
Confirm email verification token

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_token_request** | [**AuthTokenRequest**](AuthTokenRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_verify_email_request_post

> auth_verify_email_request_post(auth_email_request)
Request verification email

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_email_request** | [**AuthEmailRequest**](AuthEmailRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

