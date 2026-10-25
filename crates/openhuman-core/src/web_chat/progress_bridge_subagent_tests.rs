use super::*;

#[tokio::test]
async fn subagent_spawned_forwards_parent_call_id() {
    let mut events = super::super::super::event_bus::subscribe_web_channel_events();
    let thread_id = "thread-c1-spawned";
    let tx = spawn_test_bridge(thread_id, "req-c1-spawned");

    tx.send(AgentProgress::SubagentSpawned {
        agent_id: "researcher".into(),
        task_id: "sub-c1".into(),
        mode: "typed".into(),
        dedicated_thread: false,
        prompt_chars: 4,
        prompt: "help".into(),
        worker_thread_id: None,
        display_name: None,
        parent_call_id: Some("call-parent-1".into()),
    })
    .await
    .unwrap();

    let ev = recv_for_thread(&mut events, thread_id).await;
    assert_eq!(ev.event, "subagent_spawned");
    let subagent = ev.subagent.expect("subagent detail present");
    assert_eq!(subagent.parent_call_id.as_deref(), Some("call-parent-1"));
}

/// Terminal sub-agent events (`_completed`/`_failed`/`_awaiting_user`) must
/// keep carrying the same `parent_call_id` the spawn recorded, even though
/// those `AgentProgress` variants don't repeat it — the bridge remembers it
/// per `task_id` (#C1).
#[tokio::test]
async fn subagent_completed_carries_parent_call_id_and_capped_output() {
    let mut events = super::super::super::event_bus::subscribe_web_channel_events();
    let thread_id = "thread-c1-completed";
    let tx = spawn_test_bridge(thread_id, "req-c1-completed");

    tx.send(AgentProgress::SubagentSpawned {
        agent_id: "researcher".into(),
        task_id: "sub-c1-done".into(),
        mode: "typed".into(),
        dedicated_thread: false,
        prompt_chars: 4,
        prompt: "help".into(),
        worker_thread_id: None,
        display_name: None,
        parent_call_id: Some("call-parent-2".into()),
    })
    .await
    .unwrap();
    let spawned = recv_for_thread(&mut events, thread_id).await;
    assert_eq!(spawned.event, "subagent_spawned");

    tx.send(AgentProgress::SubagentCompleted {
        agent_id: "researcher".into(),
        task_id: "sub-c1-done".into(),
        elapsed_ms: 10,
        iterations: 1,
        output_chars: 5,
        usage: None,
        output: "final answer".into(),
        stop: None,
    })
    .await
    .unwrap();

    let completed = recv_for_thread(&mut events, thread_id).await;
    assert_eq!(completed.event, "subagent_completed");
    let subagent = completed.subagent.expect("subagent detail present");
    assert_eq!(subagent.parent_call_id.as_deref(), Some("call-parent-2"));
    assert_eq!(subagent.output.as_deref(), Some("final answer"));
}

#[tokio::test]
async fn subagent_failed_carries_parent_call_id() {
    let mut events = super::super::super::event_bus::subscribe_web_channel_events();
    let thread_id = "thread-c1-failed";
    let tx = spawn_test_bridge(thread_id, "req-c1-failed");

    tx.send(AgentProgress::SubagentSpawned {
        agent_id: "researcher".into(),
        task_id: "sub-c1-failed".into(),
        mode: "typed".into(),
        dedicated_thread: false,
        prompt_chars: 4,
        prompt: "help".into(),
        worker_thread_id: None,
        display_name: None,
        parent_call_id: Some("call-parent-3".into()),
    })
    .await
    .unwrap();
    let spawned = recv_for_thread(&mut events, thread_id).await;
    assert_eq!(spawned.event, "subagent_spawned");

    tx.send(AgentProgress::SubagentFailed {
        agent_id: "researcher".into(),
        task_id: "sub-c1-failed".into(),
        error: "boom".into(),
    })
    .await
    .unwrap();

    let failed = recv_for_thread(&mut events, thread_id).await;
    assert_eq!(failed.event, "subagent_failed");
    let subagent = failed.subagent.expect("subagent detail present");
    assert_eq!(subagent.parent_call_id.as_deref(), Some("call-parent-3"));
}

#[tokio::test]
async fn awaiting_user_checkpoint_preserves_pause_details_without_a_retired_resume_tool() {
    let tmp = tempfile::TempDir::new().expect("temporary workspace");
    let workspace = tmp.path().join("workspace");
    let action_dir = tmp.path().join("action");
    let config_path = tmp.path().join("config.toml");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    std::fs::create_dir_all(&action_dir).expect("create action directory");
    let config = crate::config::Config {
        workspace_dir: workspace.clone(),
        action_dir,
        config_path,
        ..Default::default()
    };
    let thread_id = "thread-awaiting-user-checkpoint";
    let request_id = "request-awaiting-user-checkpoint";
    let task_id = "subagent-awaiting-user-checkpoint";
    let agent_id = "workflow_builder";
    let question = "Which schedule should I use?";
    let checkpoint_path = tmp.path().join("checkpoint.json");
    let mut events = super::super::super::event_bus::subscribe_web_channel_events();
    let store = std::sync::Arc::new(tinyagents_session::port::InMemoryTurnStates::default())
        as std::sync::Arc<dyn tinyagents_session::port::TurnStates>;
    let (tx, rx) = tokio::sync::mpsc::channel(16);
    let bridge = spawn_progress_bridge(
        rx,
        "client-awaiting-user-checkpoint".into(),
        thread_id.into(),
        request_id.into(),
        store,
        ChatRequestMetadata::default(),
        config,
        crate::core::runtime::DispatchCtx::ambient(),
    );

    tx.send(AgentProgress::SubagentAwaitingUser {
        agent_id: agent_id.into(),
        task_id: task_id.into(),
        question: question.into(),
        worker_thread_id: None,
        checkpoint_path: Some(checkpoint_path.to_string_lossy().into_owned()),
    })
    .await
    .expect("send awaiting-user progress");

    let event = recv_for_thread(&mut events, thread_id).await;
    assert_eq!(event.event, "subagent_awaiting_user");
    assert_eq!(event.request_id, request_id);
    assert_eq!(event.skill_id.as_deref(), Some(task_id));
    assert_eq!(event.tool_name.as_deref(), Some(agent_id));
    assert_eq!(event.message.as_deref(), Some(question));

    let run = crate::storage::sessions::with_ledger_async(&workspace, |ledger| async move {
        ledger.get_agent_run(task_id).await
    })
    .await
    .expect("read persisted run")
    .expect("awaiting-user run was persisted before event publication");
    assert_eq!(run.id, task_id);
    assert_eq!(
        run.status,
        tinyagents_session::run_ledger::AgentRunStatus::AwaitingUser
    );
    assert_eq!(run.agent_id.as_deref(), Some(agent_id));
    assert_eq!(run.parent_run_id.as_deref(), Some(request_id));
    assert_eq!(run.parent_thread_id.as_deref(), Some(thread_id));
    assert_eq!(run.summary.as_deref(), Some(question));
    assert_eq!(
        run.checkpoint_path.as_deref(),
        Some(checkpoint_path.to_string_lossy().as_ref())
    );
    let checkpoint = run.checkpoint.expect("awaiting-user checkpoint");
    assert_eq!(checkpoint["taskId"], task_id);
    assert_eq!(checkpoint["agentId"], agent_id);
    assert_eq!(checkpoint["question"], question);
    assert_eq!(checkpoint["checkpointPersisted"], true);
    assert!(checkpoint.get("resumeTool").is_none());

    drop(tx);
    assert!(bridge.wait_drained(std::time::Duration::from_secs(5)).await);
}
