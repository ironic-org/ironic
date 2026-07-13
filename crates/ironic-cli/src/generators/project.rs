use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{naming::Names, source::write_generated};
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
/// Returns [`CliError`] when `name` contains no usable identifier characters.
pub fn directory_name(name: &str) -> Result<String, CliError> {
    Ok(Names::parse(name)?.kebab)
}

/// Creates a complete application scaffold.
///
/// `framework_workspace` selects local path dependencies for framework development and tests.
/// Published CLI use defaults to the CLI's matching framework version.
///
/// # Errors
///
/// Returns [`CliError`] when the destination is occupied or files cannot be created.
pub fn create(
    destination: &Path,
    name: &str,
    framework_workspace: Option<&Path>,
) -> Result<ProjectReport, CliError> {
    let names = Names::parse(name)?;
    if destination.exists()
        && fs::read_dir(destination)
            .map_err(|error| CliError::io("read directory", destination, error))?
            .next()
            .is_some()
    {
        return Err(CliError::ProjectExists {
            path: destination.to_owned(),
        });
    }
    fs::create_dir_all(destination)
        .map_err(|error| CliError::io("create directory", destination, error))?;

    let manifest = manifest(&names.kebab, framework_workspace);
    let files = [
        (destination.join("Cargo.toml"), manifest),
        (
            destination.join("ironic.toml"),
            project_config(&names.kebab),
        ),
        (destination.join("src/main.rs"), main_source()),
        (destination.join("src/app.rs"), app_source()),
        (destination.join("src/modules/mod.rs"), String::new()),
    ];
    for (path, contents) in files {
        write_generated(&path, &contents)?;
    }
    Ok(ProjectReport {
        destination: destination.to_owned(),
    })
}

fn manifest(name: &str, workspace: Option<&Path>) -> String {
    let dependencies = workspace.map_or_else(
        || format!("ironic = \"{}\"", env!("CARGO_PKG_VERSION")),
        |workspace| format!("ironic = {{ path = \"{}\" }}", toml_path(workspace)),
    );
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \"1.85\"\npublish = false\n\n[dependencies]\n{dependencies}\n\n[workspace]\n"
    )
}

fn main_source() -> String {
    "mod app;\nmod modules;\n\nuse ironic::{AxumAdapter, prelude::*};\n\nuse app::AppModule;\n\n#[ironic::main]\nasync fn main() {\n    let application = FrameworkApplication::builder()\n        .module(AppModule::definition())\n        .platform(AxumAdapter::new())\n        .build()\n        .await\n        .expect(\"application must initialize\");\n\n    application\n        .listen(\"127.0.0.1:3000\")\n        .await\n        .expect(\"application server failed\");\n}\n"
        .to_owned()
}

fn app_source() -> String {
    "use ironic::prelude::*;\n\n#[derive(Module)]\n#[module()]\npub struct AppModule;\n".to_owned()
}

fn project_config(name: &str) -> String {
    format!(
        "[project]\nname = \"{name}\"\nsource_root = \"src\"\ndefault_module = \"src/app.rs\"\n\n[generate]\nmodule_path = \"src/modules\"\n"
    )
}

fn toml_path(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}
