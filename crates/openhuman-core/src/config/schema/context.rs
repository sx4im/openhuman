//! TinyAgents-owned context settings re-exported through OpenHuman's config API.
//!
//! OpenHuman keeps its product-specific environment overlays in
//! [`super::load`], while the shared settings, defaults, parsing, and
//! serialization contract live with TinyAgents.

pub use tinyagents_harness::config::{CompactionSettings, CompactionStrategy, ContextConfig};

/// JSON-schema view of TinyAgents' serialized context settings.
///
/// The harness type deliberately stays independent from `schemars`; this
/// mirror keeps OpenHuman's generated config schema complete without adding a
/// documentation dependency to the owning crate.
#[derive(schemars::JsonSchema)]
pub(crate) struct ContextConfigSchema {
    enabled: bool,
    microcompact_enabled: bool,
    autocompact_enabled: bool,
    microcompact_keep_recent: usize,
    tool_result_budget_bytes: usize,
    summarizer_payload_threshold_tokens: usize,
    summarizer_max_payload_tokens: usize,
    summarizer_model: Option<String>,
    prefer_markdown_tool_output: bool,
    compaction_enabled: bool,
    compaction_trigger_tokens: Option<u64>,
    compaction_strategy: CompactionStrategySchema,
}

#[derive(schemars::JsonSchema)]
#[schemars(rename_all = "snake_case")]
enum CompactionStrategySchema {
    TaskState,
    Summary,
}
