/// Version string for the shell.
pub const VERSION: &str = "0.5.0";

/// Return the path to the RC configuration file (~/.yafshrc).
pub fn rc_path() -> Option<std::path::PathBuf> {
    dirs_or_home().map(|h| h.join(".yafshrc"))
}

/// Return the path to the history file (~/.yafsh_history).
pub fn history_path() -> Option<std::path::PathBuf> {
    dirs_or_home().map(|h| h.join(".yafsh_history"))
}

/// Get the user's home directory from $HOME.
fn dirs_or_home() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(std::path::PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        // Version should be a non-empty semver-ish string
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.'));
    }

    #[test]
    fn test_rc_path_ends_with_yafshrc() {
        if let Some(path) = rc_path() {
            assert!(
                path.to_string_lossy().ends_with(".yafshrc"),
                "rc path should end with .yafshrc, got {:?}",
                path
            );
        }
        // If HOME is not set rc_path() may return None — that's fine.
    }

    #[test]
    fn test_history_path_ends_with_yafsh_history() {
        if let Some(path) = history_path() {
            assert!(
                path.to_string_lossy().ends_with(".yafsh_history"),
                "history path should end with .yafsh_history, got {:?}",
                path
            );
        }
    }

    #[test]
    fn test_rc_path_none_when_home_unset() {
        // Temporarily remove HOME and verify the function handles it
        let saved = std::env::var("HOME").ok();
        std::env::remove_var("HOME");

        let result = rc_path();
        // Restore HOME before any assertion that could panic
        if let Some(h) = saved {
            std::env::set_var("HOME", h);
        }

        assert!(
            result.is_none(),
            "rc_path should be None when HOME is unset"
        );
    }

    #[test]
    fn test_history_path_none_when_home_unset() {
        let saved = std::env::var("HOME").ok();
        std::env::remove_var("HOME");

        let result = history_path();
        if let Some(h) = saved {
            std::env::set_var("HOME", h);
        }

        assert!(
            result.is_none(),
            "history_path should be None when HOME is unset"
        );
    }
}
