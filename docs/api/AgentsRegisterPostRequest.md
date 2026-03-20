# AgentsRegisterPostRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**agent_id** | **uuid::Uuid** | Stable per-agent-instance UUIDv4 generated once and persisted locally by the agent. | 
**agent_name** | **String** |  | 
**agent_version** | **String** |  | 
**openpgp_public_key** | **String** | Agent OpenPGP public key in ASCII-armored format. | 
**openpgp_fingerprint** | **String** | Canonical OpenPGP fingerprint of the active agent signing key. | 
**os_name** | **OsName** |  (enum: linux, macos, windows) | 
**os_version** | **String** |  | 
**arch** | **Arch** |  (enum: x86_64, arm64, armv7, other) | 
**capabilities** | **Vec<String>** |  | 
**client_feature_flags_contract_version** | Option<**String**> | Optional client-advertised feature-flags contract version (SemVer). | [optional]
**max_parallel_jobs** | Option<**i32**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


