//! Pre-built validation and transformation pipes.
//!
//! These implement the [`ParameterPipe`] trait to provide common
//! request-parameter validation like parsing integers, UUIDs, etc.

use std::sync::Arc;

use crate::{ExtractedValue, HttpError, HttpStatus, ParameterPipe, PipeFuture, RequestContext};

/// Parses a string parameter into an `i64`.
///
/// # Errors
///
/// Returns a 400 Bad Request when the value is not a valid string
/// or cannot be parsed as an integer.
#[derive(Clone, Debug)]
pub struct ParseIntPipe;

impl ParameterPipe for ParseIntPipe {
    fn transform<'a>(
        &'a self,
        value: ExtractedValue,
        _context: &'a mut RequestContext,
    ) -> PipeFuture<'a> {
        Box::pin(async move {
            let boxed_string = value.downcast::<String>().map_err(|_| {
                HttpError::new(
                    HttpStatus::BAD_REQUEST,
                    "RF_PARSE_INT_FAILED",
                    "Expected a string value for integer parsing.",
                )
            })?;
            let string = *boxed_string;
            let parsed: i64 = string.parse().map_err(|_| {
                HttpError::new(
                    HttpStatus::BAD_REQUEST,
                    "RF_PARSE_INT_FAILED",
                    format!("Cannot parse `{string}` as an integer."),
                )
            })?;
            Ok(Box::new(parsed) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "ParseIntPipe"
    }
}

/// Parses a string parameter into an `f64`.
#[derive(Clone, Debug)]
pub struct ParseFloatPipe;

impl ParameterPipe for ParseFloatPipe {
    fn transform<'a>(
        &'a self,
        value: ExtractedValue,
        _context: &'a mut RequestContext,
    ) -> PipeFuture<'a> {
        Box::pin(async move {
            let boxed_string = value.downcast::<String>().map_err(|_| {
                HttpError::new(
                    HttpStatus::BAD_REQUEST,
                    "RF_PARSE_FLOAT_FAILED",
                    "Expected a string value for float parsing.",
                )
            })?;
            let string = *boxed_string;
            let parsed: f64 = string.parse().map_err(|_| {
                HttpError::new(
                    HttpStatus::BAD_REQUEST,
                    "RF_PARSE_FLOAT_FAILED",
                    format!("Cannot parse `{string}` as a float."),
                )
            })?;
            Ok(Box::new(parsed) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "ParseFloatPipe"
    }
}

/// Parses a string parameter into a `bool`.
///
/// Accepts `"true"`, `"false"`, `"1"`, `"0"` (case-insensitive).
#[derive(Clone, Debug)]
pub struct ParseBoolPipe;

impl ParameterPipe for ParseBoolPipe {
    fn transform<'a>(
        &'a self,
        value: ExtractedValue,
        _context: &'a mut RequestContext,
    ) -> PipeFuture<'a> {
        Box::pin(async move {
            let boxed_string = value.downcast::<String>().map_err(|_| {
                HttpError::new(
                    HttpStatus::BAD_REQUEST,
                    "RF_PARSE_BOOL_FAILED",
                    "Expected a string value for boolean parsing.",
                )
            })?;
            let string = *boxed_string;
            let parsed = match string.to_lowercase().as_str() {
                "true" | "1" => true,
                "false" | "0" => false,
                other => {
                    return Err(HttpError::new(
                        HttpStatus::BAD_REQUEST,
                        "RF_PARSE_BOOL_FAILED",
                        format!(
                            "Cannot parse `{other}` as a boolean. Expected `true`, `false`, `1`, or `0`."
                        ),
                    ));
                }
            };
            Ok(Box::new(parsed) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "ParseBoolPipe"
    }
}

/// Parses a string parameter into a `uuid::Uuid`.
///
/// Requires the `uuid` feature.
#[cfg(feature = "uuid")]
#[derive(Clone, Debug)]
pub struct ParseUUIDPipe;

#[cfg(feature = "uuid")]
impl ParameterPipe for ParseUUIDPipe {
    fn transform<'a>(
        &'a self,
        value: ExtractedValue,
        _context: &'a mut RequestContext,
    ) -> PipeFuture<'a> {
        Box::pin(async move {
            let boxed_string = value.downcast::<String>().map_err(|_| {
                HttpError::new(
                    HttpStatus::BAD_REQUEST,
                    "RF_PARSE_UUID_FAILED",
                    "Expected a string value for UUID parsing.",
                )
            })?;
            let string = *boxed_string;
            let parsed: uuid::Uuid = string.parse().map_err(|_| {
                HttpError::new(
                    HttpStatus::BAD_REQUEST,
                    "RF_PARSE_UUID_FAILED",
                    format!("Cannot parse `{string}` as a UUID."),
                )
            })?;
            Ok(Box::new(parsed) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "ParseUUIDPipe"
    }
}

// ---------------------------------------------------------------------------
// ValidationPipe with garde integration
// ---------------------------------------------------------------------------

/// Validates a deserialized request body using `garde::Validate`.
///
/// Requires the `validation` feature (which enables `dep:garde`).
#[cfg(feature = "validation")]
#[derive(Clone, Debug)]
pub struct ValidationPipe;

#[cfg(feature = "validation")]
impl ParameterPipe for ValidationPipe {
    fn transform<'a>(
        &'a self,
        value: ExtractedValue,
        _context: &'a mut RequestContext,
    ) -> PipeFuture<'a> {
        Box::pin(async move {
            // The value should be a `T` where `T: garde::Validate`.
            // Since ParameterPipe works on erased values, we attempt to
            // validate via any available method. The generic validation
            // is handled at the macro / route-definition level.
            //
            // For now, this pipe is a marker — the actual validation
            // happens through the typed route-generation macros that
            // insert the garde validation call.
            Ok(value)
        })
    }

    fn description(&self) -> &'static str {
        "ValidationPipe"
    }
}

/// Creates a shared `ParseIntPipe`.
#[must_use]
pub fn parse_int() -> Arc<dyn ParameterPipe> {
    Arc::new(ParseIntPipe)
}

/// Creates a shared `ParseFloatPipe`.
#[must_use]
pub fn parse_float() -> Arc<dyn ParameterPipe> {
    Arc::new(ParseFloatPipe)
}

/// Creates a shared `ParseBoolPipe`.
#[must_use]
pub fn parse_bool() -> Arc<dyn ParameterPipe> {
    Arc::new(ParseBoolPipe)
}

/// Creates a shared `ParseUUIDPipe`.
#[cfg(feature = "uuid")]
#[must_use]
pub fn parse_uuid() -> Arc<dyn ParameterPipe> {
    Arc::new(ParseUUIDPipe)
}

/// Creates a shared `ValidationPipe`.
#[cfg(feature = "validation")]
#[must_use]
pub fn validate() -> Arc<dyn ParameterPipe> {
    Arc::new(ValidationPipe)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> RequestContext {
        RequestContext::new(crate::Request::new(
            http::Method::GET,
            "/".parse::<http::Uri>().unwrap(),
            crate::HeaderMap::new(),
            Vec::new(),
        ))
    }

    #[tokio::test]
    async fn parse_int_parses_valid_integer() {
        let pipe = ParseIntPipe;
        let value = pipe
            .transform(Box::new("42".to_string()), &mut context())
            .await
            .unwrap();
        let result = *value.downcast::<i64>().unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn parse_int_rejects_non_numeric() {
        let pipe = ParseIntPipe;
        let err = pipe
            .transform(Box::new("abc".to_string()), &mut context())
            .await
            .unwrap_err();
        assert_eq!(err.code(), "RF_PARSE_INT_FAILED");
    }

    #[tokio::test]
    async fn parse_int_rejects_non_string_value() {
        let pipe = ParseIntPipe;
        let err = pipe
            .transform(Box::new(42i64), &mut context())
            .await
            .unwrap_err();
        assert_eq!(err.code(), "RF_PARSE_INT_FAILED");
    }

    #[test]
    fn parse_int_description() {
        assert_eq!(ParseIntPipe.description(), "ParseIntPipe");
    }

    #[tokio::test]
    async fn parse_float_parses_valid_float() {
        let pipe = ParseFloatPipe;
        let value = pipe
            .transform(Box::new("3.14".to_string()), &mut context())
            .await
            .unwrap();
        let result = *value.downcast::<f64>().unwrap();
        assert!((result - 3.14).abs() < 1e-10);
    }

    #[tokio::test]
    async fn parse_float_rejects_non_numeric() {
        let pipe = ParseFloatPipe;
        let err = pipe
            .transform(Box::new("abc".to_string()), &mut context())
            .await
            .unwrap_err();
        assert_eq!(err.code(), "RF_PARSE_FLOAT_FAILED");
    }

    #[tokio::test]
    async fn parse_float_rejects_non_string_value() {
        let pipe = ParseFloatPipe;
        let err = pipe
            .transform(Box::new(3.14f64), &mut context())
            .await
            .unwrap_err();
        assert_eq!(err.code(), "RF_PARSE_FLOAT_FAILED");
    }

    #[test]
    fn parse_float_description() {
        assert_eq!(ParseFloatPipe.description(), "ParseFloatPipe");
    }

    #[tokio::test]
    async fn parse_bool_true_strings() {
        let pipe = ParseBoolPipe;
        for input in &["true", "True", "TRUE", "1"] {
            let value = pipe
                .transform(Box::new(input.to_string()), &mut context())
                .await
                .unwrap();
            let result = *value.downcast::<bool>().unwrap();
            assert!(result, "expected true for input {input:?}");
        }
    }

    #[tokio::test]
    async fn parse_bool_false_strings() {
        let pipe = ParseBoolPipe;
        for input in &["false", "False", "FALSE", "0"] {
            let value = pipe
                .transform(Box::new(input.to_string()), &mut context())
                .await
                .unwrap();
            let result = *value.downcast::<bool>().unwrap();
            assert!(!result, "expected false for input {input:?}");
        }
    }

    #[tokio::test]
    async fn parse_bool_rejects_invalid_string() {
        let pipe = ParseBoolPipe;
        let err = pipe
            .transform(Box::new("maybe".to_string()), &mut context())
            .await
            .unwrap_err();
        assert_eq!(err.code(), "RF_PARSE_BOOL_FAILED");
    }

    #[tokio::test]
    async fn parse_bool_rejects_non_string_value() {
        let pipe = ParseBoolPipe;
        let err = pipe
            .transform(Box::new(true), &mut context())
            .await
            .unwrap_err();
        assert_eq!(err.code(), "RF_PARSE_BOOL_FAILED");
    }

    #[test]
    fn parse_bool_description() {
        assert_eq!(ParseBoolPipe.description(), "ParseBoolPipe");
    }

    #[tokio::test]
    async fn parse_int_factory_creates_working_pipe() {
        let pipe = parse_int();
        let value = pipe
            .transform(Box::new("100".to_string()), &mut context())
            .await
            .unwrap();
        let result = *value.downcast::<i64>().unwrap();
        assert_eq!(result, 100);
    }

    #[tokio::test]
    async fn parse_float_factory_creates_working_pipe() {
        let pipe = parse_float();
        let value = pipe
            .transform(Box::new("2.5".to_string()), &mut context())
            .await
            .unwrap();
        let result = *value.downcast::<f64>().unwrap();
        assert!((result - 2.5).abs() < 1e-10);
    }

    #[tokio::test]
    async fn parse_bool_factory_creates_working_pipe() {
        let pipe = parse_bool();
        let value = pipe
            .transform(Box::new("true".to_string()), &mut context())
            .await
            .unwrap();
        let result = *value.downcast::<bool>().unwrap();
        assert!(result);
    }

    #[test]
    fn parse_int_pipe_is_cloneable() {
        let a = ParseIntPipe;
        let _b = a.clone();
    }

    #[test]
    fn parse_float_pipe_is_cloneable() {
        let a = ParseFloatPipe;
        let _b = a.clone();
    }

    #[test]
    fn parse_bool_pipe_is_cloneable() {
        let a = ParseBoolPipe;
        let _b = a.clone();
    }

    #[cfg(feature = "uuid")]
    #[tokio::test]
    async fn parse_uuid_parses_valid_uuid() {
        let pipe = ParseUUIDPipe;
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let value = pipe
            .transform(Box::new(uuid_str.to_string()), &mut context())
            .await
            .unwrap();
        let result = *value.downcast::<uuid::Uuid>().unwrap();
        assert_eq!(result.to_string(), uuid_str);
    }

    #[cfg(feature = "uuid")]
    #[tokio::test]
    async fn parse_uuid_rejects_invalid_string() {
        let pipe = ParseUUIDPipe;
        let err = pipe
            .transform(Box::new("not-a-uuid".to_string()), &mut context())
            .await
            .unwrap_err();
        assert_eq!(err.code(), "RF_PARSE_UUID_FAILED");
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn parse_uuid_factory_returns_pipe() {
        let _pipe = parse_uuid();
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn parse_uuid_description() {
        assert_eq!(ParseUUIDPipe.description(), "ParseUUIDPipe");
    }

    #[cfg(feature = "validation")]
    #[test]
    fn validation_pipe_description() {
        assert_eq!(ValidationPipe.description(), "ValidationPipe");
    }

    #[cfg(feature = "validation")]
    #[tokio::test]
    async fn validation_pipe_passes_value_through() {
        let pipe = ValidationPipe;
        let input = Box::new("test".to_string()) as ExtractedValue;
        let result = pipe.transform(input, &mut context()).await.unwrap();
        let value = *result.downcast::<String>().unwrap();
        assert_eq!(value, "test");
    }
}
