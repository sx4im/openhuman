//! Argument redaction for approval prompts.
//!
//! Anything written to `pending_approvals` or broadcast on the event
//! bus must be scrubbed first — per
//! `feedback_redact_paths_and_ids_in_public.md` (no `/Users/<name>/`
//! paths, no openhuman user_ids) and `feedback_pr_no_chat_content.md`
//! (no raw message bodies, contact names, subjects, addresses — only
//! counts/shape).
//!
//! Approach: walk the JSON value tree and replace any field whose
//! name matches a known PII / chat-content key with a redacted
//! marker `"<redacted: <kind> (<n> chars)>"`. Unknown fields pass
//! through unchanged so the UI can still show useful context
//! (action slug, tool name, integration id).

use serde_json::{Map, Value};

/// Field names whose values are assumed to contain raw user content
/// or PII and MUST be redacted. Matching is case-insensitive.
const SENSITIVE_KEYS: &[&str] = &[
    "body",
    "content",
    "description",
    "plaintext",
    "text",
    "message",
    "messages",
    "coverletter",
    "note",
    "reason",
    "html",
    "html_body",
    "snippet",
    "subject",
    "title",
    "recipient",
    "recipients",
    "to",
    "cc",
    "bcc",
    "from",
    "sender",
    "address",
    "email",
    "phone",
    "contact",
    "contacts",
    "name",
    "first_name",
    "last_name",
    "full_name",
    "displayname",
    "bio",
    "avatar",
    "links",
    "tags",
    "channel_name",
    "user",
    "user_id",
    "userid",
    "username",
    "thread_id",
    "thread_ts",
    "conversation_id",
    "token",
    "api_key",
    "secret",
    "password",
    "authorization",
    "auth",
    "code",
    // File-handoff params. A presigned storage link (`storage_get_link`) is a
    // BEARER CAPABILITY: anyone holding the URL can fetch the file until it
    // expires. These land in `tool_call` args whenever a flow hands a produced
    // file to an externally-executed action (a Composio `file_uploadable`
    // param such as Gmail's `attachment` or Jira's `file_to_upload`). Redacted
    // args are both rendered on the approval card and persisted with the
    // approval record, so leaving these clear would leak the capability into
    // the UI and durable storage.
    "attachment",
    "attachments",
    "file_to_upload",
    "file_url",
    "public_url",
    "signed_url",
    "presigned_url",
];

/// Produce a redacted clone of `args` suitable for persistence /
/// broadcast / display.
pub fn redact_args(args: &Value) -> Value {
    walk(args)
}

fn walk(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(walk_object(map)),
        Value::Array(items) => Value::Array(items.iter().map(walk).collect()),
        Value::String(s) => Value::String(tinyredact::redact_home_paths(s)),
        other => other.clone(),
    }
}

fn walk_object(map: &Map<String, Value>) -> Map<String, Value> {
    let mut out = Map::with_capacity(map.len());
    for (k, v) in map {
        if is_sensitive_key(k) {
            out.insert(k.clone(), redact_value(v));
        } else {
            out.insert(k.clone(), walk(v));
        }
    }
    out
}

fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    SENSITIVE_KEYS.iter().any(|s| s == &lower.as_str())
}

fn redact_value(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            Value::String(format!("<redacted: string ({} chars)>", s.chars().count()))
        }
        Value::Array(items) => Value::String(format!("<redacted: array ({} items)>", items.len())),
        Value::Object(map) => Value::String(format!("<redacted: object ({} keys)>", map.len())),
        Value::Number(_) => Value::String("<redacted: number>".to_string()),
        Value::Bool(_) => Value::String("<redacted: bool>".to_string()),
        Value::Null => Value::Null,
    }
}

/// Build a short human-readable summary of an approval-bound tool
/// call. Pulls a handful of safe fields (`action`, `tool_slug`,
/// `integration`, etc.) and tacks on a redacted-byte-count hint so
/// the user knows *what* the agent wants to do without exposing the
/// content.
pub fn summarize_action(tool_name: &str, args: &Value) -> String {
    // Friendly, human-readable summaries for tools whose approval card reads
    // better as a sentence than a `key=value` dump (#3993). `entry_id` is a
    // public catalog slug, not PII, so it is safe to surface verbatim.
    if tool_name == "skill_registry_install" {
        if let Some(id) = args.get("entry_id").and_then(|v| v.as_str()) {
            return format!("Install the \"{id}\" skill to complete your task");
        }
    }

    let safe_fields: &[&str] = &[
        "action",
        "tool_slug",
        "action_name",
        "integration",
        "app",
        "provider",
        "channel",
        "method",
        "endpoint",
    ];
    let mut parts: Vec<String> = Vec::new();
    if let Value::Object(map) = args {
        for key in safe_fields {
            if let Some(v) = map.get(*key) {
                if let Some(s) = v.as_str() {
                    parts.push(format!("{key}={s}"));
                }
            }
        }
    }
    let bytes = serde_json::to_vec(args).map(|b| b.len()).unwrap_or(0);
    if parts.is_empty() {
        format!("{tool_name} ({bytes} bytes of arguments)")
    } else {
        format!("{tool_name}({}, {bytes} bytes)", parts.join(", "))
    }
}

#[cfg(test)]
#[path = "redact_tests.rs"]
mod tests;
