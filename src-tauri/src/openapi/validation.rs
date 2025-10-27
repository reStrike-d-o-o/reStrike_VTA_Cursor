use super::types::{SchemaFormat, SchemaValidationError, SchemaValidationOutcome};
use openapiv3::OpenAPI;
use serde_json::Value;
use serde_path_to_error::Segment;

#[derive(Debug, Clone, Copy)]
pub struct ValidationOptions {
    pub strict: bool,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self { strict: true }
    }
}

pub fn validate(
    raw: &str,
    format: SchemaFormat,
    options: ValidationOptions,
) -> SchemaValidationOutcome {
    let mut errors = Vec::new();
    let value = match parse_to_json_value(raw, format) {
        Ok(v) => v,
        Err(err) => {
            errors.push(err);
            return SchemaValidationOutcome::with_errors(format, errors);
        }
    };

    errors.extend(validate_structure(&value, raw, format, options));

    SchemaValidationOutcome::with_errors(format, errors)
}

fn parse_to_json_value(raw: &str, format: SchemaFormat) -> Result<Value, SchemaValidationError> {
    match format {
        SchemaFormat::Json => serde_json::from_str::<Value>(raw).map_err(|err| {
            SchemaValidationError::new(format!("JSON parse error: {}", err))
                .with_location(Some(err.line() as u64), Some(err.column() as u64))
        }),
        SchemaFormat::Yaml => serde_yaml::from_str::<Value>(raw).map_err(|err| {
            let (line, column) = match err.location() {
                Some(loc) => (Some(loc.line() as u64), Some(loc.column() as u64)),
                None => (None, None),
            };
            SchemaValidationError::new(format!("YAML parse error: {}", err))
                .with_location(line, column)
        }),
    }
}

fn validate_structure(
    value: &Value,
    raw: &str,
    format: SchemaFormat,
    options: ValidationOptions,
) -> Vec<SchemaValidationError> {
    let mut errors = Vec::new();

    // Ensure OpenAPI field exists and is 3.x
    match value.get("openapi") {
        Some(Value::String(version)) => {
            if !version.starts_with("3.") {
                errors.push(pointer_error(
                    "The `openapi` version must be 3.x",
                    "/openapi",
                    raw,
                    format,
                ));
            }
        }
        _ => errors.push(pointer_error(
            "Missing required `openapi` version field",
            "/openapi",
            raw,
            format,
        )),
    }

    // info.title
    match value.pointer("/info/title") {
        Some(Value::String(title)) if !title.trim().is_empty() => {}
        _ => errors.push(pointer_error(
            "Missing or empty `info.title`",
            "/info/title",
            raw,
            format,
        )),
    }

    // info.version
    match value.pointer("/info/version") {
        Some(Value::String(version)) if !version.trim().is_empty() => {}
        _ => errors.push(pointer_error(
            "Missing or empty `info.version`",
            "/info/version",
            raw,
            format,
        )),
    }

    // paths must exist and have at least one entry
    match value.get("paths") {
        Some(Value::Object(paths)) if !paths.is_empty() => {
            let valid_methods = [
                "get", "put", "post", "delete", "options", "head", "patch", "trace",
            ];
            for (path, item) in paths {
                if !path.starts_with('/') {
                    let pointer = format!("/paths/{}", encode_pointer_segment(path));
                    errors.push(pointer_error(
                        "Path keys must start with '/'",
                        &pointer,
                        raw,
                        format,
                    ));
                }

                if let Value::Object(operations) = item {
                    for (method, operation) in operations {
                        if !valid_methods.contains(&method.as_str()) {
                            let pointer = format!(
                                "/paths/{}/{}",
                                encode_pointer_segment(path),
                                encode_pointer_segment(method)
                            );
                            errors.push(pointer_error(
                                "Unsupported HTTP method in paths object",
                                &pointer,
                                raw,
                                format,
                            ));
                            continue;
                        }

                        if !operation.is_object() {
                            let pointer = format!(
                                "/paths/{}/{}",
                                encode_pointer_segment(path),
                                encode_pointer_segment(method)
                            );
                            errors.push(pointer_error(
                                "Operation definition must be an object",
                                &pointer,
                                raw,
                                format,
                            ));
                            continue;
                        }

                        if !operation
                            .get("responses")
                            .map(|responses| {
                                responses.is_object() && !responses.as_object().unwrap().is_empty()
                            })
                            .unwrap_or(false)
                        {
                            let pointer = format!(
                                "/paths/{}/{}/responses",
                                encode_pointer_segment(path),
                                encode_pointer_segment(method)
                            );
                            errors.push(pointer_error(
                                "Operations must define at least one response",
                                &pointer,
                                raw,
                                format,
                            ));
                        }
                    }
                } else {
                    let pointer = format!("/paths/{}", encode_pointer_segment(path));
                    errors.push(pointer_error(
                        "Path item must be an object containing operations",
                        &pointer,
                        raw,
                        format,
                    ));
                }
            }
        }
        Some(_) => errors.push(pointer_error(
            "`paths` must be an object with at least one entry",
            "/paths",
            raw,
            format,
        )),
        None => errors.push(pointer_error(
            "Missing required `paths` section",
            "/paths",
            raw,
            format,
        )),
    }

    if options.strict {
        errors.extend(run_openapiv3_deserialize(value, raw, format));
    }

    errors
}

fn run_openapiv3_deserialize(
    value: &Value,
    raw: &str,
    format: SchemaFormat,
) -> Vec<SchemaValidationError> {
    let mut errors = Vec::new();
    let json_string = match serde_json::to_string(value) {
        Ok(json) => json,
        Err(err) => {
            errors.push(SchemaValidationError::new(format!(
                "Failed to serialize value for validation: {}",
                err
            )));
            return errors;
        }
    };

    let mut deserializer = serde_json::Deserializer::from_str(&json_string);
    match serde_path_to_error::deserialize::<_, OpenAPI>(&mut deserializer) {
        Ok(_) => {}
        Err(err) => {
            let pointer = pointer_from_path(err.path());
            let (line, column) = locate_pointer(raw, format, &pointer);
            errors.push(
                SchemaValidationError::new(format!("OpenAPI structure error: {}", err.inner()))
                    .with_pointer(if pointer.is_empty() {
                        None
                    } else {
                        Some(pointer)
                    })
                    .with_location(line, column),
            );
        }
    }

    errors
}

fn pointer_error(
    message: &str,
    pointer: &str,
    raw: &str,
    format: SchemaFormat,
) -> SchemaValidationError {
    let (line, column) = locate_pointer(raw, format, pointer);
    SchemaValidationError::new(message)
        .with_pointer(if pointer.is_empty() {
            None
        } else {
            Some(pointer.to_string())
        })
        .with_location(line, column)
}

fn pointer_from_path(path: &serde_path_to_error::Path) -> String {
    let mut pointer = String::new();
    for segment in path.iter() {
        pointer.push('/');
        match segment {
            Segment::Map { key } => pointer.push_str(&encode_pointer_segment(key)),
            Segment::Seq { index } => pointer.push_str(&index.to_string()),
            Segment::Enum { variant } => pointer.push_str(&encode_pointer_segment(variant)),
            _ => pointer.push_str("value"),
        }
    }
    pointer
}

fn locate_pointer(raw: &str, format: SchemaFormat, pointer: &str) -> (Option<u64>, Option<u64>) {
    if pointer.is_empty() {
        return (Some(1), Some(1));
    }

    let decoded_segments: Vec<String> = pointer
        .trim_start_matches('/')
        .split('/')
        .map(decode_pointer_segment)
        .collect();

    if decoded_segments.is_empty() {
        return (Some(1), Some(1));
    }

    let lines: Vec<&str> = raw.lines().collect();
    let mut candidate_line: Option<usize> = None;

    for segment in decoded_segments.iter() {
        if segment.parse::<usize>().is_ok() {
            continue;
        }

        let patterns = match format {
            SchemaFormat::Json => vec![format!("\"{}\"", segment), format!("\"{}\":", segment)],
            SchemaFormat::Yaml => vec![format!("{}:", segment)],
        };

        let start_index = candidate_line.unwrap_or(0);
        let mut found = None;
        for (idx, line) in lines.iter().enumerate().skip(start_index) {
            if patterns.iter().any(|pattern| line.contains(pattern)) {
                found = Some(idx);
                break;
            }
        }

        if let Some(idx) = found {
            candidate_line = Some(idx);
        } else {
            return (None, None);
        }
    }

    if let Some(line_idx) = candidate_line {
        let line_content = lines.get(line_idx).copied().unwrap_or("");
        let segment = decoded_segments.last().unwrap();
        let patterns = match format {
            SchemaFormat::Json => vec![format!("\"{}\"", segment), format!("\"{}\":", segment)],
            SchemaFormat::Yaml => vec![format!("{}:", segment)],
        };
        let mut column = None;
        for pattern in patterns {
            if let Some(pos) = line_content.find(&pattern) {
                column = Some((pos + 1) as u64);
                break;
            }
        }

        (Some((line_idx + 1) as u64), column.or(Some(1)))
    } else {
        (None, None)
    }
}

fn encode_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn decode_pointer_segment(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}
