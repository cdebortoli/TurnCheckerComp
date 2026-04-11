use axum::body::to_bytes;
use axum::http::StatusCode;
use uuid::Uuid;

use super::parse_json_request;
use crate::server::dto::{SyncConnectRequest, SyncPushRequest};

#[tokio::test]
async fn json_rejection_includes_unknown_top_level_field() {
    let response = parse_json_request(
        "/sync/connect",
        axum::Json::<SyncConnectRequest>::from_bytes(
            br#"{"deviceId":"ios","unexpected":true}"#,
        ),
    )
    .unwrap_err();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("unknown field `unexpected`"));
}

#[tokio::test]
async fn json_rejection_includes_nested_field_path() {
    let payload = format!(
        concat!(
            "{{",
            "\"deviceId\":\"ios\",",
            "\"checks\":[{{",
            "\"uuid\":\"{}\",",
            "\"name\":\"Scout\",",
            "\"detail\":null,",
            "\"source\":\"Game\",",
            "\"repeatCase\":\"Everytime\",",
            "\"tagUuid\":null,",
            "\"position\":0,",
            "\"isMandatory\":false,",
            "\"isChecked\":false,",
            "\"isSent\":false,",
            "\"unexpected\":true",
            "}}],",
            "\"comments\":[],",
            "\"tags\":[]",
            "}}"
        ),
        Uuid::new_v4()
    );

    let response = parse_json_request(
        "/sync/push",
        axum::Json::<SyncPushRequest>::from_bytes(payload.as_bytes()),
    )
    .unwrap_err();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("checks[0]"));
    assert!(body.contains("unknown field `unexpected`"));
}
