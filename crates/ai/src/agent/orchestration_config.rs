use super::action::RunAgentsRequest;

/// Client-side representation of the orchestration config attached to a
/// conversation via `OrchestrationConfigSnapshot`.
///
/// Originally mirrored the proto `OrchestrationConfig` from the proprietary
/// hosted-agent backend. The proto layer was removed during cloud detach;
/// this struct is preserved for the existing UI/model code that references
/// it but is no longer wire-serialized.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrchestrationConfig {
    pub model_id: String,
    pub harness_type: String,
    pub execution_mode: OrchestrationExecutionMode,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OrchestrationExecutionMode {
    Local,
    Remote {
        environment_id: String,
        worker_host: String,
    },
}

impl OrchestrationExecutionMode {
    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Remote { .. })
    }
}

/// User's approval state for orchestration on the active config.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum OrchestrationConfigStatus {
    /// No `OrchestrationConfigSnapshot` has been seen yet.
    #[default]
    None,
    Approved,
    Disapproved,
}

impl OrchestrationConfigStatus {
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved)
    }

    pub fn is_disapproved(&self) -> bool {
        matches!(self, Self::Disapproved)
    }
}

/// Returns `true` when the `run_agents` call's run-wide fields match
/// the active approved `OrchestrationConfig`, meaning the confirmation
/// card can be skipped (auto-launch).
pub fn matches_active_config(request: &RunAgentsRequest, config: &OrchestrationConfig) -> bool {
    if !request.model_id.is_empty() && request.model_id != config.model_id {
        return false;
    }

    if !request.harness_type.is_empty() && request.harness_type != config.harness_type {
        return false;
    }

    match (&request.execution_mode, &config.execution_mode) {
        (super::action::RunAgentsExecutionMode::Local, OrchestrationExecutionMode::Local) => true,
        (
            super::action::RunAgentsExecutionMode::Remote {
                environment_id,
                worker_host,
                ..
            },
            OrchestrationExecutionMode::Remote {
                environment_id: cfg_env,
                worker_host: cfg_host,
            },
        ) => {
            let env_matches = environment_id.is_empty() || environment_id == cfg_env;
            let host_matches = worker_host.is_empty() || worker_host == cfg_host;
            env_matches && host_matches
        }
        _ => false,
    }
}
