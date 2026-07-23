use std::{
    collections::{BTreeMap, HashMap},
    hash::BuildHasher,
};

use serde_json::{Value, json};

/// Produces an `OpenAPI`-compatible JSON Schema for a Rust type.
///
/// Implementors provide the JSON Schema representation that is used when
/// generating request/response schemas and component schemas in the `OpenAPI`
/// document.
///
/// # Examples
///
/// ```ignore
/// use ironic::OpenApiSchema;
///
/// let schema = <Option<String>>::openapi_schema();
/// assert_eq!(schema["type"], "string");
/// assert_eq!(schema["nullable"], true);
/// ```
pub trait OpenApiSchema {
    /// Returns the inline schema for this type.
    #[must_use]
    fn openapi_schema() -> Value;
}

macro_rules! integer_schema {
    ($($ty:ty),+ $(,)?) => {$ (
        impl OpenApiSchema for $ty {
            fn openapi_schema() -> Value {
                json!({ "type": "integer" })
            }
        }
    )+};
}

macro_rules! number_schema {
    ($($ty:ty),+ $(,)?) => {$ (
        impl OpenApiSchema for $ty {
            fn openapi_schema() -> Value {
                json!({ "type": "number" })
            }
        }
    )+};
}

integer_schema!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
number_schema!(f32, f64);

impl OpenApiSchema for bool {
    fn openapi_schema() -> Value {
        json!({ "type": "boolean" })
    }
}

impl OpenApiSchema for String {
    fn openapi_schema() -> Value {
        json!({ "type": "string" })
    }
}

impl OpenApiSchema for str {
    fn openapi_schema() -> Value {
        json!({ "type": "string" })
    }
}

impl<T: OpenApiSchema> OpenApiSchema for Option<T> {
    fn openapi_schema() -> Value {
        let mut schema = T::openapi_schema();
        if let Some(object) = schema.as_object_mut() {
            object.insert("nullable".to_owned(), Value::Bool(true));
        }
        schema
    }
}

impl<T: OpenApiSchema> OpenApiSchema for Vec<T> {
    fn openapi_schema() -> Value {
        json!({ "type": "array", "items": T::openapi_schema() })
    }
}

impl<T: OpenApiSchema, const N: usize> OpenApiSchema for [T; N] {
    fn openapi_schema() -> Value {
        json!({
            "type": "array",
            "items": T::openapi_schema(),
            "minItems": N,
            "maxItems": N
        })
    }
}

impl<T: OpenApiSchema, S: BuildHasher> OpenApiSchema for HashMap<String, T, S> {
    fn openapi_schema() -> Value {
        json!({ "type": "object", "additionalProperties": T::openapi_schema() })
    }
}

impl<T: OpenApiSchema> OpenApiSchema for BTreeMap<String, T> {
    fn openapi_schema() -> Value {
        json!({ "type": "object", "additionalProperties": T::openapi_schema() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_schema() {
        assert_eq!(i32::openapi_schema(), json!({"type": "integer"}));
        assert_eq!(u64::openapi_schema(), json!({"type": "integer"}));
    }

    #[test]
    fn number_schema() {
        assert_eq!(f64::openapi_schema(), json!({"type": "number"}));
        assert_eq!(f32::openapi_schema(), json!({"type": "number"}));
    }

    #[test]
    fn boolean_schema() {
        assert_eq!(bool::openapi_schema(), json!({"type": "boolean"}));
    }

    #[test]
    fn string_schema() {
        assert_eq!(String::openapi_schema(), json!({"type": "string"}));
    }

    #[test]
    fn str_schema() {
        assert_eq!(<str>::openapi_schema(), json!({"type": "string"}));
    }

    #[test]
    fn option_schema_is_nullable() {
        let schema = <Option<String>>::openapi_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["nullable"], true);
    }

    #[test]
    fn option_non_nullable_type_preserved() {
        let schema = <Option<i32>>::openapi_schema();
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["nullable"], true);
    }

    #[test]
    fn vec_schema() {
        let schema = <Vec<i32>>::openapi_schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"], json!({"type": "integer"}));
    }

    #[test]
    fn array_schema_fixed_size() {
        let schema = <[String; 3]>::openapi_schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"], json!({"type": "string"}));
        assert_eq!(schema["minItems"], 3);
        assert_eq!(schema["maxItems"], 3);
    }

    #[test]
    fn hashmap_schema() {
        let schema = <std::collections::HashMap<String, bool>>::openapi_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["additionalProperties"], json!({"type": "boolean"}));
    }

    #[test]
    fn btreemap_schema() {
        let schema = <BTreeMap<String, f64>>::openapi_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["additionalProperties"], json!({"type": "number"}));
    }
}
