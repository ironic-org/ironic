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
/// Returns a 400 Bad Request when the value is not a valid integer.
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
