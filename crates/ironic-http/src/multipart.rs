use std::marker::PhantomData;

use multer::Error as MulterError;

use crate::{
    ExtractFuture, ExtractedValue, HttpError, HttpStatus, ParameterExtractor, RequestContext,
};

fn is_size_limit_error(err: &MulterError) -> bool {
    matches!(
        err,
        MulterError::FieldSizeExceeded { .. } | MulterError::StreamSizeExceeded { .. }
    )
}

/// Configuration for multipart form data extraction.
#[derive(Clone, Debug)]
pub struct MultipartConfig {
    /// Maximum size for a single file upload in bytes (default: 5 MiB).
    pub max_file_size: usize,
    /// Maximum size for a single non-file form field in bytes (default: 256 KiB).
    pub max_field_size: usize,
}

impl Default for MultipartConfig {
    fn default() -> Self {
        Self {
            max_file_size: 5 * 1024 * 1024,
            max_field_size: 256 * 1024,
        }
    }
}

/// Represents an uploaded file from a multipart form.
#[derive(Clone, Debug)]
pub struct UploadedFile {
    /// The field name from the form.
    pub field_name: String,
    /// The original file name, if provided in the `Content-Disposition` header.
    pub file_name: Option<String>,
    /// The content type of the file, if provided.
    pub content_type: Option<String>,
    /// The size of the file in bytes.
    pub size: usize,
    /// The raw file data.
    pub data: Vec<u8>,
}

/// The extracted value from a multipart form, containing structured fields and files.
#[derive(Clone, Debug)]
pub struct MultipartFormData<T> {
    /// Deserialized form fields.
    pub fields: T,
    /// Uploaded files.
    pub files: Vec<UploadedFile>,
}

/// Extractor for `multipart/form-data` requests.
///
/// Parses the request body as multipart form data, deserializes non-file
/// fields into `T`, and collects uploaded files into [`MultipartFormData::files`].
///
/// # Usage
///
/// ```rust,ignore
/// #[derive(serde::Deserialize)]
/// struct UploadDto {
///     title: String,
///     description: String,
/// }
///
/// #[post("/upload")]
/// async fn upload(
///     &self,
///     #[custom(MultipartForm<UploadDto>)]
///     form: MultipartFormData<UploadDto>,
/// ) -> Result<Json<()>, HttpError> {
///     let fields = form.fields;
///     let files = form.files;
///     Ok(Json(()))
/// }
/// ```
pub struct MultipartForm<T> {
    config: MultipartConfig,
    _marker: PhantomData<fn() -> T>,
}

impl<T> MultipartForm<T> {
    /// Creates a new multipart form extractor with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: MultipartConfig::default(),
            _marker: PhantomData,
        }
    }

    /// Creates a new multipart form extractor with a custom configuration.
    #[must_use]
    pub fn with_config(config: MultipartConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }
}

impl<T> Default for MultipartForm<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ParameterExtractor for MultipartForm<T>
where
    T: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        let config = self.config.clone();
        Box::pin(async move {
            let content_type = context
                .request()
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| {
                    HttpError::bad_request(
                        "RF_MULTIPART_MISSING_CONTENT_TYPE",
                        "Missing Content-Type header for multipart request",
                    )
                })?;

            let boundary = multer::parse_boundary(content_type).map_err(|_| {
                HttpError::bad_request(
                    "RF_MULTIPART_INVALID_BOUNDARY",
                    "Invalid or missing multipart boundary in Content-Type",
                )
            })?;

            let body = context.request().body().to_vec();
            let stream = futures_util::stream::once(async move {
                Ok::<Vec<u8>, std::io::Error>(body)
            });

            let max_file = u64::from(u32::try_from(config.max_file_size).unwrap_or(u32::MAX));
            let max_field = u64::from(u32::try_from(config.max_field_size).unwrap_or(u32::MAX));

            let size_limit = multer::SizeLimit::new()
                .whole_stream(max_file + max_field * 10)
                .per_field(max_file.max(max_field));

            let constraints = multer::Constraints::new().size_limit(size_limit);

            let mut multipart =
                multer::Multipart::with_constraints(stream, boundary, constraints);

            let mut field_map: std::collections::BTreeMap<String, String> =
                std::collections::BTreeMap::new();
            let mut files: Vec<UploadedFile> = Vec::new();

            while let Some(field) = multipart.next_field().await.map_err(|e| {
                let (code, status) = if is_size_limit_error(&e) {
                    ("RF_MULTIPART_FIELD_TOO_LARGE", HttpStatus::PAYLOAD_TOO_LARGE)
                } else {
                    ("RF_MULTIPART_PARSE_ERROR", HttpStatus::BAD_REQUEST)
                };
                HttpError::new(
                    status,
                    code,
                    format!("Failed to parse multipart form data: {e}"),
                )
            })? {
                let field_name = field.name().unwrap_or("").to_string();

                if field_name.is_empty() {
                    continue;
                }

                let is_file = field.file_name().is_some();
                let file_name = field.file_name().map(String::from);
                let content_type = field.content_type().map(|m| m.to_string());

                let data = field.bytes().await.map_err(|e| {
                    let (code, status) = if is_size_limit_error(&e) {
                        ("RF_MULTIPART_FIELD_TOO_LARGE", HttpStatus::PAYLOAD_TOO_LARGE)
                    } else {
                        ("RF_MULTIPART_FIELD_READ_ERROR", HttpStatus::BAD_REQUEST)
                    };
                    HttpError::new(
                        status,
                        code,
                        format!("Failed to read multipart field `{field_name}`: {e}"),
                    )
                })?;

                if is_file {
                    files.push(UploadedFile {
                        field_name,
                        file_name,
                        content_type,
                        size: data.len(),
                        data: data.to_vec(),
                    });
                } else {
                    let text = String::from_utf8(data.to_vec()).map_err(|_| {
                        HttpError::bad_request(
                            "RF_MULTIPART_INVALID_UTF8",
                            format!("Non-file field `{field_name}` is not valid UTF-8"),
                        )
                    })?;
                    field_map.insert(field_name, text);
                }
            }

            let encoded = serde_urlencoded::to_string(&field_map).map_err(|_| {
                HttpError::bad_request(
                    "RF_MULTIPART_FORM_ENCODE",
                    "Failed to serialize form fields",
                )
            })?;

            let fields: T = serde_urlencoded::from_str(&encoded).map_err(|e| {
                HttpError::bad_request(
                    "RF_MULTIPART_DESERIALIZE",
                    format!("Failed to deserialize form fields: {e}"),
                )
            })?;

            Ok(Box::new(MultipartFormData { fields, files }) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "multipart form data"
    }
}
