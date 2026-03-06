use axum::response::sse::Event;
use chrono::Utc;
use serde_json::{Value, json};

pub(crate) const SSE_SCHEMA_VERSION: u8 = 1;

pub(crate) fn sync_event(
    stream: &str,
    entity: &str,
    key: &str,
    sync_id: &str,
    seq: u64,
    schema: &str,
    snapshot: Value,
) -> Event {
    Event::default().event("sync").data(
        json!({
            "version": SSE_SCHEMA_VERSION,
            "stream": stream,
            "entity": entity,
            "key": key,
            "sync_id": sync_id,
            "seq": seq,
            "schema": schema,
            "server_unix_ms": Utc::now().timestamp_millis(),
            "snapshot": snapshot,
        })
        .to_string(),
    )
}

pub(crate) fn delta_event(
    stream: &str,
    entity: &str,
    key: &str,
    sync_id: &str,
    seq: u64,
    schema: &str,
    ops: Vec<Value>,
) -> Event {
    Event::default().event("delta").data(
        json!({
            "version": SSE_SCHEMA_VERSION,
            "stream": stream,
            "entity": entity,
            "key": key,
            "sync_id": sync_id,
            "seq": seq,
            "schema": schema,
            "server_unix_ms": Utc::now().timestamp_millis(),
            "ops": ops,
        })
        .to_string(),
    )
}

pub(crate) fn error_event(message: &str) -> Event {
    Event::default().event("error").data(
        json!({
            "version": SSE_SCHEMA_VERSION,
            "server_unix_ms": Utc::now().timestamp_millis(),
            "message": message,
        })
        .to_string(),
    )
}

pub(crate) fn op_upsert(path: &str, value: Value) -> Value {
    json!({
        "op": "upsert",
        "path": path,
        "value": value,
    })
}

pub(crate) fn op_append(path: &str, value: Value) -> Value {
    json!({
        "op": "append",
        "path": path,
        "value": value,
    })
}
