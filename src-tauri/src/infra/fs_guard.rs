use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct FilesystemGuard;

impl FilesystemGuard {
    /// Returns Ok(canonical_path) or Err(reason).
    pub fn validate_and_canonicalize(
        &self,
        raw_path: &str,
        workspace_root: &str,
    ) -> Result<PathBuf, String> {
        let path = Path::new(raw_path);

        // 1. Must not be empty
        if raw_path.trim().is_empty() {
            return Err("path is empty".into());
        }

        // 2. Must not contain traversal components
        for component in path.components() {
            let s = component.as_os_str().to_string_lossy();
            if s == ".." {
                return Err(format!("path traversal detected in: {}", raw_path));
            }
        }

        // 3. Resolve to canonical form - file must exist for validation
        let canonical = path
            .canonicalize()
            .map_err(|err| format!("cannot canonicalize '{}': {}", raw_path, err))?;

        // 4. Must live inside the workspace boundary
        let workspace = Path::new(workspace_root)
            .canonicalize()
            .map_err(|err| format!("cannot canonicalize workspace '{}': {}", workspace_root, err))?;

        if !canonical.starts_with(&workspace) {
            return Err(format!(
                "path '{}' is outside workspace boundary '{}'",
                canonical.display(),
                workspace.display()
            ));
        }

        // 5. Must not be a symlink (follow chain - if canonical != original metadata says symlink)
        if path
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(format!("symlinks not permitted: {}", raw_path));
        }

        Ok(canonical)
    }

    /// Validate workspace path exists and is a directory.
    pub fn validate_workspace_path(&self, path: &str) -> bool {
        !path.trim().is_empty() && Path::new(path).is_dir()
    }

    /// Generate a collision-safe target path by appending _001, _002, etc.
    pub fn collision_safe_path(&self, proposed: &Path) -> PathBuf {
        if !proposed.exists() {
            return proposed.to_path_buf();
        }
        let stem = proposed
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let ext = proposed
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let parent = proposed.parent().unwrap_or(Path::new("."));

        for i in 1u32..=999 {
            let candidate = parent.join(format!("{stem}_{i:03}{ext}"));
            if !candidate.exists() {
                return candidate;
            }
        }
        // Fallback: append timestamp nanos
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        parent.join(format!("{stem}_{ts}{ext}"))
    }

    /// Normalize a filename to {DocType}_{Vendor}_{YYYY-MM-DD}.ext
    /// Applies only if LLM inference provides all three fields.
    pub fn normalize_filename(
        &self,
        doc_type: &str,
        vendor: &str,
        date: &str,
        original_ext: &str,
    ) -> String {
        let clean = |s: &str| -> String {
            s.chars()
                .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
                .collect::<String>()
                .trim_matches('_')
                .to_string()
        };
        format!(
            "{}_{}_{}.{}",
            clean(doc_type),
            clean(vendor),
            date.replace('/', "-"),
            original_ext.trim_start_matches('.')
        )
    }
}
