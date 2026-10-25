//! Host budget policy, handed to the turn once at admission and carried
//! explicitly to children (`HostTurnEnv::model_budget` -> `OpenHumanRunContext`).

use std::sync::Arc;

/// Product recursion ceiling shared by native and MCP delegation.
pub(crate) const MAX_SPAWN_DEPTH: usize = 3;
pub use tinyagents_harness::cost::{
    ControlledDispatchError, DispatchModelOptions, DispatchRefusal, DispatchRefusalReason,
    PhysicalDispatchAdmission, PhysicalDispatchPermit,
};
pub use tinyinference_llm::model::budget::{
    Budget, BudgetExceeded, BudgetSnapshot, CallBudget, Spend, SpendLimits,
};

/// Effective host depth ceiling, never wider than product policy.
pub(crate) fn depth(context: &super::host::OpenHumanRunContext) -> usize {
    context
        .max_spawn_depth
        .unwrap_or(crate::agent::tinyagents::budget::MAX_SPAWN_DEPTH)
        .min(crate::agent::tinyagents::budget::MAX_SPAWN_DEPTH)
}
pub(super) fn install_depth(
    harness: &mut tinyagents_harness::runtime::AgentHarness<(), super::host::OpenHumanRunContext>,
    context: &super::host::OpenHumanRunContext,
) {
    if let Some(max_depth) = context.max_spawn_depth {
        let mut policy = harness.policy().clone();
        policy.limits.max_depth = policy.limits.max_depth.min(max_depth);
        harness.with_policy(policy);
    }
}

impl crate::inference::turn_models::TurnModels {
    /// Wrap every concrete route and summarizer below harness retries/fallback.
    pub(crate) fn with_host_controls(
        mut self,
        budget: Option<&tinyagents_harness::cost::ModelBudget>,
        admission: Option<Arc<dyn PhysicalDispatchAdmission>>,
    ) -> Self {
        if budget.is_none() && admission.is_none() {
            return self;
        }
        // A native turn set owns the resolver and error slots, so the
        // decorator must be applied inside it, not only to the copies held here.
        if let Some(models) = self.native_model_set.take() {
            let models =
                models.map_models(|_, model| control_model(model, budget, admission.clone()));
            let mut registrations = models.registrations("primary").into_iter();
            self.primary = registrations
                .next()
                .expect("native turn set contains its primary")
                .1;
            self.routes = registrations.collect();
            self.summarizer = models.summarizer();
            self.error_slot = models.primary_error_slot();
            self.native_model_set = Some(models);
            return self;
        }
        self.primary = control_model(self.primary, budget, admission.clone());
        self.summarizer = control_model(self.summarizer, budget, admission.clone());
        for (_, route) in &mut self.routes {
            *route = control_model(route.clone(), budget, admission.clone());
        }
        self
    }
}

fn control_model(
    model: Arc<dyn tinyinference_llm::model::ChatModel<()>>,
    budget: Option<&tinyagents_harness::cost::ModelBudget>,
    admission: Option<Arc<dyn PhysicalDispatchAdmission>>,
) -> Arc<dyn tinyinference_llm::model::ChatModel<()>> {
    let monetary_budget = budget.map(|budget| {
        tinyagents_harness::cost::ModelBudget::new(budget.ledger.clone(), budget.call)
    });
    Arc::new(tinyagents_harness::cost::ControlledChatModel::new(
        model,
        DispatchModelOptions::new()
            .monetary_budget(monetary_budget)
            .admission(admission),
    ))
}
