# AppFeaturesResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**app_feature_enabled** | **std::collections::HashMap<String, bool>** | Application-level feature switches. Effective feature availability requires Core `FeatureFlags` AND `AppFeatureEnabled`. `features.ai` controls MCP global availability (false => MCP disabled).  | 
**core_v1_global_features** | [**HashSet<models::CoreV1GlobalFeatureKey>**](CoreV1GlobalFeatureKey.md) | Canonical list of non-disableable v1 global core feature keys. | 
**feature_governance** | [**Vec<models::FeatureGovernanceRule>**](FeatureGovernanceRule.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


