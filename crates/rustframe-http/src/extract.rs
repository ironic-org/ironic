use std::{any::Any, fmt::Display, future::Future, marker::PhantomData, pin::Pin, str::FromStr};

use serde::de::DeserializeOwned;

use crate::{HttpError, RequestContext};

/// A type-erased extracted handler argument.
pub type ExtractedValue = Box<dyn Any + Send>;

/// The asynchronous result of parameter extraction.
pub type ExtractFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ExtractedValue, HttpError>> + Send + 'a>>;

/// Extracts one typed handler parameter from a request context.
pub trait ParameterExtractor: Send + Sync + 'static {
    /// Extracts and erases one parameter.
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a>;

    /// Returns a short description for diagnostics.
    fn description(&self) -> &'static str;
}

/// Extracts and parses one named path parameter.
#[derive(Debug)]
pub struct PathParameter<T> {
    name: &'static str,
    marker: PhantomData<fn() -> T>,
}

impl<T> PathParameter<T> {
    /// Creates a named path extractor.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            marker: PhantomData,
        }
    }
}

impl<T> ParameterExtractor for PathParameter<T>
where
    T: FromStr + Send + Sync + 'static,
    T::Err: Display,
{
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let raw = context.request().path_parameter(self.name).ok_or_else(|| {
                HttpError::bad_request(
                    "RF_HTTP_MISSING_PATH_PARAMETER",
                    format!("Missing path parameter `{}`", self.name),
                )
            })?;
            let value = raw.parse::<T>().map_err(|error| {
                HttpError::bad_request(
                    "RF_HTTP_INVALID_PATH_PARAMETER",
                    format!("Invalid path parameter `{}`: {error}", self.name),
                )
            })?;
            Ok(Box::new(value) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "path parameter"
    }
}

/// Extracts and parses one named request header.
#[derive(Debug)]
pub struct HeaderParameter<T> {
    name: &'static str,
    marker: PhantomData<fn() -> T>,
}

impl<T> HeaderParameter<T> {
    /// Creates a named header extractor.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            marker: PhantomData,
        }
    }
}

impl<T> ParameterExtractor for HeaderParameter<T>
where
    T: FromStr + Send + Sync + 'static,
    T::Err: Display,
{
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let raw = context
                .request()
                .headers()
                .get(self.name)
                .ok_or_else(|| {
                    HttpError::bad_request(
                        "RF_HTTP_MISSING_HEADER",
                        format!("Missing request header `{}`", self.name),
                    )
                })?
                .to_str()
                .map_err(|error| {
                    HttpError::bad_request(
                        "RF_HTTP_INVALID_HEADER",
                        format!("Invalid request header `{}`: {error}", self.name),
                    )
                })?;
            let value = raw.parse::<T>().map_err(|error| {
                HttpError::bad_request(
                    "RF_HTTP_INVALID_HEADER",
                    format!("Invalid request header `{}`: {error}", self.name),
                )
            })?;
            Ok(Box::new(value) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "request header"
    }
}

/// Deserializes the complete query string into `T`.
#[derive(Debug, Default)]
pub struct QueryParameters<T>(PhantomData<fn() -> T>);

impl<T> QueryParameters<T> {
    /// Creates a query extractor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> ParameterExtractor for QueryParameters<T>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let query = context.request().uri().query().unwrap_or_default();
            let value = serde_urlencoded::from_str::<T>(query).map_err(|error| {
                HttpError::bad_request(
                    "RF_HTTP_INVALID_QUERY",
                    format!("Invalid query parameters: {error}"),
                )
            })?;
            Ok(Box::new(value) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "query parameters"
    }
}

/// Deserializes the request body from JSON into `T`.
#[derive(Debug, Default)]
pub struct JsonBody<T>(PhantomData<fn() -> T>);

impl<T> JsonBody<T> {
    /// Creates a JSON body extractor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> ParameterExtractor for JsonBody<T>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let value = serde_json::from_slice::<T>(context.request().body()).map_err(|error| {
                HttpError::bad_request(
                    "RF_HTTP_INVALID_JSON_BODY",
                    format!("Invalid JSON request body: {error}"),
                )
            })?;
            Ok(Box::new(value) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "JSON body"
    }
}

#[cfg(test)]
mod tests {
    use http::Uri;
    use serde::Deserialize;

    use super::*;
    use crate::{FrameworkRequest, HeaderMap, HttpMethod};

    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct Query {
        page: u32,
    }

    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct Payload {
        name: String,
    }

    fn context(uri: &str, body: &[u8]) -> RequestContext {
        RequestContext::new(FrameworkRequest::new(
            HttpMethod::POST,
            uri.parse::<Uri>().unwrap(),
            HeaderMap::new(),
            body.to_vec(),
        ))
    }

    #[tokio::test]
    async fn extracts_query_objects() {
        let value = QueryParameters::<Query>::new()
            .extract(&mut context("/users?page=3", &[]))
            .await
            .unwrap()
            .downcast::<Query>()
            .unwrap();
        assert_eq!(*value, Query { page: 3 });
    }

    #[tokio::test]
    async fn extracts_json_bodies() {
        let value = JsonBody::<Payload>::new()
            .extract(&mut context("/users", br#"{"name":"Ada"}"#))
            .await
            .unwrap()
            .downcast::<Payload>()
            .unwrap();
        assert_eq!(value.name, "Ada");
    }

    #[tokio::test]
    async fn rejects_malformed_json() {
        let error = JsonBody::<Payload>::new()
            .extract(&mut context("/users", b"{"))
            .await
            .unwrap_err();
        assert_eq!(error.code(), "RF_HTTP_INVALID_JSON_BODY");
    }

    #[tokio::test]
    async fn extracts_headers() {
        let mut request = context("/users", &[]);
        request
            .request_mut()
            .headers_mut()
            .insert("x-page", "7".parse().unwrap());
        let value = HeaderParameter::<u32>::new("x-page")
            .extract(&mut request)
            .await
            .unwrap()
            .downcast::<u32>()
            .unwrap();
        assert_eq!(*value, 7);
    }
}
