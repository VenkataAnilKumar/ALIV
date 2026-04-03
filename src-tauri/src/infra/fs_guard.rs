#[derive(Debug, Default)]
pub struct FilesystemGuard;

impl FilesystemGuard {
    pub fn validate_workspace_path(&self, path: &str) -> bool {
        !path.trim().is_empty()
    }
}
