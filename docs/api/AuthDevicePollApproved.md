# AuthDevicePollApproved

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | **Status** |  (enum: APPROVED) | 
**client_id** | **String** |  | 
**client_kind** | [**models::NonUiClientKind**](NonUiClientKind.md) |  | 
**secret_key** | **String** | One-shot credential shown only once after approval. | 
**approved_at** | **String** | Timestamp of the human approval performed in UI_WEB. | 
**approved_by_user_id** | **uuid::Uuid** | Audit identifier of the human approver in UI_WEB. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


