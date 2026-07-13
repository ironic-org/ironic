use std::{
    collections::{BTreeMap, HashMap},
    hash::BuildHasher,
};

use serde_json::{Value, json};

/// Produces an `OpenAPI`-compatible JSON Schema for a Rust type.
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
