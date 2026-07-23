use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// The Ironic command-line interface.
///
/// # Example
///
/// ```no_run
/// use clap::Parser;
/// use ironic::cli::Cli;
///
/// let cli = Cli::parse();
/// ```
#[derive(Debug, Parser)]
#[command(name = "ironic", version, about)]
pub struct Cli {
    /// Command to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Supported MVP commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Creates a new application project.
    New(NewArgs),
    /// Runs the current project through Cargo.
    Start(CargoArgs),
    /// Runs the server in development mode with hot reload on file changes.
    Dev(CargoArgs),
    /// Builds the current project through Cargo.
    Build(CargoArgs),
    /// Tests the current project through Cargo.
    Test(CargoArgs),
    /// Generates application source files.
    #[command(alias = "g")]
    Generate(GenerateArgs),
    /// Checks the local Rust and project environment.
    Doctor,
    /// Checks crates.io for a newer version and shows update instructions.
    #[command(alias = "upgrade")]
    Update,
    /// Removes the Ironic binary and caches from the system.
    Uninstall,
    /// Prints project workspace information (name, version, modules).
    Workspace(InspectArgs),
    /// Lists routes declared by controller macros.
    Routes(InspectArgs),
    /// Prints a Graphviz dependency graph from module and injectable declarations.
    Graph(InspectArgs),
    /// Manage database migrations.
    Migrate(MigrateArgs),
}

/// Arguments for project creation.
#[derive(Debug, Args)]
pub struct NewArgs {
    /// Project name and destination directory, or `.` for the current directory.
    pub name: String,
    /// Uses local framework crates from a workspace checkout.
    #[arg(long, hide = true)]
    pub framework_workspace: Option<PathBuf>,
}

/// Arguments passed through to Cargo after `--`.
#[derive(Debug, Args)]
pub struct CargoArgs {
    /// Additional Cargo arguments.
    #[arg(last = true, allow_hyphen_values = true)]
    pub cargo_args: Vec<String>,
}

/// Arguments shared by source inspection commands.
#[derive(Debug, Args)]
pub struct InspectArgs {
    /// Project directory containing `src`.
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

/// Generator selection and source name.
#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// Artifact to generate.
    #[command(subcommand)]
    pub generator: Generator,
}

/// Supported source generators.
#[derive(Debug, Subcommand)]
pub enum Generator {
    /// Generates a module.
    #[command(alias = "mo")]
    Module(NameArgs),
    /// Generates a controller.
    #[command(alias = "co")]
    Controller(NameArgs),
    /// Generates a service.
    #[command(alias = "s")]
    Service(NameArgs),
    /// Generates a repository.
    #[command(alias = "r")]
    Repository(NameArgs),
    /// Generates a module, service, and controller vertical slice.
    #[command(alias = "res")]
    Resource(NameArgs),
    /// Generates a custom parameter decorator.
    #[command(alias = "de")]
    Decorator(NameArgs),
    /// Generates an exception filter.
    #[command(alias = "f")]
    Filter(NameArgs),
    /// Generates a WebSocket gateway.
    #[command(alias = "ga")]
    Gateway(NameArgs),
    /// Generates a guard.
    #[command(alias = "gu")]
    Guard(NameArgs),
    /// Generates an interceptor.
    #[command(alias = "itc")]
    Interceptor(NameArgs),
    /// Generates middleware.
    #[command(alias = "mi")]
    Middleware(NameArgs),
    /// Generates a pipe.
    #[command(alias = "pi")]
    Pipe(NameArgs),
    /// Generates an injectable provider.
    #[command(alias = "pr")]
    Provider(NameArgs),
    /// Generates a production-ready module with authentication and authorization.
    #[command(alias = "rr")]
    ReadyResource(ReadyResourceArgs),
}

/// Ready-resource variant selection.
#[derive(Debug, Subcommand)]
pub enum ReadyResourceVariant {
    /// Full authentication: passwords, JWT, OAuth, sessions, RBAC.
    Auth,
    /// Password hashing and sessions only.
    AuthBasic,
    /// JWT token management only.
    AuthJwt,
    /// `OAuth2` social login with Google and GitHub.
    AuthOauth,
    /// File upload with local, `S3`, `R2`, `Azure`, and `GCS` backends.
    FileUpload,
    /// Email delivery with `SMTP`, `SES`, `SendGrid`, `Mailgun`, and development log.
    Email,
}

/// Arguments for generating a ready resource.
#[derive(Debug, Args)]
pub struct ReadyResourceArgs {
    /// Variant of the ready resource.
    #[command(subcommand)]
    pub variant: ReadyResourceVariant,
}

/// A named generator target.
#[derive(Debug, Args)]
pub struct NameArgs {
    /// Resource or module name.
    pub name: String,
}

/// Arguments for the migrate command.
#[derive(Debug, Args)]
pub struct MigrateArgs {
    /// Migration subcommand to execute.
    #[command(subcommand)]
    pub action: MigrateAction,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn parse_new_with_name() {
        let cli = Cli::try_parse_from(["ironic", "new", "my-app"]).unwrap();
        assert!(matches!(cli.command, Command::New(NewArgs { name, .. }) if name == "my-app"));
    }

    #[test]
    fn parse_new_with_current_dir() {
        let cli = Cli::try_parse_from(["ironic", "new", "."]).unwrap();
        assert!(matches!(cli.command, Command::New(NewArgs { name, .. }) if name == "."));
    }

    #[test]
    fn parse_new_with_workspace() {
        let cli =
            Cli::try_parse_from(["ironic", "new", "my-app", "--framework-workspace", "../"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::New(NewArgs { name, framework_workspace: Some(_), .. }) if name == "my-app"
        ));
    }

    #[test]
    fn parse_start_with_cargo_args() {
        let cli = Cli::try_parse_from(["ironic", "start", "--", "--features", "extra"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Start(CargoArgs { cargo_args }) if cargo_args == ["--features", "extra"]
        ));
    }

    #[test]
    fn parse_build_with_release_flag() {
        let cli = Cli::try_parse_from(["ironic", "build", "--", "--release"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Build(CargoArgs { cargo_args }) if cargo_args == ["--release"]
        ));
    }

    #[test]
    fn parse_start_without_cargo_args() {
        let cli = Cli::try_parse_from(["ironic", "start"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Start(CargoArgs { cargo_args }) if cargo_args.is_empty()
        ));
    }

    #[test]
    fn parse_inspect_default_path() {
        let cli = Cli::try_parse_from(["ironic", "workspace"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Workspace(InspectArgs { path }) if path == PathBuf::from(".")
        ));
    }

    #[test]
    fn parse_inspect_custom_path() {
        let cli = Cli::try_parse_from(["ironic", "routes", "src"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Routes(InspectArgs { path }) if path == PathBuf::from("src")
        ));
    }

    #[test]
    fn parse_generate_all_variants() {
        let cases = [
            ("module", "mo"),
            ("controller", "co"),
            ("service", "s"),
            ("repository", "r"),
            ("resource", "res"),
            ("decorator", "de"),
            ("filter", "f"),
            ("gateway", "ga"),
            ("guard", "gu"),
            ("interceptor", "itc"),
            ("middleware", "mi"),
            ("pipe", "pi"),
            ("provider", "pr"),
        ];
        for (name, alias) in &cases {
            let cli = Cli::try_parse_from(["ironic", "g", alias, "test"]).unwrap();
            let Command::Generate(GenerateArgs { generator }) = &cli.command else {
                panic!("{name} alias `{alias}` did not produce Generate");
            };
            let gen_name = format!("{generator:?}");
            assert!(
                gen_name.to_lowercase().contains(name),
                "{name} alias `{alias}` produced {gen_name}"
            );
        }
    }

    #[test]
    fn parse_ready_resource_variants() {
        let variants = ["auth", "auth-basic", "auth-jwt", "auth-oauth", "file-upload", "email"];
        for variant in &variants {
            let args = ["ironic", "g", "rr", variant];
            let result = Cli::try_parse_from(args);
            assert!(result.is_ok(), "ready-resource variant `{variant}` should parse");
        }
    }

    #[test]
    fn parse_generate_with_name() {
        let cli = Cli::try_parse_from(["ironic", "g", "co", "user_controller"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Generate(GenerateArgs {
                generator: Generator::Controller(NameArgs { name }),
            }) if name == "user_controller"
        ));
    }

    #[test]
    fn parse_migrate_down_with_steps() {
        let cli = Cli::try_parse_from(["ironic", "migrate", "down", "--steps", "3"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Migrate(MigrateArgs {
                action: MigrateAction::Down { steps: 3 },
            })
        ));
    }

    #[test]
    fn parse_migrate_create() {
        let cli = Cli::try_parse_from(["ironic", "migrate", "create", "add_users_table"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Migrate(MigrateArgs {
                action: MigrateAction::Create { name },
            }) if name == "add_users_table"
        ));
    }

    #[test]
    fn debug_output() {
        let cli = Cli::try_parse_from(["ironic", "doctor"]).unwrap();
        let debug = format!("{cli:?}");
        assert!(debug.contains("Doctor"));
    }
}

/// Supported migration operations.
#[derive(Debug, Subcommand)]
pub enum MigrateAction {
    /// Run pending migrations.
    Up,
    /// Revert the last N migrations.
    Down {
        /// Number of migrations to revert (default: 1).
        #[arg(long, default_value = "1")]
        steps: i64,
    },
    /// Create a new migration file.
    Create {
        /// Migration name (e.g. "`create_users_table`").
        name: String,
    },
    /// Show migration status (applied vs pending).
    Status,
}
