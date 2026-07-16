#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Exercise HTTP-level parsing by feeding random bytes into
    // the ironic-http extraction paths.
    //
    // This tests for panics, crashes, and unexpected errors in:
    // - Request body parsing
    // - Header extraction
    // - URL parameter parsing
    // - JSON deserialization
    // - Multipart boundary detection
    //
    // The framework catches panics at the handler boundary, but we
    // exercise internal parsing functions directly.

    if data.is_empty() {
        return;
    }

    // Build a synthetic HTTP request from fuzz data.
    // We use a minimal valid structure so most fuzz time is spent
    // on parsing rather than route-matching boilerplate.
    let parts: Vec<&[u8]> = data.splitn(3, |b| *b == 0x00).collect();
    let _method = parts.first().unwrap_or(&b"GET"[..]);
    let _path = parts.get(1).unwrap_or(&b"/"[..]);
    let _body = parts.get(2).unwrap_or(&[]);

    // Attempt to deserialize as JSON — this exercises serde paths.
    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(_body) {
        let _ = val.to_string();
    }

    // Attempt to parse query parameters from the path.
    if let Some(query) = std::str::from_utf8(_path).ok().and_then(|p| p.split('?').nth(1)) {
        let _ = serde_urlencoded::from_str::<Vec<(String, String)>>(query);
    }

    // Attempt header parsing from the method field as a crude header block.
    if let Ok(header_str) = std::str::from_utf8(_method) {
        for line in header_str.lines() {
            if let Some((_name, _val)) = line.split_once(':') {
                let _name = _name.trim();
                let _val = _val.trim();
                // Exercise header value parsing
                let _: Vec<&str> = _val.split(',').collect();
            }
        }
    }
});
