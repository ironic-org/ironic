# Static Files

## Purpose

Built-in static file serving with ETag, Cache-Control, and directory index.

## Requirements

### Requirement: System SHALL provide built-in static file serving
The framework SHALL serve static files from a directory via `AxumAdapter::static_files(route_path, fs_dir)`.

#### Scenario: Static file served
- **WHEN** `AxumAdapter::static_files("/static", "./public")` is configured
- **AND** a `GET /static/styles.css` request arrives
- **AND** `./public/styles.css` exists
- **THEN** the file content SHALL be returned with the correct MIME type

#### Scenario: Missing file returns 404
- **WHEN** `GET /static/missing.js` is requested
- **AND** the file does not exist
- **THEN** a 404 response SHALL be returned

### Requirement: Static files SHALL support ETags and Cache-Control
The static file middleware SHALL generate ETag headers from file metadata and respect `If-None-Match` requests, returning 304 Not Modified. Cache-Control headers SHALL be configurable.

#### Scenario: ETag-based caching
- **WHEN** a file is served
- **THEN** the response SHALL include an `ETag` header based on file modification time and size

#### Scenario: 304 Not Modified for cached files
- **WHEN** a request includes `If-None-Match` matching the current ETag
- **THEN** the response SHALL be 304 Not Modified with an empty body
