# FeatureGovernanceRule

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**key** | **String** | Feature key. Keys listed in `core_v1_global_features` are protected. | 
**tier** | **Tier** |  (enum: CORE_V1_GLOBAL, OPTIONAL) | 
**user_can_disable** | **bool** |  | 
**dependencies** | **Vec<String>** |  | 
**disable_escalation** | **Vec<String>** | Features automatically disabled when this feature is disabled at app/user scope.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


