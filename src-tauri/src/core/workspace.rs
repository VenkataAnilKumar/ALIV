#[derive(Debug, Default)]
pub struct WorkspaceService;

impl WorkspaceService {
    pub fn register_path(&self, path: &str) -> String {
        format!("workspace registered for {}", path)
    }
}
