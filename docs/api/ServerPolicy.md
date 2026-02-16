# ServerPolicy

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**min_poll_interval_seconds** | Option<**i32**> |  | [optional]
**max_parallel_jobs_allowed** | Option<**i32**> |  | [optional]
**allowed_job_types** | Option<**Vec<String>**> |  | [optional]
**quiet_hours** | Option<**std::collections::HashMap<String, serde_json::Value>**> |  | [optional]
**feature_flags** | [**models::FeatureFlags**](FeatureFlags.md) |  | 
**feature_flags_contract_version** | **String** | Latest feature-flags contract version served by Core (SemVer). | 
**accepted_feature_flags_contract_versions** | **Vec<String>** | Feature-flags contract versions still accepted by Core for compatibility. | 
**effective_feature_flags_contract_version** | **String** | Feature-flags contract version effectively served to this client request (SemVer). | 
**feature_flags_compatibility_mode** | **FeatureFlagsCompatibilityMode** | STRICT = latest contract served. COMPAT = compatibility profile served (including retired-flag tombstones when required).  (enum: STRICT, COMPAT) | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


