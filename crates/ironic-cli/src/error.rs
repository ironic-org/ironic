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
    pub(crate) fn io(action: &'static str, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }
}
