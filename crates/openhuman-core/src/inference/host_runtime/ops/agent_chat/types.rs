use super::*;

/// What an in-process caller hands one [`agent_chat_for`] turn besides its
/// request: the live progress receiver and the host environment (shared model
/// budget, delegation depth, cancellation, deadline, hooks, tool-belt
/// authority). Both are explicit values that travel with the call -- never an
/// ambient scope -- so they reach exactly this turn and its child runs.
#[derive(Clone, Default)]
pub struct AgentChatEnv {
    /// Live progress for the turn; `None` attaches a drain-only sink so the
    /// model calls still stream.
    pub progress: Option<tokio::sync::mpsc::Sender<crate::progress::AgentProgress>>,
    /// The host environment installed on the session
    /// ([`OpenHumanSessionHost::set_host_env`]).
    pub host: crate::agent::turn_env::HostTurnEnv,
    /// Messages to steer the turn while it runs
    /// ([`OpenHumanSessionHost::set_run_queue`]). Items pushed on its
    /// [`QueueLane::Steer`](tinyagents_harness::run_queue::QueueLane::Steer)
    /// lane before or during the turn are injected as user messages at the
    /// loop's next safe checkpoint; a steer that arrives while the model is
    /// writing its final answer earns one more model call. Steers the turn
    /// did not apply are back on the lane once this call returns.
    pub run_queue: Option<
        std::sync::Arc<
            tinyagents_harness::run_queue::RunQueue<crate::agent::queued_turn::QueuedTurn>,
        >,
    >,
}

/// The log line every successful agent chat turn carries.
pub const AGENT_CHAT_COMPLETED_LOG: &str = "agent chat completed";

/// What one agent chat turn produced: its reply and whether it stopped at the
/// tool-iteration cap rather than finishing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChatReply {
    /// The reply text. When `hit_cap` is set this is the turn's checkpoint.
    pub text: String,
    /// The turn paused at its iteration cap (see
    /// [`OpenHumanSessionHost::last_turn_hit_cap`]).
    pub hit_cap: bool,
}

impl AgentChatReply {
    /// The historical `Outcome<String>` shape, for library callers.
    pub fn into_outcome(self) -> Outcome<String> {
        Outcome::single_log(self.text, AGENT_CHAT_COMPLETED_LOG)
    }

    /// The `inference.agent_chat` wire shape: the historical
    /// `{"result": <text>, "logs": [...]}` envelope plus `hit_cap`, and, when
    /// the turn was capped, `checkpoint` (the same text, named for what it is).
    ///
    /// Additive on purpose: existing clients keep reading the reply from
    /// `result` (directly or through `unwrap_rpc`), and a headless caller can
    /// now tell a capped turn from a finished one and continue it (#6958).
    pub fn into_rpc_json(self) -> Result<serde_json::Value, String> {
        let hit_cap = self.hit_cap;
        let checkpoint = hit_cap.then(|| self.text.clone());
        let mut value = self.into_outcome().into_cli_compatible_json()?;
        let serde_json::Value::Object(map) = &mut value else {
            return Err("agent chat envelope is not a JSON object".to_string());
        };
        map.insert("hit_cap".to_string(), serde_json::Value::Bool(hit_cap));
        if let Some(checkpoint) = checkpoint {
            map.insert(
                "checkpoint".to_string(),
                serde_json::Value::String(checkpoint),
            );
        }
        Ok(value)
    }
}

/// Which session [`agent_chat_for`] builds the turn on.
#[derive(Clone, Copy)]
pub enum AgentChatTarget<'a> {
    /// The orchestrator — [`OpenHumanSessionHost::from_config`], today's `agent_chat`.
    Orchestrator,
    /// Resolve `id` the way every other id-keyed entry point does: the
    /// process registry first, then `config.agent_registry.entries`.
    AgentId(&'a str),
    /// A definition the caller already holds; nothing is resolved by id. The
    /// entry point for a library host running its own per-agent specs — see
    /// [`OpenHumanSessionHost::from_config_with_definition`].
    ///
    /// `host` carries the caller's own `dyn Tool` objects. It rides the target
    /// rather than `agent_chat_for`'s argument list because only this target
    /// can honour it: the other two resolve a definition the caller does not
    /// hold, so there is no agent for a host belt to belong to.
    Definition {
        definition: &'a crate::agent_registry::host_definition::AgentDefinition,
        host: Option<&'a crate::agent::HostTools>,
        /// History to seed this turn with, as `(role, content)` rows, instead
        /// of whatever the session would otherwise resume.
        ///
        /// Rides the target for the same reason `host` does, and it is the
        /// per-turn half of the same idea: a host whose history lives in its
        /// own log -- a journal, a board, an episode -- is the only thing that
        /// can say what this turn should have seen. Seeding is how that view
        /// reaches the session with roles intact; passing it as prose in the
        /// message would flatten the host's own prior turns into quoted text.
        ///
        /// `None` leaves resume untouched, which is every existing caller.
        seed: Option<&'a [(String, String)]>,
        /// Where to report what the turn spent.
        ///
        /// An out-parameter because this is the only target that can fill it:
        /// the turn runs in-process here, so the session that counted the
        /// tokens is still in hand when it ends. The other two answer over
        /// `AGENT_CHAT`, whose reply is a string.
        ///
        /// Written whether the turn succeeded or failed -- a turn that ended
        /// badly still spent what it spent, and a host that meters only
        /// successes bills nothing for the ones that cost most.
        usage: Option<&'a std::sync::Mutex<Option<crate::agent::tinyagents::host::LastTurnUsage>>>,
        /// Build the session from `host` alone: no config-derived, delegation,
        /// memory, skill or MCP tool, and a deny-by-default gate for any other
        /// name. See [`OpenHumanSessionHost::from_config_host_only`].
        host_only: bool,
        /// Skip the prompt-injection guard: the message is untrusted data (a
        /// PR diff) the agent reads, not an instruction. Refused unless
        /// `host_only`, whose agent has nothing it could be talked into doing.
        untrusted_input: bool,
        /// A response format and output cap for every model call of the turn,
        /// and the slot its final call is reported into.
        shape: Option<&'a std::sync::Arc<tinyagents_harness::middleware::ResponseShapeScope>>,
        /// The observer scoped around the caller, re-established around the
        /// turn because the dispatch may run on a worker task that does not
        /// inherit task locals.
        observer: Option<
            &'a std::sync::Arc<tinyagents_harness::observability::turn_observer::ObserverScope>,
        >,
        /// Files the host attaches to this turn as bytes, in order.
        ///
        /// Rides the target for the same reason `host` does: only an
        /// in-process caller can hand over bytes, and `AGENT_CHAT` carries a
        /// string. Each is staged into the acting workspace and reaches the
        /// model as an image or document block ahead of the message, under
        /// the configured multimodal count and size limits. No local path is
        /// read, so they are accepted whatever the turn's origin.
        ///
        /// Empty for every caller that attaches nothing.
        attachments: &'a [crate::agent::HostAttachment],
    },
}

// Hand-written because a host belt is a closure, and a closure is not `Debug`.
// Reporting whether one is present is what a log line here is ever for; the
// belt it would return is not known until the turn builds it.
impl std::fmt::Debug for AgentChatTarget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Orchestrator => f.write_str("Orchestrator"),
            Self::AgentId(id) => f.debug_tuple("AgentId").field(id).finish(),
            Self::Definition {
                definition,
                host,
                seed,
                usage,
                host_only,
                untrusted_input,
                shape,
                observer,
                attachments,
            } => f
                .debug_struct("Definition")
                .field("definition", &definition.id)
                .field("host_tools", &host.is_some())
                .field("seed_rows", &seed.map_or(0, <[(String, String)]>::len))
                .field("meters", &usage.is_some())
                .field("host_only", host_only)
                .field("untrusted_input", untrusted_input)
                .field("attachments", &attachments.len())
                .field("shaped", &shape.is_some())
                .field("observed", &observer.is_some())
                .finish(),
        }
    }
}
