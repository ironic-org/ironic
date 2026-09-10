use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{
    app::{app_controller, app_production_guide, app_service},
    common::{naming::Names, source::write_generated},
};
use crate::CliError;

/// Result of creating a new project.
#[derive(Debug)]
pub struct ProjectReport {
    /// Created project directory.
    pub destination: PathBuf,
}

/// Returns the normalized destination directory for a project name.
///
/// # Errors
///
/// Returns an error when the name is not a safe Rust identifier.
pub fn directory_name(name: &str) -> Result<String, CliError> {
    Ok(Names::parse(name)?.kebab)
}

/// Derives a normalized project name from an existing directory.
///
/// # Errors
///
/// Returns an error when the directory cannot form a safe Rust identifier.
pub fn name_from_directory(directory: &Path) -> Result<String, CliError> {
    let name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CliError::InvalidName {
            name: directory.display().to_string(),
        })?;
    directory_name(name)
}

/// Creates a minimal HTTP application scaffold.
///
/// # Errors
///
/// Returns an error when the destination is occupied or files cannot be created.
pub fn create(
    destination: &Path,
    name: &str,
    framework_workspace: Option<&Path>,
) -> Result<ProjectReport, CliError> {
    let names = Names::parse(name)?;
    let version = env!("CARGO_PKG_VERSION");
    let dependency = framework_workspace.map_or_else(
        || {
            format!(
                "version = \"{}.{}\"",
                version.split('.').next().unwrap_or("1"),
                version.split('.').nth(1).unwrap_or("0")
            )
        },
        |path| format!("path = \"{}\"", path.display()),
    );
    let files = [
        ("Cargo.toml", format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nironic = {{ {dependency} }}\n", names.kebab)),
        ("src/main.rs", "mod app;\nmod app_controller;\nmod app_service;\n\nuse ironic::prelude::*;\nuse app::AppModule;\n\n#[ironic::main]\nasync fn main() {\n    let app = Application::builder()\n        .module(AppModule::definition())\n        .platform(AxumAdapter::new())\n        .build().await.expect(\"application must initialise\");\n    app.listen(\"127.0.0.1:3000\").await.expect(\"server failed\");\n}\n".to_string()),
        ("src/app.rs", "use ironic::prelude::*;\nuse crate::{app_controller::AppController, app_service::AppService};\n\n#[derive(Module)]\n#[module(controllers = [AppController], providers = [AppService])]\npub struct AppModule;\n".to_string()),
        ("src/app_controller.rs", app_controller().to_string()),
        ("src/app_service.rs", app_service(&names.kebab, version)),
        ("README.md", format!("# {}\n\nRun `cargo run`, then open http://127.0.0.1:3000.\n", names.kebab)),
        (".gitignore", "/target\n.env\n".to_string()),
        ("PRODUCTION.md", app_production_guide(&names.kebab, 3000)),
    ];
    for (relative, contents) in &files {
        let path = destination.join(relative);
        if path.exists()
            && fs::read_to_string(&path).map_err(|error| CliError::io("read", &path, error))?
                != *contents
        {
            return Err(CliError::FileConflict { path });
        }
    }
    fs::create_dir_all(destination)
        .map_err(|error| CliError::io("create directory", destination, error))?;
    for (relative, contents) in files {
        let path = destination.join(relative);
        if !path.exists() {
            write_generated(&path, &contents)?;
        }
    }
    Ok(ProjectReport {
        destination: destination.to_owned(),
    })
}
