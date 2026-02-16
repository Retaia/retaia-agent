# UserFeaturesResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**user_feature_enabled** | **std::collections::HashMap<String, bool>** | User-level feature switches. Only non-core features can be disabled at user scope. Missing key is interpreted as `true` (migration-safe default).  | 
**effective_feature_enabled** | **std::collections::HashMap<String, bool>** | Effective availability after all gates (`feature_flags`, `app_feature_enabled`, `user_feature_enabled`, dependencies). Evaluation order is strict: feature_flags -> app_feature_enabled -> user_feature_enabled -> dependency/escalation rules.  | 
**feature_governance** | [**Vec<models::FeatureGovernanceRule>**](FeatureGovernanceRule.md) |  | 
**core_v1_global_features** | [**HashSet<models::CoreV1GlobalFeatureKey>**](CoreV1GlobalFeatureKey.md) | Canonical list of non-disableable v1 global core feature keys. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


