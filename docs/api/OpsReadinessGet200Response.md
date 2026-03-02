# OpsReadinessGet200Response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | **Status** | Overall status (`down` when database check fails; `degraded` when critical non-database checks fail but auto-repair is active). (enum: ok, degraded, down) | 
**self_healing** | [**models::OpsReadinessGet200ResponseSelfHealing**](OpsReadinessGet200ResponseSelfHealing.md) |  | 
**checks** | [**Vec<models::OpsReadinessGet200ResponseChecksInner>**](OpsReadinessGet200ResponseChecksInner.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


