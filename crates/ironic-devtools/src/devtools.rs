//! Read-only development UI for compiled modules, providers, and routes.

use std::sync::Arc;

use axum::{Json, Router, extract::State, response::Html, routing::get};
use serde::Serialize;

use crate::{CompiledApplicationGraph, CompiledHttpApplication};

/// Serializable application graph and route snapshot.
///
/// # Errors
///
/// Construction is infallible; serialization may fail at Axum runtime.
///
/// # Panics
///
/// Never panics.
#[derive(Clone, Debug, Serialize)]
pub struct DevtoolsSnapshot {
    /// Root module type name.
    pub root: String,
    /// Compiled modules in initialization order.
    pub modules: Vec<ModuleSnapshot>,
    /// Compiled HTTP routes.
    pub routes: Vec<RouteSnapshot>,
}

/// Serializable module details.
#[derive(Clone, Debug, Serialize)]
pub struct ModuleSnapshot {
    /// Module type name.
    pub name: String,
    /// Direct imported module type names.
    pub imports: Vec<String>,
    /// Owned provider type names and scopes.
    pub providers: Vec<ProviderSnapshot>,
}

/// Serializable provider details.
#[derive(Clone, Debug, Serialize)]
pub struct ProviderSnapshot {
    /// Provider type name.
    pub name: String,
    /// `Singleton`, `Transient`, or `Request`.
    pub scope: String,
    /// Declared dependency type names.
    pub dependencies: Vec<String>,
}

/// Serializable route details.
#[derive(Clone, Debug, Serialize)]
pub struct RouteSnapshot {
    /// HTTP method.
    pub method: String,
    /// Normalized path.
    pub path: String,
    /// Owning controller type.
    pub controller: String,
    /// Handler method name.
    pub handler: String,
}

impl DevtoolsSnapshot {
    /// Captures immutable metadata from compiled runtime state.
    #[must_use]
    pub fn capture(graph: &CompiledApplicationGraph, http: &CompiledHttpApplication) -> Self {
        let modules = graph
            .modules()
            .iter()
            .map(|module| ModuleSnapshot {
                name: module.id().type_name().into(),
                imports: module
                    .imports()
                    .iter()
                    .map(|id| id.type_name().into())
                    .collect(),
                providers: module
                    .providers()
                    .iter()
                    .map(|provider| ProviderSnapshot {
                        name: provider.key().type_name().into(),
                        scope: format!("{:?}", provider.scope()),
                        dependencies: provider
                            .dependencies()
                            .iter()
                            .map(|dependency| dependency.key().type_name().into())
                            .collect(),
                    })
                    .collect(),
            })
            .collect();
        let routes = http
            .routes()
            .iter()
            .map(|route| RouteSnapshot {
                method: route.method().to_string(),
                path: route.path().into(),
                controller: route.controller().type_name().into(),
                handler: route.handler_name().into(),
            })
            .collect();
        Self {
            root: graph.root().type_name().into(),
            modules,
            routes,
        }
    }
}

/// Builds a read-only Axum UI router. Mount it only in trusted development environments.
pub fn router(snapshot: DevtoolsSnapshot) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/snapshot.json", get(snapshot_json))
        .with_state(Arc::new(snapshot))
}

async fn snapshot_json(State(snapshot): State<Arc<DevtoolsSnapshot>>) -> Json<DevtoolsSnapshot> {
    Json((*snapshot).clone())
}

async fn index(State(snapshot): State<Arc<DevtoolsSnapshot>>) -> Html<String> {
    use std::fmt::Write;

    let mut rows = String::new();
    for route in &snapshot.routes {
        let _ = write!(
            rows,
            "<tr><td>{}</td><td>{}</td><td>{}::{}</td></tr>",
            escape(&route.method),
            escape(&route.path),
            escape(&route.controller),
            escape(&route.handler)
        );
    }
    Html(format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Ironic Devtools</title><style>body{{font:14px system-ui;margin:2rem;max-width:1100px}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #ddd;padding:.6rem;text-align:left}}code{{background:#f4f4f4;padding:.15rem .3rem}}</style></head><body><h1>Ironic Devtools</h1><p>Root module: <code>{}</code></p><p>{} modules · {} routes · <a href=\"snapshot.json\">JSON snapshot</a></p><table><thead><tr><th>Method</th><th>Path</th><th>Handler</th></tr></thead><tbody>{rows}</tbody></table></body></html>",
        escape(&snapshot.root),
        snapshot.modules.len(),
        snapshot.routes.len()
    ))
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_ampersand() {
        assert_eq!(escape("a&b"), "a&amp;b");
    }

    #[test]
    fn escape_angle_brackets() {
        assert_eq!(escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn escape_quotes() {
        assert_eq!(escape(r#"a"b'c"#), "a&quot;b&#39;c");
    }

    #[test]
    fn escape_noop_for_plain_text() {
        assert_eq!(escape("hello world"), "hello world");
    }

    #[test]
    fn devtools_snapshot_serialize() {
        let snapshot = DevtoolsSnapshot {
            root: "AppModule".into(),
            modules: vec![ModuleSnapshot {
                name: "FooModule".into(),
                imports: vec!["BarModule".into()],
                providers: vec![ProviderSnapshot {
                    name: "DbPool".into(),
                    scope: "Singleton".into(),
                    dependencies: vec![],
                }],
            }],
            routes: vec![RouteSnapshot {
                method: "GET".into(),
                path: "/users".into(),
                controller: "UserController".into(),
                handler: "list".into(),
            }],
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("AppModule"));
        assert!(json.contains("FooModule"));
        assert!(json.contains("DbPool"));
        assert!(json.contains("GET"));
    }
}
