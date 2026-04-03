use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct AppState {
    registry: Mutex<RegistryState>,
}

#[derive(Debug, Default)]
struct RegistryState {
    workspace_paths: Vec<String>,
    command_log: Vec<String>,
    preview_queue: Vec<String>,
    action_approval_active: bool,
    benchmark_runs: Vec<String>,
    last_ingestion_target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeSnapshot {
    pub workspace_count: usize,
    pub command_count: usize,
    pub preview_queue_count: usize,
    pub benchmark_run_count: usize,
    pub last_ingestion_target: Option<String>,
}

impl AppState {
    pub fn register_workspace(&self, path: &str) -> bool {
        let mut registry = self.registry.lock().expect("app state poisoned");
        if registry.workspace_paths.iter().any(|existing| existing == path) {
            return false;
        }
        registry.workspace_paths.push(path.to_string());
        true
    }

    pub fn has_workspace(&self, path: &str) -> bool {
        self.registry
            .lock()
            .expect("app state poisoned")
            .workspace_paths
            .iter()
            .any(|existing| existing == path)
    }

    pub fn record_command(&self, command: &str) {
        self.registry
            .lock()
            .expect("app state poisoned")
            .command_log
            .push(command.to_string());
    }

    pub fn set_last_ingestion_target(&self, path: &str) {
        self.registry
            .lock()
            .expect("app state poisoned")
            .last_ingestion_target = Some(path.to_string());
    }

    pub fn replace_preview_queue(&self, queue: Vec<String>) {
        self.registry.lock().expect("app state poisoned").preview_queue = queue;
    }

    pub fn set_approved_preview_queue(&self, queue: Vec<String>) {
        let mut registry = self.registry.lock().expect("app state poisoned");
        registry.preview_queue = queue;
        registry.action_approval_active = true;
    }

    pub fn clear_action_approval(&self) {
        self.registry
            .lock()
            .expect("app state poisoned")
            .action_approval_active = false;
    }

    pub fn is_action_approval_active(&self) -> bool {
        self.registry
            .lock()
            .expect("app state poisoned")
            .action_approval_active
    }

    pub fn preview_queue_len(&self) -> usize {
        self.registry
            .lock()
            .expect("app state poisoned")
            .preview_queue
            .len()
    }

    pub fn drain_preview_queue(&self) -> Vec<String> {
        let mut registry = self.registry.lock().expect("app state poisoned");
        registry.action_approval_active = false;
        std::mem::take(&mut registry.preview_queue)
    }

    pub fn record_benchmark_run(&self, run_id: &str) {
        self.registry
            .lock()
            .expect("app state poisoned")
            .benchmark_runs
            .push(run_id.to_string());
    }

    pub fn has_benchmark_run(&self, run_id: &str) -> bool {
        self.registry
            .lock()
            .expect("app state poisoned")
            .benchmark_runs
            .iter()
            .any(|existing| existing == run_id)
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        let registry = self.registry.lock().expect("app state poisoned");
        RuntimeSnapshot {
            workspace_count: registry.workspace_paths.len(),
            command_count: registry.command_log.len(),
            preview_queue_count: registry.preview_queue.len(),
            benchmark_run_count: registry.benchmark_runs.len(),
            last_ingestion_target: registry.last_ingestion_target.clone(),
        }
    }
}
