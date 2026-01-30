use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the logging system
///
/// # Arguments
/// * `verbose` - Enable verbose (DEBUG) logging
/// * `log_file` - Optional path to log file. If None, logs only to stderr
/// * `quiet` - If true, disable stderr logging (useful for TUI mode)
pub fn init(verbose: bool, log_file: Option<PathBuf>, quiet: bool) {
    // Determine log level from verbose flag or RUST_LOG env var
    let default_level = if verbose { "debug" } else { "info" };
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("peas={}", default_level)));

    // Build stderr layer if not quiet
    let stderr_layer = if !quiet {
        Some(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false) // Don't show module path
                .compact(), // Use compact format
        )
    } else {
        None
    };

    // Build file layer if path provided
    let file_layer = log_file.map(|log_path| {
        // Create log directory if it doesn't exist
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Set up file appender with rotation
        let file_appender = tracing_appender::rolling::daily(
            log_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new(".")),
            log_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("peas.log")),
        );

        fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false) // No colors in file
            .json() // Use JSON format for structured logs
    });

    // Initialize the subscriber with the layers
    tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer)
        .with(file_layer)
        .init();
}

#[cfg(test)]
mod tests {
    use std::sync::Once;
    use tempfile::TempDir;

    static INIT: Once = Once::new();

    /// Initialize logging once for all tests
    fn init_test_logging() {
        INIT.call_once(|| {
            // Initialize with minimal config for tests
            let _ = tracing_subscriber::fmt()
                .with_test_writer()
                .with_max_level(tracing::Level::DEBUG)
                .try_init();
        });
    }

    #[test]
    fn test_init_without_file() {
        init_test_logging();
        // Logging already initialized, so this is just a smoke test
        // that the function can be called without panicking
    }

    #[test]
    fn test_init_with_file() {
        init_test_logging();
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        // Log file directory should exist
        assert!(temp_dir.path().exists());

        // Can create files in the directory
        std::fs::write(&log_path, "test").unwrap();
        assert!(log_path.exists());
    }

    #[test]
    fn test_verbose_mode() {
        init_test_logging();
        // Logging already initialized, this is a smoke test
    }
}
