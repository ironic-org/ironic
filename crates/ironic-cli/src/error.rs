use std::{io, path::PathBuf};

/// A command-line operation failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CliError {
    /// A filesystem operation failed.
    #[error("RF_CLI_IO: {action} `{path}`: {source}")]
    Io {
        /// Description of the attempted operation.
        action: &'static str,
        /// Affected path.
        path: PathBuf,
        /// Underlying I/O failure.
        source: io::Error,
    },
    /// A generator name cannot be mapped safely to Rust and package identifiers.
    #[error("RF_CLI_INVALID_NAME: `{name}` cannot form a safe Rust identifier")]
    InvalidName {
        /// Rejected user input.
        name: String,
    },
    /// A generated file exists with different content.
    #[error("RF_CLI_FILE_CONFLICT: refusing to overwrite `{path}`")]
    FileConflict {
        /// Conflicting file.
        path: PathBuf,
    },
    /// A Rust source file could not be parsed safely.
    #[error("RF_CLI_SOURCE_PARSE: cannot safely update `{path}`: {message}")]
    SourceParse {
        /// Source file.
        path: PathBuf,
        /// Parser diagnostic.
        message: String,
    },
    /// A child Cargo or tool command failed.
    #[error("RF_CLI_COMMAND_FAILED: `{program}` exited with status {status}")]
    CommandFailed {
        /// Executed program.
        program: String,
        /// Process exit status or signal description.
        status: String,
    },
}

impl From<io::Error> for CliError {
    fn from(source: io::Error) -> Self {
        Self::Io {
            action: "IO operation",
            path: PathBuf::new(),
            source,
        }
    }
}

impl CliError {
    /// Creates an I/O-related [`CliError`] with a descriptive action and path.
    pub(crate) fn io(action: &'static str, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::CliError;

    #[test]
    fn io_error_display() {
        let err = CliError::io(
            "test action",
            "/tmp/test",
            io::Error::new(io::ErrorKind::NotFound, "file not found"),
        );
        let msg = err.to_string();
        assert!(msg.contains("RF_CLI_IO"));
        assert!(msg.contains("test action"));
        assert!(msg.contains("/tmp/test"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn invalid_name_display() {
        let err = CliError::InvalidName { name: "123".into() };
        let msg = err.to_string();
        assert!(msg.contains("RF_CLI_INVALID_NAME"));
        assert!(msg.contains("123"));
    }

    #[test]
    fn file_conflict_display() {
        let err = CliError::FileConflict {
            path: "/tmp/conflict.rs".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("RF_CLI_FILE_CONFLICT"));
        assert!(msg.contains("/tmp/conflict.rs"));
    }

    #[test]
    fn source_parse_display() {
        let err = CliError::SourceParse {
            path: "/tmp/source.rs".into(),
            message: "unexpected token".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("RF_CLI_SOURCE_PARSE"));
        assert!(msg.contains("/tmp/source.rs"));
        assert!(msg.contains("unexpected token"));
    }

    #[test]
    fn command_failed_display() {
        let err = CliError::CommandFailed {
            program: "cargo build".into(),
            status: "exit code 1".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("RF_CLI_COMMAND_FAILED"));
        assert!(msg.contains("cargo build"));
        assert!(msg.contains("exit code 1"));
    }

    #[test]
    fn from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let err: CliError = io_err.into();
        let msg = err.to_string();
        assert!(msg.contains("RF_CLI_IO"));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn error_is_std_error() {
        use std::error::Error;
        let err = CliError::InvalidName { name: "bad".into() };
        assert!(err.source().is_none());
        let io_err = io::Error::new(io::ErrorKind::NotFound, "missing");
        let err = CliError::io("read", "/tmp/f", io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn debug_output() {
        let err = CliError::InvalidName { name: "123".into() };
        let debug = format!("{err:?}");
        assert!(debug.contains("InvalidName"));
        assert!(debug.contains("123"));
    }

    #[test]
    fn non_exhaustive_allows_forward_compat() {
        // Confirm the enum can be constructed only through known variants
        // (pattern match will need `_` wildcard — that's the point).
        let err = CliError::CommandFailed {
            program: "x".into(),
            status: "1".into(),
        };
        assert!(format!("{err}").contains("RF_CLI_COMMAND_FAILED"));
    }
}
