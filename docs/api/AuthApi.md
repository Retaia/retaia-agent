# \AuthApi

All URIs are relative to */api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**app_features_get**](AuthApi.md#app_features_get) | **GET** /app/features | Get effective app feature switches
[**app_features_patch**](AuthApi.md#app_features_patch) | **PATCH** /app/features | Update effective app feature switches
[**app_policy_get**](AuthApi.md#app_policy_get) | **GET** /app/policy | Get runtime app policy
[**app_policy_post**](AuthApi.md#app_policy_post) | **POST** /app/policy | Update runtime app policy
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
[**auth_mcp_challenge_post**](AuthApi.md#auth_mcp_challenge_post) | **POST** /auth/mcp/challenge | Create MCP technical auth challenge
[**auth_mcp_client_id_rotate_key_post**](AuthApi.md#auth_mcp_client_id_rotate_key_post) | **POST** /auth/mcp/{client_id}/rotate-key | Rotate MCP public key
[**auth_mcp_register_post**](AuthApi.md#auth_mcp_register_post) | **POST** /auth/mcp/register | Register MCP technical client public key
[**auth_mcp_token_post**](AuthApi.md#auth_mcp_token_post) | **POST** /auth/mcp/token | Mint MCP bearer token from signed challenge
[**auth_me_features_get**](AuthApi.md#auth_me_features_get) | **GET** /auth/me/features | Get current user feature preferences
[**auth_me_features_patch**](AuthApi.md#auth_me_features_patch) | **PATCH** /auth/me/features | Update current user feature preferences
[**auth_me_get**](AuthApi.md#auth_me_get) | **GET** /auth/me | Get current authenticated user
[**auth_refresh_post**](AuthApi.md#auth_refresh_post) | **POST** /auth/refresh | Refresh interactive user bearer token
[**auth_verify_email_admin_confirm_post**](AuthApi.md#auth_verify_email_admin_confirm_post) | **POST** /auth/verify-email/admin-confirm | Admin confirms user email verification
[**auth_verify_email_confirm_post**](AuthApi.md#auth_verify_email_confirm_post) | **POST** /auth/verify-email/confirm | Confirm email verification token
[**auth_verify_email_request_post**](AuthApi.md#auth_verify_email_request_post) | **POST** /auth/verify-email/request | Request verification email
[**auth_webauthn_authenticate_options_post**](AuthApi.md#auth_webauthn_authenticate_options_post) | **POST** /auth/webauthn/authenticate/options | Start WebAuthn authentication
[**auth_webauthn_authenticate_verify_post**](AuthApi.md#auth_webauthn_authenticate_verify_post) | **POST** /auth/webauthn/authenticate/verify | Verify WebAuthn authentication
[**auth_webauthn_register_options_post**](AuthApi.md#auth_webauthn_register_options_post) | **POST** /auth/webauthn/register/options | Start WebAuthn device registration
[**auth_webauthn_register_verify_post**](AuthApi.md#auth_webauthn_register_verify_post) | **POST** /auth/webauthn/register/verify | Verify WebAuthn device registration



## app_features_get

> models::AppFeaturesResponse app_features_get()
Get effective app feature switches

Returns global app switches (`app_feature_enabled`). Also returns dependency/escalation metadata for deterministic client behavior. Effective global availability requires Core `feature_flags` AND `app_feature_enabled`. Normative gate: when `app_feature_enabled.features.ai=false`, only MCP functions that depend on AI are disabled at runtime. Admin-only endpoint. Runtime payload contract is stable: `app_feature_enabled`, `feature_governance`, `core_v1_global_features`. 

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

Returns runtime `server_policy` including `feature_flags`. This endpoint is the canonical runtime policy transport for UI_WEB, AGENT, and MCP clients. Clients may optionally send their supported feature-flags contract version for compatibility negotiation. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_feature_flags_contract_version** | Option<**String**> | Optional client-advertised feature-flags contract version (SemVer). |  |

### Return type

[**models::AppPolicyResponse**](AppPolicyResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth), [TechnicalBearerAuth](../README.md#TechnicalBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## app_policy_post

> models::AppPolicyResponse app_policy_post(app_policy_update_request)
Update runtime app policy

Updates runtime `feature_flags` when they are persisted in a mutable backend controlled by Core. Requires `UserBearerAuth` and an authenticated admin actor, per AUTHZ matrix. Flags still in `code-backed` introduction/validation phase are visible in `GET /app/policy` but MUST be rejected by this endpoint with `409 STATE_CONFLICT`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**app_policy_update_request** | [**AppPolicyUpdateRequest**](AppPolicyUpdateRequest.md) |  | [required] |

### Return type

[**models::AppPolicyResponse**](AppPolicyResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
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

Starts a browser-assisted authorization flow (GitHub-style) for `AGENT_TECHNICAL`. User validation (and optional 2FA) happens via `verification_uri`. 

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

Exchanges `(client_id, secret_key)` for a bearer token. Normative rule: one active token per AGENT technical client_id; minting a new token revokes the previous one. This endpoint is for `AGENT_TECHNICAL` only. 

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

Interactive bootstrap/recovery login endpoint for supported human-operated clients (`UI_WEB` and `AGENT`). Supports optional TOTP 2FA code when enabled.

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


## auth_mcp_challenge_post

> models::AuthMcpChallengeResponse auth_mcp_challenge_post(auth_mcp_challenge_request)
Create MCP technical auth challenge

Creates a one-shot challenge for `MCP_TECHNICAL`. The challenge must expire within 5 minutes and must be rejected after first successful use or replay. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_mcp_challenge_request** | [**AuthMcpChallengeRequest**](AuthMcpChallengeRequest.md) |  | [required] |

### Return type

[**models::AuthMcpChallengeResponse**](AuthMcpChallengeResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_mcp_client_id_rotate_key_post

> models::AuthMcpRegisterResponse auth_mcp_client_id_rotate_key_post(client_id, auth_mcp_register_request)
Rotate MCP public key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** |  | [required] |
**auth_mcp_register_request** | [**AuthMcpRegisterRequest**](AuthMcpRegisterRequest.md) |  | [required] |

### Return type

[**models::AuthMcpRegisterResponse**](AuthMcpRegisterResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_mcp_register_post

> models::AuthMcpRegisterResponse auth_mcp_register_post(auth_mcp_register_request)
Register MCP technical client public key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_mcp_register_request** | [**AuthMcpRegisterRequest**](AuthMcpRegisterRequest.md) |  | [required] |

### Return type

[**models::AuthMcpRegisterResponse**](AuthMcpRegisterResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_mcp_token_post

> models::AuthClientTokenSuccess auth_mcp_token_post(auth_mcp_token_request)
Mint MCP bearer token from signed challenge

Mints a technical bearer token for `MCP_TECHNICAL` from a valid signature over a still-valid one-shot challenge. Expired, replayed or already-consumed challenges must be rejected. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_mcp_token_request** | [**AuthMcpTokenRequest**](AuthMcpTokenRequest.md) |  | [required] |

### Return type

[**models::AuthClientTokenSuccess**](AuthClientTokenSuccess.md)

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


## auth_refresh_post

> models::AuthLoginSuccess auth_refresh_post(auth_refresh_request)
Refresh interactive user bearer token

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**auth_refresh_request** | [**AuthRefreshRequest**](AuthRefreshRequest.md) |  | [required] |

### Return type

[**models::AuthLoginSuccess**](AuthLoginSuccess.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
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


## auth_webauthn_authenticate_options_post

> models::WebAuthnPublicKeyOptionsResponse auth_webauthn_authenticate_options_post(web_authn_authenticate_options_request)
Start WebAuthn authentication

Returns one-shot WebAuthn authentication options for a previously enrolled device/browser. The returned challenge/options must expire within 5 minutes and must be rejected after first successful use or replay. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**web_authn_authenticate_options_request** | Option<[**WebAuthnAuthenticateOptionsRequest**](WebAuthnAuthenticateOptionsRequest.md)> |  |  |

### Return type

[**models::WebAuthnPublicKeyOptionsResponse**](WebAuthnPublicKeyOptionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_webauthn_authenticate_verify_post

> models::AuthLoginSuccess auth_webauthn_authenticate_verify_post(web_authn_authenticate_verify_request)
Verify WebAuthn authentication

Verifies a WebAuthn assertion against a still-valid one-shot authentication challenge. Successful verification consumes the challenge permanently. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**web_authn_authenticate_verify_request** | [**WebAuthnAuthenticateVerifyRequest**](WebAuthnAuthenticateVerifyRequest.md) |  | [required] |

### Return type

[**models::AuthLoginSuccess**](AuthLoginSuccess.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_webauthn_register_options_post

> models::WebAuthnPublicKeyOptionsResponse auth_webauthn_register_options_post()
Start WebAuthn device registration

Returns one-shot WebAuthn registration options for the authenticated user. The returned challenge/options must expire within 5 minutes and must be rejected after first successful use or replay. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::WebAuthnPublicKeyOptionsResponse**](WebAuthnPublicKeyOptionsResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_webauthn_register_verify_post

> models::WebAuthnDeviceResponse auth_webauthn_register_verify_post(web_authn_register_verify_request)
Verify WebAuthn device registration

Verifies a WebAuthn attestation against a still-valid one-shot registration challenge. Successful verification consumes the challenge permanently. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**web_authn_register_verify_request** | [**WebAuthnRegisterVerifyRequest**](WebAuthnRegisterVerifyRequest.md) |  | [required] |

### Return type

[**models::WebAuthnDeviceResponse**](WebAuthnDeviceResponse.md)

### Authorization

[UserBearerAuth](../README.md#UserBearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

