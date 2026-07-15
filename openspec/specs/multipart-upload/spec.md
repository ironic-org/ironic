# Multipart Upload

## Purpose

Multipart form data extractor with streaming, per-file limits, configurable total body size.

## Requirements

### Requirement: System SHALL support multipart form data extraction
The framework SHALL provide a `MultipartForm<T>` extractor that can parse `multipart/form-data` request bodies with typed fields and file uploads.

#### Scenario: Text fields extracted from multipart
- **WHEN** a `POST` request with `Content-Type: multipart/form-data` arrives
- **AND** the body contains text fields `name` and `email`
- **THEN** a `MultipartForm<MyForm>` extractor SHALL deserialize them into the struct

### Requirement: System SHALL support per-field size limits
The multipart extractor SHALL allow configuring per-field and per-file maximum sizes.

#### Scenario: File exceeds size limit
- **WHEN** an uploaded file exceeds the configured `max_file_size` (default 10 MiB)
- **THEN** the request SHALL be rejected with a 413 Payload Too Large error

### Requirement: System SHALL stream multipart fields
The multipart extractor SHALL process fields incrementally, not buffer the entire body before parsing.

#### Scenario: Streaming field processing
- **WHEN** a large multipart request is received
- **THEN** text fields SHALL be extracted before file fields complete
- **AND** memory usage SHALL NOT grow with total request size (bounded by per-field limits)
