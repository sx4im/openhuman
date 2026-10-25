//! Internal entry point for agent chat turns that inherit a resolved model source.

use super::agent_chat::{
    agent_chat_reply_for_inner, AgentChatEnv, AgentChatReply, AgentChatTarget,
};
use crate::config::Config;

#[allow(clippy::too_many_arguments)]
pub(crate) fn agent_chat_for_with_turn_model_source<'a>(
    config: &'a mut Config,
    target: AgentChatTarget<'a>,
    message: &'a str,
    model_override: Option<String>,
    temperature: Option<f64>,
    thread_id: Option<String>,
    cwd: Option<String>,
    route: Option<crate::config::schema::EphemeralRoute>,
    env: AgentChatEnv,
    turn_model_source: (crate::agent::tinyagents::TurnModelSource, String),
) -> futures::future::BoxFuture<'a, Result<crate::core::Outcome<String>, String>> {
    Box::pin(async move {
        agent_chat_reply_for_inner(
            config,
            target,
            message,
            model_override,
            temperature,
            thread_id,
            cwd,
            route,
            env,
            Some(turn_model_source),
            false,
        )
        .await
        .map(AgentChatReply::into_outcome)
    })
}
