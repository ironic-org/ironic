## ADDED Requirements

### Requirement: Framework SHALL support response compression
The framework SHALL provide configurable response compression middleware supporting gzip, brotli, and deflate encodings.

#### Scenario: Client accepts gzip receives compressed response
- **WHEN** a request includes `Accept-Encoding: gzip`
- **AND** compression middleware is enabled
- **THEN** the response body SHALL be gzip-compressed and the response SHALL include `Content-Encoding: gzip`

#### Scenario: Client does not accept compression receives uncompressed response
- **WHEN** a request does NOT include `Accept-Encoding`
- **THEN** the response body SHALL NOT be compressed

### Requirement: Compression level SHALL be configurable
The middleware SHALL accept a configurable compression level.

#### Scenario: Compression level setting
- **WHEN** compression middleware is configured with `CompressionLevel::Best`
- **THEN** responses SHALL be compressed at the maximum compression level

### Requirement: Compression middleware SHALL respect content type allowlist
The middleware SHALL only compress responses whose `Content-Type` matches a configurable allowlist.

#### Scenario: Compressible content type is compressed
- **WHEN** a response has `Content-Type: application/json`
- **AND** `application/json` is in the compression allowlist
- **THEN** the response SHALL be compressed

#### Scenario: Non-compressible content type is skipped
- **WHEN** a response has `Content-Type: image/png`
- **THEN** the response SHALL NOT be compressed
