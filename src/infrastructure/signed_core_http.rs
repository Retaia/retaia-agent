use chrono::Utc;
use reqwest::blocking::{Body, Client, RequestBuilder};
use reqwest::header::{ACCEPT_LANGUAGE, AUTHORIZATION, CONTENT_TYPE, HeaderValue, IF_MATCH};
use reqwest::{Method, Url};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::infrastructure::agent_identity::{AgentIdentity, AgentIdentityError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedRequest {
    pub timestamp: String,
    pub nonce: String,
    pub signature: String,
}

#[derive(Debug, Error)]
pub enum SignedCoreHttpError {
    #[error("invalid url: {0}")]
    Url(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("agent identity error: {0}")]
    AgentIdentity(#[from] AgentIdentityError),
}

pub fn json_bytes<T: Serialize>(payload: &T) -> Result<Vec<u8>, SignedCoreHttpError> {
    serde_json::to_vec(payload).map_err(Into::into)
}

pub fn signature_payload(
    method: Method,
    path: &str,
    agent_id: &str,
    timestamp: &str,
    nonce: &str,
    body: &[u8],
) -> String {
    let body_hash = hex::encode(Sha256::digest(body));
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method.as_str(),
        path,
        agent_id,
        timestamp,
        nonce,
        body_hash
    )
}

pub fn signed_request(
    identity: &AgentIdentity,
    method: Method,
    path: &str,
    body: &[u8],
) -> Result<SignedRequest, SignedCoreHttpError> {
    let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let nonce = uuid::Uuid::new_v4().to_string();
    let payload = signature_payload(method, path, &identity.agent_id, &timestamp, &nonce, body);
    let signature = identity.detached_signature_ascii_armored(payload.as_bytes())?;
    Ok(SignedRequest {
        timestamp,
        nonce,
        signature,
    })
}

pub fn absolute_url(base_path: &str, relative_path: &str) -> Result<Url, SignedCoreHttpError> {
    let base = Url::parse(&format!("{}/", base_path.trim_end_matches('/')))
        .map_err(|error| SignedCoreHttpError::Url(error.to_string()))?;
    base.join(relative_path.trim_start_matches('/'))
        .map_err(|error| SignedCoreHttpError::Url(error.to_string()))
}

pub fn apply_signed_headers(
    builder: RequestBuilder,
    identity: &AgentIdentity,
    signed: &SignedRequest,
    bearer_token: Option<&str>,
    accept_language: Option<&str>,
) -> RequestBuilder {
    let mut builder = builder
        .header("X-Retaia-Agent-Id", identity.agent_id.clone())
        .header(
            "X-Retaia-OpenPGP-Fingerprint",
            identity.openpgp_fingerprint.clone(),
        )
        .header("X-Retaia-Signature", signed.signature.clone())
        .header("X-Retaia-Signature-Timestamp", signed.timestamp.clone())
        .header("X-Retaia-Signature-Nonce", signed.nonce.clone());

    if let Some(token) = bearer_token {
        builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    if let Some(language) = accept_language {
        builder = builder.header(ACCEPT_LANGUAGE, language);
    }
    builder
}

pub fn signed_json_request(
    client: &Client,
    identity: &AgentIdentity,
    bearer_token: Option<&str>,
    base_path: &str,
    method: Method,
    relative_path: &str,
    payload: &[u8],
    accept_language: Option<&str>,
) -> Result<RequestBuilder, SignedCoreHttpError> {
    let url = absolute_url(base_path, relative_path)?;
    let signed = signed_request(identity, method.clone(), url.path(), payload)?;
    let builder = client
        .request(method, url)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(payload.to_vec());
    Ok(apply_signed_headers(
        builder,
        identity,
        &signed,
        bearer_token,
        accept_language,
    ))
}

pub fn signed_empty_request(
    client: &Client,
    identity: &AgentIdentity,
    bearer_token: Option<&str>,
    base_path: &str,
    method: Method,
    relative_path: &str,
    accept_language: Option<&str>,
) -> Result<RequestBuilder, SignedCoreHttpError> {
    let empty = Vec::new();
    signed_json_request(
        client,
        identity,
        bearer_token,
        base_path,
        method,
        relative_path,
        &empty,
        accept_language,
    )
}

pub fn multipart_part_request(
    client: &Client,
    identity: &AgentIdentity,
    bearer_token: Option<&str>,
    base_path: &str,
    relative_path: &str,
    if_match: &str,
    upload_id: &str,
    part_number: u32,
    chunk: Vec<u8>,
    accept_language: Option<&str>,
) -> Result<RequestBuilder, SignedCoreHttpError> {
    let boundary = format!("retaia-agent-{}", uuid::Uuid::new_v4());
    let mut body = Vec::new();
    write_multipart_field(&mut body, &boundary, "upload_id", upload_id.as_bytes());
    write_multipart_field(
        &mut body,
        &boundary,
        "part_number",
        part_number.to_string().as_bytes(),
    );
    write_multipart_file(&mut body, &boundary, "chunk", "chunk.bin", &chunk);
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    let url = absolute_url(base_path, relative_path)?;
    let signed = signed_request(identity, Method::POST, url.path(), &body)?;
    let content_type = format!("multipart/form-data; boundary={boundary}");
    let mut builder = client
        .post(url)
        .header(CONTENT_TYPE, content_type)
        .header(IF_MATCH, if_match)
        .body(Body::from(body));
    if let Some(language) = accept_language {
        builder = builder.header(ACCEPT_LANGUAGE, language);
    }
    Ok(apply_signed_headers(
        builder,
        identity,
        &signed,
        bearer_token,
        None,
    ))
}

fn write_multipart_field(buf: &mut Vec<u8>, boundary: &str, name: &str, value: &[u8]) {
    buf.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    buf.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
    );
    buf.extend_from_slice(value);
    buf.extend_from_slice(b"\r\n");
}

fn write_multipart_file(
    buf: &mut Vec<u8>,
    boundary: &str,
    name: &str,
    filename: &str,
    value: &[u8],
) {
    buf.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    buf.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n"
        )
        .as_bytes(),
    );
    buf.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    buf.extend_from_slice(value);
    buf.extend_from_slice(b"\r\n");
}

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::signature_payload;

    #[test]
    fn tdd_signature_payload_uses_exact_contract_shape() {
        let payload = signature_payload(
            Method::POST,
            "/api/v1/jobs/job-1/claim",
            "agent-1",
            "2026-03-19T12:00:00Z",
            "nonce-1",
            br#"{"ok":true}"#,
        );
        assert_eq!(payload.lines().count(), 6);
        assert!(payload.contains("/api/v1/jobs/job-1/claim"));
        assert!(payload.ends_with("4062edaf750fb8074e7e83e0c9028c94e32468a8b6f1614774328ef045150f93"));
    }
}
