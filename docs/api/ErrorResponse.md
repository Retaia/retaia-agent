# ErrorResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**code** | **Code** |  (enum: UNAUTHORIZED, EMAIL_NOT_VERIFIED, FORBIDDEN_SCOPE, FORBIDDEN_ACTOR, USER_NOT_FOUND, STATE_CONFLICT, IDEMPOTENCY_CONFLICT, STALE_LOCK_TOKEN, NAME_COLLISION_EXHAUSTED, PURGED, VALIDATION_FAILED, INVALID_TOKEN, LOCK_REQUIRED, LOCK_INVALID, TOO_MANY_ATTEMPTS, MFA_REQUIRED, INVALID_2FA_CODE, MFA_ALREADY_ENABLED, MFA_NOT_ENABLED, INVALID_DEVICE_CODE, EXPIRED_DEVICE_CODE, UNSUPPORTED_FEATURE_FLAGS_CONTRACT_VERSION, SLOW_DOWN, RATE_LIMITED, TEMPORARY_UNAVAILABLE) | 
**message** | **String** |  | 
**details** | Option<**std::collections::HashMap<String, serde_json::Value>**> |  | [optional]
**retryable** | **bool** |  | 
**correlation_id** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


