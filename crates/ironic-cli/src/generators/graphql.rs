use std::path::Path;

use crate::CliError;

use super::{
    GenerationReport,
    common::{naming, source},
    record,
};

/// Generates a GraphQL resolver scaffold.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_graphql_resolver(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = naming::Names::parse(name)?;
    let mut report = GenerationReport::default();
    let path = root
        .join("src")
        .join(format!("{}_resolver.rs", names.snake));
    let contents = format!(
        r#"use ::ironic::prelude::*;

#[resolver]
pub struct {name}Resolver;

#[gql_query]
async fn {snake}_query(&self) -> String {{
    "Hello from {name}!".to_string()
}}
"#,
        name = names.pascal,
        snake = names.snake,
    );
    let changed = source::write_generated(&path, &contents)?;
    record(&mut report, &path, changed);
    Ok(report)
}
