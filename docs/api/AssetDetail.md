# AssetDetail

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**summary** | [**models::AssetSummary**](AssetSummary.md) |  | 
**notes** | Option<**String**> |  | [optional]
**fields** | Option<[**std::collections::HashMap<String, models::AssetFieldValue>**](AssetFieldValue.md)> | Shared complementary metadata map. Values stay JSON-simple; domains requiring dedicated semantics, storage or security rules must not be hidden implicitly here.  | [optional]
**gps_latitude** | Option<**f64**> |  | [optional]
**gps_longitude** | Option<**f64**> |  | [optional]
**gps_altitude_m** | Option<**f64**> |  | [optional]
**gps_altitude_relative_m** | Option<**f64**> |  | [optional]
**gps_altitude_absolute_m** | Option<**f64**> |  | [optional]
**location_country** | Option<**String**> |  | [optional]
**location_city** | Option<**String**> |  | [optional]
**location_label** | Option<**String**> |  | [optional]
**paths** | [**models::AssetPaths**](AssetPaths.md) |  | 
**processing** | [**models::AssetProcessing**](AssetProcessing.md) |  | 
**derived** | [**models::AssetDerived**](AssetDerived.md) |  | 
**transcript** | Option<[**models::AssetTranscript**](AssetTranscript.md)> | Pre-release field only. Outside v1 conformance scope; may be exposed earlier under feature flags before validated v1.1+ rollout. | [optional]
**decisions** | [**models::AssetDecisions**](AssetDecisions.md) |  | 
**audit** | [**models::AssetAudit**](AssetAudit.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


