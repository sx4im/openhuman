use super::*;
use tiny_artifacts::{ArtifactRecord, ArtifactVersion, PublishOptions, PublishedLink};

struct FixedPublisher;

#[async_trait::async_trait]
impl ArtifactPublisher for FixedPublisher {
    async fn publish(
        &self,
        _record: &ArtifactRecord,
        _version: &ArtifactVersion,
        _options: &PublishOptions,
    ) -> anyhow::Result<PublishedLink> {
        anyhow::bail!("not used")
    }
}

#[test]
fn nothing_registered_offers_no_publisher() {
    assert!(artifact_publisher(&Config::default()).is_none());
}

#[test]
fn the_first_factory_that_answers_serves_sharing() {
    register_artifact_publisher("declines", |_| None);
    assert!(artifact_publisher(&Config::default()).is_none());
    let binder: ArtifactPublisherBinder = Arc::new(|_files| Arc::new(FixedPublisher));
    register_artifact_publisher("answers", move |_| Some(binder.clone()));
    assert!(artifact_publisher(&Config::default()).is_some());
    assert!(unregister_artifact_publisher("answers"));
    assert!(artifact_publisher(&Config::default()).is_none());
}

#[tokio::test]
async fn publisher_lookup_uses_the_carried_context_not_the_ambient_profile() {
    use crate::core::runtime::{ContextOverlay, CoreContext, DispatchCtx, DomainSet};
    use crate::tools::toolpacks::ToolGroups;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let overlay = |profile: &'static str| {
        ContextOverlay::new(Config::default(), DomainSet::kernel(), ToolGroups::none())
            .profile(profile)
    };
    // Derived contexts share their parent's extension table, so start each
    // test context from its own root to model separate host registrations.
    let core_a = CoreContext::for_test(DomainSet::full(), None).derive_with(overlay("publisher-a"));
    let core_b = CoreContext::for_test(DomainSet::full(), None).derive_with(overlay("publisher-b"));
    let ctx_a = DispatchCtx::new(core_a.clone());
    let ctx_b = DispatchCtx::new(core_b.clone());
    let binder: ArtifactPublisherBinder = Arc::new(|_files| Arc::new(FixedPublisher));
    let selected_a = Arc::new(AtomicUsize::new(0));
    let selected_b = Arc::new(AtomicUsize::new(0));
    let factory_a = selected_a.clone();
    register_artifact_publisher_in(core_a.extensions(), "publisher-context-a", move |_| {
        factory_a.fetch_add(1, Ordering::SeqCst);
        Some(binder.clone())
    });
    let factory_b = selected_b.clone();
    register_artifact_publisher_in(core_b.extensions(), "publisher-context-b", move |_| {
        factory_b.fetch_add(1, Ordering::SeqCst);
        Some(Arc::new(|_files| Arc::new(FixedPublisher)))
    });

    CoreContext::scope(core_b, async {
        assert!(artifact_publisher_in(&Config::default(), &ctx_a).is_some());
        assert_eq!(selected_a.load(Ordering::SeqCst), 1);
        assert_eq!(selected_b.load(Ordering::SeqCst), 0);

        assert!(artifact_publisher_in(&Config::default(), &ctx_b).is_some());
        assert_eq!(selected_a.load(Ordering::SeqCst), 1);
        assert_eq!(selected_b.load(Ordering::SeqCst), 1);
    })
    .await;
}
