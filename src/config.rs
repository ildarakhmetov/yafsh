/// Version string for the shell.
pub const VERSION: &str = "0.4.0";

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
