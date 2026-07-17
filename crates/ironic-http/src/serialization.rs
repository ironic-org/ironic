use serde_json::Value;

use crate::{
    FrameworkBody, HttpError, Interceptor, InterceptorNext, PipelineFuture, RequestContext,
};

/// A serialization rule for a single JSON field path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldRule {
    /// Field is always excluded from the serialized output.
    Exclude,
    /// Field is only included when the current user has the specified role.
    Expose {
        /// The role required to see this field.
        role: String,
    },
}

/// Describes which fields of a response DTO should be excluded or conditionally
/// exposed.
///
/// Each entry maps a dot-separated field path (e.g. `"secret"`, `"nested.field"`)
/// to its rule.
#[derive(Clone, Debug, Default)]
pub struct FieldRules {
    rules: Vec<(String, FieldRule)>,
}

impl FieldRules {
    /// Empty rule set (all fields are serialised normally).
    #[must_use]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Marks `field` as always excluded from the JSON output.
    #[must_use]
    pub fn exclude(mut self, field: impl Into<String>) -> Self {
        self.rules.push((field.into(), FieldRule::Exclude));
        self
    }

    /// Marks `field` as only included when the current user has `role`.
    #[must_use]
    pub fn expose(mut self, field: impl Into<String>, role: impl Into<String>) -> Self {
        self.rules
            .push((field.into(), FieldRule::Expose { role: role.into() }));
        self
    }

    /// Returns the stored rules.
    #[must_use]
    pub fn to_rules(&self) -> &[(String, FieldRule)] {
        &self.rules
    }
}

/// Helper to insert the current user's roles into a request context.
///
/// An auth middleware or guard would call this before the handler executes.
pub fn set_current_roles(context: &mut RequestContext, roles: Vec<String>) {
    context.insert_extension(CurrentRoles(roles));
}

/// Reads the current user's roles from the request context.
fn current_roles(context: &RequestContext) -> Vec<String> {
    context
        .extension::<CurrentRoles>()
        .map(|r| r.0.clone())
        .unwrap_or_default()
}

/// Extension value stored on `RequestContext`.
#[derive(Clone, Debug)]
struct CurrentRoles(Vec<String>);

/// Interceptor that applies [`FieldRule`]s to JSON response bodies.
///
/// When registered on a route, controller, or globally, this interceptor
/// deserialises JSON responses, walks the value tree, and removes fields
/// that should be excluded or are conditionally exposed (when the current
/// user does not have the required role).
///
/// # Example
///
/// ```ignore
/// use ironic::http::serialization::{FieldRules, SerializeInterceptor};
///
/// RouteDefinition::new(...)
///     .interceptor(SerializeInterceptor::new(
///         FieldRules::new()
///             .exclude("internal_token")
///             .expose("admin_secret", "admin"),
///     ));
/// ```
pub struct SerializeInterceptor {
    rules: FieldRules,
}

impl SerializeInterceptor {
    /// Creates an interceptor that applies the given field rules.
    #[must_use]
    pub fn new(rules: FieldRules) -> Self {
        Self { rules }
    }
}

impl Interceptor for SerializeInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let mut response = next.run(context).await?;

            // Only process JSON responses
            let is_json = response
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.starts_with("application/json"));

            if !is_json {
                return Ok(response);
            }

            let body = match response.body() {
                FrameworkBody::Bytes(bytes) => bytes.clone(),
                FrameworkBody::Stream(bytes) => bytes.as_ref().clone(),
                FrameworkBody::Empty => return Ok(response),
            };

            let mut value: Value = serde_json::from_slice(&body).map_err(|e| {
                HttpError::internal(
                    "RF_SERIALIZE_INTERCEPTOR_PARSE_FAILED",
                    format!("Failed to parse JSON response: {e}"),
                )
            })?;

            let roles = current_roles(context);
            apply_rules(&mut value, &self.rules, &roles);

            let new_body = serde_json::to_vec(&value).map_err(|e| {
                HttpError::internal(
                    "RF_SERIALIZE_INTERCEPTOR_SERIALIZE_FAILED",
                    format!("Failed to re-serialize JSON response: {e}"),
                )
            })?;

            response.set_body(FrameworkBody::Bytes(new_body));
            Ok(response)
        })
    }
}

/// Applies field rules to a JSON value tree in place.
#[allow(clippy::similar_names)]
fn apply_rules(value: &mut Value, rules: &FieldRules, roles: &[String]) {
    match value {
        Value::Object(map) => {
            let mut to_remove = Vec::new();
            for (field_path, rule) in &rules.rules {
                if let Some((top, _rest)) = field_path.split_once('.') {
                    if let Some(nested) = map.get_mut(top) {
                        // Build a sub-ruleset for the nested path
                        let sub_rules = {
                            let mut r = FieldRules::new();
                            for (fp, rl) in &rules.rules {
                                if let Some(suffix) = fp.strip_prefix(&format!("{top}.")) {
                                    r.rules.push((suffix.to_string(), rl.clone()));
                                }
                            }
                            r
                        };
                        apply_rules(nested, &sub_rules, roles);
                    }
                } else if map.contains_key(field_path) {
                    match rule {
                        FieldRule::Exclude => {
                            to_remove.push(field_path.clone());
                        }
                        FieldRule::Expose { role } => {
                            if !roles.iter().any(|r| r == role) {
                                to_remove.push(field_path.clone());
                            }
                        }
                    }
                }
            }
            for key in to_remove {
                map.remove(&key);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                apply_rules(item, rules, roles);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        CompiledHttpApplication, ControllerDefinition, HeaderMap, HttpMethod, Json,
        ProviderDefinition, RequestContext, RouteDefinition, compile_controller_routes, handler_fn,
    };
    use ironic_di::{ContainerBuilder, Scope};
    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct TestDto {
        pub id: u64,
        pub name: String,
        pub secret: String,
        pub internal: String,
    }

    #[test]
    fn field_rules_builder() {
        let rules = FieldRules::new()
            .exclude("internal")
            .expose("secret", "admin");
        assert_eq!(rules.rules.len(), 2);
    }

    #[test]
    fn apply_exclude_rule() {
        let mut value = serde_json::json!({"id": 1, "name": "test", "internal": "hidden"});
        let rules = FieldRules::new().exclude("internal");
        apply_rules(&mut value, &rules, &[]);
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("name"));
        assert!(!obj.contains_key("internal"));
    }

    #[test]
    fn apply_expose_rule_with_role() {
        let mut value = serde_json::json!({"id": 1, "secret": "classified"});
        let rules = FieldRules::new().expose("secret", "admin");

        // No roles -> field removed
        apply_rules(&mut value, &rules, &[]);
        assert!(!value.as_object().unwrap().contains_key("secret"));
    }

    #[test]
    fn apply_expose_rule_without_role() {
        let mut value = serde_json::json!({"id": 1, "secret": "classified"});
        let rules = FieldRules::new().expose("secret", "admin");

        // Wrong role -> field removed
        apply_rules(&mut value, &rules, &["user".to_string()]);
        assert!(!value.as_object().unwrap().contains_key("secret"));
    }

    #[test]
    fn apply_expose_rule_with_matching_role() {
        let mut value = serde_json::json!({"id": 1, "secret": "classified"});
        let rules = FieldRules::new().expose("secret", "admin");

        // Matching role -> field kept
        apply_rules(&mut value, &rules, &["admin".to_string()]);
        assert!(value.as_object().unwrap().contains_key("secret"));
    }

    #[test]
    fn apply_rules_to_nested_field() {
        let mut value = serde_json::json!({"user": {"name": "alice", "ssn": "123-45-6789"}});
        let rules = FieldRules::new().exclude("user.ssn");
        apply_rules(&mut value, &rules, &[]);
        let user = &value["user"];
        assert!(user.get("name").is_some());
        assert!(user.get("ssn").is_none());
    }

    #[test]
    fn apply_rules_to_array_items() {
        let mut value = serde_json::json!([
            {"id": 1, "token": "abc"},
            {"id": 2, "token": "def"},
        ]);
        let rules = FieldRules::new().exclude("token");
        apply_rules(&mut value, &rules, &[]);
        for item in value.as_array().unwrap() {
            assert!(!item.as_object().unwrap().contains_key("token"));
            assert!(item.as_object().unwrap().contains_key("id"));
        }
    }

    #[test]
    fn no_rules_preserves_all_fields() {
        let mut value = serde_json::json!({"a": 1, "b": 2});
        let rules = FieldRules::new();
        apply_rules(&mut value, &rules, &[]);
        assert_eq!(value, serde_json::json!({"a": 1, "b": 2}));
    }

    struct Controller;

    fn controller_route(interceptors: Vec<SerializeInterceptor>) -> CompiledHttpApplication {
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/",
            "handler",
            handler_fn(|_controller: Arc<Controller>, _arguments| async move {
                Ok::<_, HttpError>(Json(TestDto {
                    id: 1,
                    name: "test".into(),
                    secret: "classified".into(),
                    internal: "hidden".into(),
                }))
            }),
        )
        .unwrap();

        let route = if interceptors.is_empty() {
            route
        } else {
            let mut r = route;
            for interceptor in interceptors {
                r = r.interceptor(interceptor);
            }
            r
        };

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/api", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        CompiledHttpApplication::new(
            container.build(),
            compile_controller_routes([controller]).unwrap(),
        )
    }

    fn request_context() -> RequestContext {
        RequestContext::new(crate::FrameworkRequest::new(
            HttpMethod::GET,
            "/api".parse().unwrap(),
            HeaderMap::new(),
            Vec::new(),
        ))
    }

    #[tokio::test]
    async fn serialize_interceptor_excludes_field() {
        let rules = FieldRules::new().exclude("internal");
        let interceptor = SerializeInterceptor::new(rules);
        let app = controller_route(vec![interceptor]);
        let mut cx = request_context();
        let route = &app.routes()[0];
        let response = app.execute(route, &mut cx).await.unwrap();
        let body = match response.body() {
            FrameworkBody::Bytes(b) => b.clone(),
            FrameworkBody::Stream(b) => b.as_ref().clone(),
            _ => panic!("expected bytes body"),
        };
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("id").is_some());
        assert!(value.get("name").is_some());
        assert!(value.get("secret").is_some());
        assert!(value.get("internal").is_none());
    }

    #[tokio::test]
    async fn serialize_interceptor_exposes_field_with_role() {
        let rules = FieldRules::new().expose("secret", "admin");
        let interceptor = SerializeInterceptor::new(rules);
        let app = controller_route(vec![interceptor]);
        let mut cx = request_context();
        set_current_roles(&mut cx, vec!["admin".into()]);
        let route = &app.routes()[0];
        let response = app.execute(route, &mut cx).await.unwrap();
        let body = match response.body() {
            FrameworkBody::Bytes(b) => b.clone(),
            FrameworkBody::Stream(b) => b.as_ref().clone(),
            _ => panic!("expected bytes body"),
        };
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("secret").is_some());
    }

    #[tokio::test]
    async fn serialize_interceptor_hides_exposed_field_without_role() {
        let rules = FieldRules::new().expose("secret", "admin");
        let interceptor = SerializeInterceptor::new(rules);
        let app = controller_route(vec![interceptor]);
        let mut cx = request_context();
        set_current_roles(&mut cx, vec!["user".into()]);
        let route = &app.routes()[0];
        let response = app.execute(route, &mut cx).await.unwrap();
        let body = match response.body() {
            FrameworkBody::Bytes(b) => b.clone(),
            FrameworkBody::Stream(b) => b.as_ref().clone(),
            _ => panic!("expected bytes body"),
        };
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("secret").is_none());
    }

    #[tokio::test]
    async fn serialize_interceptor_no_rules_preserves_response() {
        let rules = FieldRules::new();
        let interceptor = SerializeInterceptor::new(rules);
        let app = controller_route(vec![interceptor]);
        let mut cx = request_context();
        let route = &app.routes()[0];
        let response = app.execute(route, &mut cx).await.unwrap();
        let body = match response.body() {
            FrameworkBody::Bytes(b) => b.clone(),
            FrameworkBody::Stream(b) => b.as_ref().clone(),
            _ => panic!("expected bytes body"),
        };
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value.get("id").unwrap(), 1);
        assert_eq!(value.get("name").unwrap(), "test");
        assert_eq!(value.get("secret").unwrap(), "classified");
        assert_eq!(value.get("internal").unwrap(), "hidden");
    }
}
